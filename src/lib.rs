use std::convert::TryFrom;

use bitflags::bitflags;
use thiserror::Error;
use uuid::Uuid;

pub mod request;
pub mod response;

type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("{0} hm/h is greater than the maximum supported speed of 60 hm/h")]
    InvalidSpeed(u8),

    #[error("{0} isn't a valid response type")]
    InvalidResponseType(u8),

    #[error("{0} isn't a valid response header")]
    InvalidResponseHeader(u8),

    #[error("{0} isn't a valid response footer")]
    InvalidResponseFooter(u8),

    #[error("the response continues past footer")]
    BytesAfterFooter,

    #[error("the response length doesn't match any known response type or is missing bytes")]
    ResponseTooShort,
}

const MESSAGE_HEADER: u8 = 0xf7;
const MESSAGE_FOOTER: u8 = 0xfd;

// Stole this from the btleplug crate
const BLUETOOTH_BASE_UUID: u128 = 0x00000000_0000_1000_8000_00805f9b34fb;

pub const TREADMILL_CHARACTERISTIC_UUID: Uuid =
    Uuid::from_u128(BLUETOOTH_BASE_UUID | ((0xfe02) << 96));

pub const TREADMILL_READ_CHARACTERISTIC_UUID: Uuid =
    Uuid::from_u128(BLUETOOTH_BASE_UUID | ((0xfe01) << 96));

pub const WALKINGPAD_SERVICE_UUID: Uuid =
    Uuid::from_u128(0xfe00 << 96 | 0x1000 << 64 | 0x8000 << 48 | 0x805f9b34fb);

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

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Mode {
    Auto = 0,
    Manual = 1,
    Sleep = 2,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Sensitivity {
    High = 1,
    Medium = 2,
    Low = 3,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Unit {
    Metric = 0,
    Imperial = 1,
}

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
