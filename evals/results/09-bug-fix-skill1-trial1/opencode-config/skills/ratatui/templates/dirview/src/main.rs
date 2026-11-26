use std::path::PathBuf;

use color_eyre::eyre::WrapErr;
use dirview::App;

const USAGE: &str = "usage: dirview [DIR]\n\nBrowse a directory. Press ? inside for keys.";

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
    let dir = arg.map_or_else(|| PathBuf::from("."), PathBuf::from);

    let app = App::new(dir.clone()).wrap_err_with(|| format!("cannot open {}", dir.display()))?;
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
