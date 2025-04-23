#![doc = include_str!("../README.md")]
#![no_std]

use core::ops::Deref;

// re-exports
pub use datetime::*;
pub use macros::*;
pub use toml::value::{Date, Datetime, Offset, Time};

/// An array referencing a `'static` slice of type `T`.
#[derive(Clone, Copy, Debug)]
pub struct Array<T: 'static>(pub &'static [T]);

/// An empty value. Empty toml arrays contain this type.
#[derive(Clone, Copy, Debug)]
pub struct Empty;

impl<T: 'static + Copy> Deref for crate::Array<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Destructured datetime structs
mod datetime {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
    pub struct OffsetDateTime {
        pub date: Date,
        pub time: Time,
        pub offset: Offset,
    }

    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
    pub struct LocalDateTime {
        pub date: Date,
        pub time: Time,
    }

    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
    pub struct LocalDate {
        pub date: Date,
    }

    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
    pub struct LocalTime {
        pub time: Time,
    }

    impl From<OffsetDateTime> for Datetime {
        fn from(value: OffsetDateTime) -> Self {
            Self {
                date: Some(value.date),
                time: Some(value.time),
                offset: Some(value.offset),
            }
        }
    }

    impl From<LocalDateTime> for Datetime {
        fn from(value: LocalDateTime) -> Self {
            Self {
                date: Some(value.date),
                time: Some(value.time),
                offset: None,
            }
        }
    }

    impl From<LocalDate> for Datetime {
        fn from(value: LocalDate) -> Self {
            Self {
                date: Some(value.date),
                time: None,
                offset: None,
            }
        }
    }

    impl From<LocalTime> for Datetime {
        fn from(value: LocalTime) -> Self {
            Self {
                date: None,
                time: Some(value.time),
                offset: None,
            }
        }
    }

    impl core::fmt::Display for OffsetDateTime {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let as_dt = Datetime::from(*self);
            write!(f, "{}", as_dt)
        }
    }

    impl core::fmt::Display for LocalDateTime {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let as_dt = Datetime::from(*self);
            write!(f, "{}", as_dt)
        }
    }

    impl core::fmt::Display for LocalDate {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let as_dt = Datetime::from(*self);
            write!(f, "{}", as_dt)
        }
    }

    impl core::fmt::Display for LocalTime {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let as_dt = Datetime::from(*self);
            write!(f, "{}", as_dt)
        }
    }
}
