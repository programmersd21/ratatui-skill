use std::path::PathBuf;

use color_eyre::eyre::WrapErr;
use todo_tui::App;

const USAGE: &str = "usage: todo [FILE]\n\nManage a todo list. Press ? inside for keys.\nDefault file: ~/.todo-tui.txt (or TODO_FILE)";

#[allow(clippy::print_stdout)] // before the terminal is taken over
fn main() -> color_eyre::Result<()> {
    // Short error messages: no source locations or backtrace hints.
    color_eyre::config::HookBuilder::default()
        .display_location_section(false)
        .display_env_section(false)
        .install()?;

    let arg = std::env::args_os().nth(1);
    if matches!(
        arg.as_deref().and_then(|a| a.to_str()),
        Some("-h" | "--help")
    ) {
        println!("{USAGE}");
        return Ok(());
    }
    let path = arg
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("TODO_FILE").map(PathBuf::from))
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from);
            home.join(".todo-tui.txt")
        });

    let app = App::new(path.clone()).wrap_err_with(|| format!("cannot open {}", path.display()))?;
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
