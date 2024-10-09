use std::{io::stdout, panic};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    Terminal,
};

pub fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    install_panic_hook();
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    Ok(terminal)
}

pub fn restore_terminal() -> color_eyre::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn install_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));
}
