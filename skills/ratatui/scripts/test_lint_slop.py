import pathlib
import subprocess
import tempfile
import unittest


SCRIPT = pathlib.Path(__file__).with_name("lint-slop.sh")


class LintSlopTests(unittest.TestCase):
    def run_scanner(self, root):
        return subprocess.run(
            ["bash", str(SCRIPT), str(root)],
            capture_output=True,
            text=True,
            check=False,
        )

    def test_project_root_named_tests_is_scanned(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "tests"
            source = root / "src" / "main.rs"
            source.parent.mkdir(parents=True)
            source.write_text("let color = Color::Rgb(1, 2, 3);\n")
            result = self.run_scanner(root)
            self.assertEqual(result.returncode, 1)
            self.assertIn("colors are defined", result.stdout)

    def test_target_directory_is_excluded_relative_to_project_root(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "project"
            (root / "target").mkdir(parents=True)
            (root / "src").mkdir()
            (root / "target" / "generated.rs").write_text("let color = Color::Rgb(1, 2, 3);\n")
            (root / "src" / "main.rs").write_text("fn main() {}\n")
            result = self.run_scanner(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_nested_test_target_is_not_treated_as_production_source(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "project"
            (root / "tests").mkdir(parents=True)
            (root / "src").mkdir()
            (root / "tests" / "render.rs").write_text("let color = Color::Rgb(1, 2, 3);\\n")
            (root / "src" / "main.rs").write_text("fn main() {}\\n")
            result = self.run_scanner(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_test_module_cannot_hide_later_production_source(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "project"
            root.mkdir()
            source = root / "main.rs"
            source.write_text(
                '#[cfg(test)]\nmod tests {\n    const BRACE: &str = "}";\n}\n'
                'fn production() { let color = Color::Rgb(1, 2, 3); }\n'
            )
            result = self.run_scanner(root)
            self.assertEqual(result.returncode, 1)
            self.assertIn("production()", result.stdout)

    def test_theme_module_may_define_colors(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary) / "project"
            root.mkdir()
            (root / "theme.rs").write_text("let color = Color::Rgb(1, 2, 3);\n")
            result = self.run_scanner(root)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
