#![allow(missing_docs)]

use std::{io, path::PathBuf, thread::sleep, time::Duration};

use clap::{Parser, ValueEnum};
use linux_uevent::{Action, UeventSocket};
use neli as _;
use rustix as _;
use tracing::{Level, debug, info};
use tracing_subscriber::EnvFilter;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum CliAction {
    Add,
    Bind,
    Change,
    Move,
    Offline,
    Online,
    Remove,
    Unbind,
}

impl From<CliAction> for Action {
    fn from(value: CliAction) -> Self {
        match value {
            CliAction::Add => Action::Add,
            CliAction::Bind => Action::Bind,
            CliAction::Change => Action::Change,
            CliAction::Move => Action::Move,
            CliAction::Offline => Action::Offline,
            CliAction::Online => Action::Online,
            CliAction::Remove => Action::Remove,
            CliAction::Unbind => Action::Unbind,
        }
    }
}

#[derive(Parser)]
struct CliArgs {
    #[arg(long, short, value_enum)]
    action: Option<CliAction>,

    #[arg(long, short)]
    subsystem: Option<String>,

    #[arg(long, short = 'S')]
    sysfs_path: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let args = CliArgs::parse();

    tracing_subscriber::fmt()
        .without_time()
        .with_ansi(true)
        .with_max_level(Level::INFO)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let mut socket = UeventSocket::new()?;

    loop {
        if let Some(event) = socket.event_filter(|e| {
            if let Some(action) = args.action {
                if e.action() != Action::from(action) {
                    return false;
                }
            }

            if let Some(subsystem) = &args.subsystem {
                if e.subsystem() != subsystem {
                    return false;
                }
            }

            if let Some(path) = &args.sysfs_path {
                if e.path() != path {
                    return false;
                }
            }

            true
        })? {
            info!("Found event {event:#?}");
        } else {
            debug!("No message received yet. Sleeping.");
            sleep(Duration::from_millis(100));
        }
    }
}
