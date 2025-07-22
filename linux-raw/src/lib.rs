#![doc = include_str!("../README.md")]

use core::{cmp::Ordering, fmt, num::ParseIntError, str::FromStr};

/// A kernel version representation
#[derive(Clone, Debug)]
pub struct KernelVersion {
    major: u16,
    minor: u8,
    patch: u8,
    sublevel: Option<u8>,
    extraversion: Option<String>,
}

impl KernelVersion {
    /// Returns the Linux version we run from.
    ///
    /// # Panics
    ///
    /// If the kernel returns a poorly formatted string.
    #[must_use]
    pub fn current() -> Self {
        let uname = rustix::system::uname();
        let version_str = uname
            .release()
            .to_str()
            .expect("The kernel release name is always in ASCII.");

        KernelVersion::from_str(version_str)
            .expect("The version comes straight from uname. It's valid.")
    }

    /// Creates a new kernel version, with no sublevel version number or extra version.
    #[must_use]
    pub fn new(major: u16, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
            sublevel: None,
            extraversion: None,
        }
    }

    /// Major Version Number
    #[must_use]
    pub fn major(&self) -> u16 {
        self.major
    }

    /// Minor Version Number
    #[must_use]
    pub fn minor(&self) -> u8 {
        self.minor
    }

    /// Patch Version Number
    #[must_use]
    pub fn patch(&self) -> u8 {
        self.patch
    }

    /// Sublevel Version Number
    ///
    /// This was used on Linux up to (and including) Linux 2.6.39 stable releases.
    #[must_use]
    pub fn sublevel(&self) -> Option<u8> {
        self.sublevel
    }

    /// Extra Version
    #[must_use]
    pub fn extraversion(&self) -> Option<&str> {
        self.extraversion.as_deref()
    }
}

impl fmt::Display for KernelVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}.{}.{}", self.major, self.minor, self.patch))
    }
}

impl PartialOrd for KernelVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.major.partial_cmp(&other.major) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }

        match self.minor.partial_cmp(&other.minor) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }

        match self.patch.partial_cmp(&other.patch) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }

        match self.sublevel.partial_cmp(&other.sublevel) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }

        // We ignore extraversion when comparing
        Some(Ordering::Equal)
    }
}

impl PartialEq for KernelVersion {
    fn eq(&self, other: &Self) -> bool {
        PartialOrd::partial_cmp(&self, &other) == Some(Ordering::Equal)
    }
}

impl From<u32> for KernelVersion {
    fn from(value: u32) -> Self {
        let major = ((value >> 16) & Into::<u32>::into(u16::MAX))
            .try_into()
            .expect("I'm terrible at math.");

        let minor = ((value >> 8) & Into::<u32>::into(u8::MAX))
            .try_into()
            .expect("I'm terrible at math.");

        let patch = (value & Into::<u32>::into(u8::MAX))
            .try_into()
            .expect("I'm terrible at math.");

        Self {
            major,
            minor,
            patch,
            sublevel: None,
            extraversion: None,
        }
    }
}

/// An error returned when parsing a kernel version
#[derive(Debug, PartialEq)]
pub enum KernelVersionParseError {
    /// The format of the version String is invalid
    InvalidFormat,

    /// A version number can't be parsed
    ParseInt(ParseIntError),
}

impl From<ParseIntError> for KernelVersionParseError {
    fn from(err: ParseIntError) -> Self {
        KernelVersionParseError::ParseInt(err)
    }
}

impl FromStr for KernelVersion {
    type Err = KernelVersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.splitn(2, '-').collect::<Vec<_>>();

        let version_str = parts[0];
        let extraversion = if parts.len() > 1 {
            Some(parts[1].to_owned())
        } else {
            None
        };

        let version_str = if let Some(idx) = version_str.find('+') {
            &version_str[..idx]
        } else {
            version_str
        };

        let mut version_items = version_str.split('.');

        let major = version_items
            .next()
            .ok_or(KernelVersionParseError::InvalidFormat)?
            .parse()?;

        let minor = version_items
            .next()
            .ok_or(KernelVersionParseError::InvalidFormat)?
            .parse()?;

        let patch = version_items
            .next()
            .ok_or(KernelVersionParseError::InvalidFormat)?
            .parse()?;

        let sublevel = if let Some(item) = version_items.next() {
            Some(item.parse()?)
        } else {
            None
        };

        // Fail if we have numbers after that
        if version_items.next().is_some() {
            return Err(KernelVersionParseError::InvalidFormat);
        }

        Ok(KernelVersion {
            major,
            minor,
            patch,
            sublevel,
            extraversion,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::KernelVersion;

    #[test]
    fn test_from_str() {
        // Fedora Kernel
        assert_eq!(
            KernelVersion::from_str("6.14.6-300.fc42.x86_64").unwrap(),
            KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: None,
                extraversion: Some(String::from("300.fc42.x86_64"))
            }
        );

        // Ubuntu Kernel
        assert_eq!(
            KernelVersion::from_str("4.10.0-28-generic").unwrap(),
            KernelVersion {
                major: 4,
                minor: 10,
                patch: 0,
                sublevel: None,
                extraversion: Some(String::from("28-generic"))
            }
        );

        // Random STB running 2.6.29
        assert_eq!(
            KernelVersion::from_str("2.6.29.6-22-sigma").unwrap(),
            KernelVersion {
                major: 2,
                minor: 6,
                patch: 29,
                sublevel: Some(6),
                extraversion: Some(String::from("22-sigma"))
            }
        );

        // Custom Built Kernel, with modifications
        assert_eq!(
            KernelVersion::from_str("6.15.0+").unwrap(),
            KernelVersion {
                major: 6,
                minor: 15,
                patch: 0,
                sublevel: None,
                extraversion: None,
            }
        );
    }

    #[test]
    fn test_ord() {
        assert_eq!(
            KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: Some(12),
                extraversion: Some(String::from("300.fc42.x86_64")),
            },
            KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: Some(12),
                extraversion: Some(String::from("300.fc42.x86_64")),
            },
        );

        assert!(
            KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: Some(12),
                extraversion: Some(String::from("300.fc42.x86_64")),
            } > KernelVersion {
                major: 5,
                minor: 14,
                patch: 6,
                sublevel: Some(12),
                extraversion: Some(String::from("300.fc42.x86_64")),
            },
        );

        assert!(
            KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: Some(12),
                extraversion: Some(String::from("300.fc42.x86_64")),
            } > KernelVersion {
                major: 6,
                minor: 13,
                patch: 6,
                sublevel: Some(12),
                extraversion: Some(String::from("300.fc42.x86_64")),
            },
        );

        assert!(
            KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: Some(12),
                extraversion: Some(String::from("300.fc42.x86_64")),
            } > KernelVersion {
                major: 6,
                minor: 14,
                patch: 5,
                sublevel: Some(12),
                extraversion: Some(String::from("300.fc42.x86_64")),
            },
        );

        assert!(
            KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: Some(12),
                extraversion: Some(String::from("300.fc42.x86_64")),
            } > KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: Some(11),
                extraversion: Some(String::from("300.fc42.x86_64")),
            },
        );

        assert!(
            KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: Some(12),
                extraversion: Some(String::from("300.fc42.x86_64")),
            } > KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: None,
                extraversion: Some(String::from("300.fc42.x86_64")),
            },
        );

        assert_eq!(
            KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: Some(12),
                extraversion: Some(String::from("300.fc42.x86_64")),
            },
            KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: Some(12),
                extraversion: Some(String::from("200.fc42.x86_64")),
            },
        );

        assert_eq!(
            KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: Some(12),
                extraversion: Some(String::from("300.fc42.x86_64")),
            },
            KernelVersion {
                major: 6,
                minor: 14,
                patch: 6,
                sublevel: Some(12),
                extraversion: None,
            },
        );
    }
}
