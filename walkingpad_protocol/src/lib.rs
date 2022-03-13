/*!
    Structs and functions for implementing the Walkingpad A1 Pro protocol.

    The Walkingpad communicates over Bluetooth Low Energy, so a library like btleplug may be
    used in conjunction with this one to control and query the pad.
*/

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

/// Represents the speed values used in requests and responses.
/// The Walkingpad displays speeds in kilometers per second, but stores them internally in
/// hectometers (100 meters) per seconds to represent fractional values.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Speed(u8);

impl Speed {
    pub const fn from_km_per_hour(value: u8) -> Result<Speed> {
        Speed::from_hm_per_hour(value * 10)
    }

    pub const fn from_hm_per_hour(value: u8) -> Result<Speed> {
        if value <= 60 {
            Ok(Speed(value))
        } else {
            Err(ProtocolError::InvalidSpeed(value))
        }
    }

    pub const fn hm_per_hour(&self) -> u8 {
        self.0
    }
}

impl Default for Speed {
    fn default() -> Self {
        // This is the default speed when the device is first turned on from the factory.
        Self(20)
    }
}

/// Defines the subjects which can be queried or set on the Walkingpad.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
enum Subject {
    State = 0xa2,
    Settings = 0xa6,
    StoredStats = 0xa7,
}

impl TryFrom<u8> for Subject {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self> {
        Self::from_repr(value).ok_or(ProtocolError::InvalidType(value, "subject"))
    }
}

/// Defines the operational modes the Walkingpad can be in.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
pub enum Mode {
    /// In the Automatic mode, the Walkingpad will automatically adjust the belt speed to keep the
    /// user roughly within the center.
    Auto = 0,

    /// In the Manual mode, the Walkingpad works as expected, with all speed adjustments happening
    /// through either the remote or a Bluetooth command.
    Manual = 1,

    Sleep = 2,

    /// In the Calibration mode, the Walkingpad simply runs continuously at a speed of 4 km/h.
    Calibration = 4,
}

impl TryFrom<u8> for Mode {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self> {
        Self::from_repr(value).ok_or(ProtocolError::InvalidType(value, "mode"))
    }
}

/// Defines the sensitivy levels for the Walkingpad's Automatic mode.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
pub enum Sensitivity {
    High = 1,
    Medium = 2,
    Low = 3,
}

impl TryFrom<u8> for Sensitivity {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self> {
        Self::from_repr(value).ok_or(ProtocolError::InvalidType(value, "sensitivity"))
    }
}

/// Defines the units of measure used by the display on the Walkingpad.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
pub enum Units {
    Metric = 0,
    Imperial = 1,
}

impl TryFrom<u8> for Units {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self> {
        Self::from_repr(value).ok_or(ProtocolError::InvalidType(value, "units"))
    }
}

bitflags! {
    /// Defines the kinds of statistics the Walkingpad will cycle through on its display.
    ///
    /// ```rust
    /// use crate::Request;
    ///
    /// use InfoFlags::*;
    /// let request = Request.set().display(TIME | SPEED);
    /// ```
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
