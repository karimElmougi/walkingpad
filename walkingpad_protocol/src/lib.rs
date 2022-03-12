pub mod request;
pub mod response;

use std::convert::TryFrom;

use bitflags::bitflags;
use strum_macros::FromRepr;
use thiserror::Error;

const MESSAGE_FOOTER: u8 = 0xfd;

type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("{0} hm/h is greater than the maximum supported speed of 60 hm/h")]
    InvalidSpeed(u8),

    #[error("{0} isn't a valid {1} type")]
    InvalidType(u8, &'static str),

    #[error("{0} isn't a valid response header")]
    InvalidResponseHeader(u8),

    #[error("{0} isn't a valid response footer")]
    InvalidResponseFooter(u8),

    #[error("the response continues past footer")]
    BytesAfterFooter,

    #[error("the response length doesn't match any known response type or is missing bytes")]
    ResponseTooShort,
}

impl From<std::io::Error> for ProtocolError {
    fn from(_: std::io::Error) -> Self {
        ProtocolError::ResponseTooShort
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Speed(u8);

impl Speed {
    // An hectometer is 100 meters, or 0.1 kilometers
    fn hm_per_hour(&self) -> u8 {
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
