#![doc = include_str!("../README.md")]

use core::{cmp::max, fmt, str::FromStr};
use std::{
    collections::HashMap,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use neli::{
    consts::socket::{Msg, NlFamily},
    socket::NlSocket,
    utils::Groups,
};
use tracing::trace;

/// Uevent Action
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Action {
    /// A device has been added.
    Add,

    /// A driver has been bound to a device
    Bind,

    /// A device has been modified
    Change,

    /// A device has been renamed
    Move,

    /// A device is ready to be hot-removed
    Offline,

    /// A device is now online
    Online,

    /// A device has been removed.
    Remove,

    /// A driver has been unbound to a device
    Unbind,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Action::Add => "add",
            Action::Bind => "bind",
            Action::Change => "change",
            Action::Move => "move",
            Action::Offline => "offline",
            Action::Online => "online",
            Action::Remove => "remove",
            Action::Unbind => "unbind",
        })
    }
}

impl FromStr for Action {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add" => Ok(Action::Add),
            "bind" => Ok(Action::Bind),
            "change" => Ok(Action::Change),
            "move" => Ok(Action::Move),
            "offline" => Ok(Action::Offline),
            "online" => Ok(Action::Online),
            "remove" => Ok(Action::Remove),
            "unbind" => Ok(Action::Unbind),
            _ => Err(io::Error::new(ErrorKind::InvalidData, "Unknown Action")),
        }
    }
}

/// A Uevent
#[derive(Debug)]
pub struct Uevent {
    action: Action,
    path: PathBuf,
    seqnum: usize,
    subsystem: String,
    attributes: HashMap<String, String>,
}

impl Uevent {
    /// Returns the [Action] notified by the event
    #[must_use]
    pub fn action(&self) -> Action {
        self.action
    }

    /// Returns an attribute value, if it exists
    #[must_use]
    pub fn attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).map(String::as_str)
    }

    /// Returns the sysfs path of the device the event is for.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the event sequence number
    #[must_use]
    pub fn sequence_number(&self) -> usize {
        self.seqnum
    }

    /// Returns the subsystem name the event is for.
    #[must_use]
    pub fn subsystem(&self) -> &str {
        &self.subsystem
    }
}

impl TryFrom<&[u8]> for Uevent {
    type Error = io::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut lines = bytes.split(|byte| *byte == 0);

        let event = String::from_utf8_lossy(lines.next().ok_or(io::Error::new(
            ErrorKind::UnexpectedEof,
            "Event is not properly formatted",
        ))?);
        let event_parts = event.split('@').collect::<Vec<_>>();

        if event_parts.len() != 2 {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Invalid Event Content",
            ));
        }

        let action = event_parts[0].parse::<Action>()?;
        let path = PathBuf::from(format!("/sys{}", event_parts[1]));

        let mut attributes = HashMap::new();

        for item in lines {
            if item.is_empty() {
                continue;
            }

            let item_str = String::from_utf8_lossy(item);
            trace!("Parsing event string {item_str}");

            let parts = item_str.split('=').collect::<Vec<_>>();

            // Action and devpath have already been handled.
            if parts[0] == "ACTION" || parts[0] == "DEVPATH" {
                continue;
            }

            attributes.insert(parts[0].to_owned(), parts[1].to_owned());
        }

        Ok(Uevent {
            action,
            path,
            subsystem: attributes.remove("SUBSYSTEM").ok_or(io::Error::new(
                ErrorKind::InvalidData,
                "Missing SUBSYSTEM key",
            ))?,
            seqnum: attributes
                .remove("SEQNUM")
                .ok_or(io::Error::new(ErrorKind::InvalidData, "Missing SEQNUM key"))?
                .parse::<usize>()
                .map_err(|_e| io::Error::new(ErrorKind::InvalidData, "Invalid Sequence Number"))?,
            attributes,
        })
    }
}

/// A Netlink Socket to receive Uevents
pub struct UeventSocket {
    buffer_size: usize,
    socket: NlSocket,
}

impl fmt::Debug for UeventSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UeventSocket")
            .field("buffer_size", &self.buffer_size)
            .finish_non_exhaustive()
    }
}

impl UeventSocket {
    /// Creates a new Netlink Socket to receive Uevents
    ///
    /// # Errors
    ///
    /// If the socket cannot be created
    pub fn new() -> io::Result<Self> {
        let socket = NlSocket::connect(NlFamily::KobjectUevent, None, Groups::empty())?;
        socket.nonblock()?;
        socket.add_mcast_membership(Groups::new_groups(&[1]))?;

        Ok(Self {
            buffer_size: max(8192, rustix::param::page_size()),
            socket,
        })
    }

    /// Returns an available event
    ///
    /// This function is non-blocking, and will return an event only if there's one available.
    ///
    /// # Errors
    ///
    /// If the socket access fails, or if the uevent cannot be parsed
    pub fn event(&mut self) -> io::Result<Option<Uevent>> {
        let mut buf = vec![0; self.buffer_size];

        match self.socket.recv(&mut buf, Msg::empty()) {
            Ok((msg_len, _)) => Ok(Some(Uevent::try_from(&buf[..msg_len])?)),
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Returns an available event if the given predicate returns true.
    ///
    /// This function is non-blocking, and will return an event only if there's one available.
    ///
    /// # Errors
    ///
    /// If the socket access fails, or if the uevent cannot be parsed
    pub fn event_filter<F>(&mut self, filter: F) -> io::Result<Option<Uevent>>
    where
        F: Fn(&Uevent) -> bool,
    {
        self.event().map(|o| o.filter(filter))
    }
}
