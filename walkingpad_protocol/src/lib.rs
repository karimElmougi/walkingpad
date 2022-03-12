#![no_std]

pub mod request;
pub mod response;

pub use request::Request;
pub use response::Response;

use core::convert::TryFrom;
use core::fmt;

use bitflags::bitflags;
use strum_macros::FromRepr;

const MESSAGE_FOOTER: u8 = 0xfd;

type Result<T> = core::result::Result<T, ProtocolError>;

#[derive(Debug)]
pub enum ProtocolError {
    InvalidSpeed(u8),
    InvalidType(u8, &'static str),
    InvalidResponseHeader(u8),
    InvalidResponseFooter(u8),
    BytesAfterFooter,
    ResponseTooShort,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ProtocolError::*;
        match self {
            InvalidSpeed(speed) => write!(
                f,
                "{} hm/h is greater than the maximum supported speed of 60 hm/h",
                speed
            ),
            InvalidType(byte, typename) => write!(f, "{} isn't a valid {} type", byte, typename),
            InvalidResponseHeader(byte) => write!(f, "{} isn't a valid response header", byte),
            InvalidResponseFooter(byte) => write!(f, "{} isn't a valid response footer", byte),
            BytesAfterFooter => write!(f, "the response continues past footer"),
            ResponseTooShort => write!(f, "the response is missing bytes"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Speed(u8);

impl Speed {
    // An hectometer is 100 meters, or 0.1 kilometers
    const fn hm_per_hour(&self) -> u8 {
        self.0
    }
}

impl Default for Speed {
    fn default() -> Self {
        Self(20)
    }
}

impl TryFrom<u8> for Speed {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self> {
        if value <= 60 {
            Ok(Speed(value))
        } else {
            Err(ProtocolError::InvalidSpeed(value))
        }
    }
}

macro_rules! impl_try_from {
    ($int:ty, $enum:ty) => {
        impl TryFrom<$int> for $enum {
            type Error = ProtocolError;

            fn try_from(value: $int) -> Result<Self> {
                Self::from_repr(value).ok_or(ProtocolError::InvalidType(value, stringify!($enum)))
            }
        }
    };
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
enum Subject {
    State = 0xa2,
    Settings = 0xa6,
    StoredStats = 0xa7,
}

impl_try_from!(u8, Subject);

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
pub enum Mode {
    Auto = 0,
    Manual = 1,
    Sleep = 2,
    Calibration = 4,
}

impl_try_from!(u8, Mode);

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
pub enum Sensitivity {
    High = 1,
    Medium = 2,
    Low = 3,
}

impl_try_from!(u8, Sensitivity);

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
pub enum Units {
    Metric = 0,
    Imperial = 1,
}

impl_try_from!(u8, Units);

bitflags! {
    pub struct InfoFlags: u8 {
        const NONE = 0b0;
        const TIME = 0b1;
        const SPEED = 0b10;
        const DISTANCE = 0b100;
        const CALORIE = 0b1000;
        const STEP = 0b10000;
    }
}

impl TryFrom<u8> for InfoFlags {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self> {
        Self::from_bits(value).ok_or(ProtocolError::InvalidType(value, "InfoFlags"))
    }
}
