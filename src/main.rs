use std::io;

use anyhow::Result;
use dmitui::{
    app::App,
    event::{Event, EventHandler},
    handlers::handle_key_events,
    tui::Tui,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use clap::{Command, crate_description, crate_version};

fn main() -> Result<()> {
    Command::new("dmitui")
        .about(crate_description!())
        .version(crate_version!())
        .get_matches();

    if unsafe { libc::geteuid() } != 0 {
        eprintln!("dmitui must be run as root");
        std::process::exit(1);
    }

    let mut app = App::new()?;

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;
        if let Event::Key(key_event) = tui.events.next()? {
            handle_key_events(key_event, &mut app)?;
        }
    }

    tui.exit()?;

    Ok(())
}
