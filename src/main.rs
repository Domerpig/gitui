mod app;
mod components;
mod git_status;
mod git_utils;
mod keys;
mod poll;
mod strings;
mod ui;

use crate::{app::App, poll::QueueEvent};
use crossbeam_channel::{select, unbounded};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand, Result,
};
use scopetime::scope_time;
use simplelog::*;
use std::{env, fs, fs::File, io};
use tui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<()> {
    setup_logging();
    enable_raw_mode()?;
    io::stdout()
        .execute(EnterAlternateScreen)?
        .execute(EnableMouseCapture)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    terminal.clear()?;

    let (tx, rx) = unbounded();

    let mut app = App::new(tx);

    let receiver = poll::start_polling_thread();

    app.update();

    loop {
        let mut events: Vec<QueueEvent> = Vec::new();
        select! {
            recv(receiver) -> inputs => events = inputs.unwrap(),
            recv(rx) -> _ => events.push(QueueEvent::AsyncEvent),
        }

        {
            scope_time!("loop");

            for e in events {
                match e {
                    QueueEvent::InputEvent(ev) => app.event(ev),
                    QueueEvent::Tick => app.update(),
                    QueueEvent::AsyncEvent => app.update_diff(),
                }
            }

            terminal.draw(|mut f| app.draw(&mut f))?;

            if app.is_quit() {
                break;
            }
        }
    }

    io::stdout()
        .execute(LeaveAlternateScreen)?
        .execute(DisableMouseCapture)?;
    disable_raw_mode()?;
    Ok(())
}

fn setup_logging() {
    if env::var("GITUI_LOGGING").is_ok() {
        let mut path = dirs::home_dir().unwrap();
        path.push(".gitui");
        path.push("gitui.log");
        fs::create_dir(path.parent().unwrap()).unwrap_or_default();

        let _ = WriteLogger::init(
            LevelFilter::Trace,
            Config::default(),
            File::create(path).unwrap(),
        );
    }
}
