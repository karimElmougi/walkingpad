use std::convert::{TryFrom, TryInto};
use std::mem::size_of;

use bitflags::bitflags;
use thiserror::Error;
use uuid::Uuid;

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

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Command {
    // Asks for run statistics maybe? And its SetSettings counterpart asks for current settings?
    Query,
    SetSpeed(Speed),
    SetMode(Mode),
    SetCalibrationMode(bool),
    SetMaxSpeed(Speed),
    Start, // This actually acts as a toggle it seems
    Stop,
    SetStartSpeed(Speed),
    SetAutoStart(bool),
    SetSensitivity(Sensitivity),
    SetDisplayInfo(InfoFlags),
    SetUnit(Unit),
    SetLock(bool),
}

impl Command {
    fn code(&self) -> u8 {
        use Command::*;

        match self {
            Query => 0,
            SetSpeed(_) => 1,
            SetMode(_) => 2,
            SetCalibrationMode(_) => 2,
            SetMaxSpeed(_) => 3,
            Start => 4,
            Stop => 4,
            SetStartSpeed(_) => 4,
            SetAutoStart(_) => 5,
            SetSensitivity(_) => 6,
            SetDisplayInfo(_) => 7,
            SetUnit(_) => 8,
            SetLock(_) => 9,
        }
    }

    fn mode(&self) -> u8 {
        use Command::*;

        // This is my best guess as to what that particular byte in the command header represents,
        // but I truly have no idea
        #[repr(u8)]
        enum Mode {
            Command = 0xa2,
            SetSettings = 0xa6,

            #[allow(dead_code)]
            Unknown = 0xa7, // No idea what this one means, maybe sync?
        }

        let mode = match self {
            Query => Mode::Command,
            SetSpeed(_) => Mode::Command,
            SetMode(_) => Mode::Command,
            SetCalibrationMode(_) => Mode::SetSettings,
            SetMaxSpeed(_) => Mode::SetSettings,
            Start => Mode::Command,
            Stop => Mode::Command,
            SetStartSpeed(_) => Mode::SetSettings,
            SetAutoStart(_) => Mode::SetSettings,
            SetSensitivity(_) => Mode::SetSettings,
            SetDisplayInfo(_) => Mode::SetSettings,
            SetUnit(_) => Mode::SetSettings,
            SetLock(_) => Mode::SetSettings,
        };

        mode as u8
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        use Command::*;

        let mut param = match self {
            Query => vec![0],
            SetSpeed(speed) => vec![speed.hm_per_hour()],
            SetMode(mode) => vec![*mode as u8],
            Start => vec![1],
            Stop => vec![0],

            SetCalibrationMode(enabled) => to_bytes(*enabled as u32),
            SetStartSpeed(speed) => to_bytes(speed.hm_per_hour() as u32),
            SetMaxSpeed(speed) => to_bytes(speed.hm_per_hour() as u32),
            SetSensitivity(sensitivity) => to_bytes(*sensitivity as u32),
            SetAutoStart(enabled) => to_bytes(*enabled as u32),
            SetDisplayInfo(info) => to_bytes(info.bits() as u32),
            SetUnit(unit) => to_bytes(*unit as u32),
            SetLock(enabled) => to_bytes(*enabled as u32),
        };

        let mut bytes = vec![MESSAGE_HEADER, self.mode(), self.code()];

        bytes.append(&mut param);

        bytes.push(compute_crc(&bytes[1..]));

        bytes.push(MESSAGE_FOOTER);

        bytes
    }
}

macro_rules! parse {
    ($int_type:ty, $bytes:ident) => {
        $bytes
            .try_into()
            .map(<$int_type>::from_be_bytes)
            .map_err(|_| ProtocolError::ResponseTooShort)
            .map(|val| (val, &$bytes[size_of::<$int_type>()..]))
    };
    ($int_type:ty, $to_type:ty, $bytes:ident) => {
        $bytes
            .try_into()
            .map(<$int_type>::from_be_bytes)
            .map_err(|_| ProtocolError::ResponseTooShort)
            .and_then(|val| <$to_type>::try_from(val))
            .map(|val| (val, &$bytes[size_of::<$int_type>()..]))
    };
}

/// Computes the simplistic CRC checksum scheme of the message's contents.
/// The bytes must exclude any header or footer values.
fn compute_crc(message: &[u8]) -> u8 {
    message
        .iter()
        .copied()
        .fold(0, |crc, byte| crc.wrapping_add(byte))
}

fn to_bytes(val: u32) -> Vec<u8> {
    val.to_be_bytes().to_vec()
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
enum ResponseType {
    CurrentRun = 0xa2,
    Settings = 0xa6,
    PreviousRun = 0xa7,
}

impl TryFrom<u8> for ResponseType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self> {
        use ResponseType::*;

        const VARIANTS: [ResponseType; 3] = [CurrentRun, Settings, PreviousRun];
        VARIANTS
            .iter()
            .copied()
            .find(|&variant| variant as u8 == value)
            .ok_or(ProtocolError::InvalidResponseType(value))
    }
}

pub enum Response {
    CurrentRunStats {
        state: u8,
        speed: Speed,
        mode: u8,
        time: u32,
        distance: u32,
        steps: u32,
    },
    Settings {
        goal_type: u8,
        goal: u8,
        calibration: u8,
        max_speed: Speed,
        start_speed: Speed,
        start_mode: Mode,
        sensitivity: Sensitivity,
        display: InfoFlags,
        lock: bool,
        unit: Unit,
    },
    PreviousRuns,
}

impl Response {
    pub fn parse(bytes: &[u8]) -> Result<Response> {
        let bytes = Response::parse_header(bytes)?;
        let (response_type, bytes) = Response::parse_response_type(bytes)?;

        let response = Response::PreviousRuns;
        // match response_type {
        //     ResponseType::CurrentRun => Response::parse_current_run(bytes)?,
        //     ResponseType::Settings => (),
        //     ResponseType::PreviousRun => (),
        // }

        Response::parse_footer(bytes)?;

        Ok(response)
    }

    fn parse_current_run(bytes: &[u8]) -> Result<(Response, &[u8])> {
        let (state, bytes) = parse!(u8, bytes)?;
        let (speed, bytes) = parse!(u8, Speed, bytes)?;
        let (mode, bytes) = parse!(u8, bytes)?;
        let (time, bytes) = parse!(u32, bytes)?;
        let (distance, bytes) = parse!(u32, bytes)?;
        let (steps, bytes) = parse!(u32, bytes)?;

        let current_run_stats = Response::CurrentRunStats {
            state,
            speed,
            mode,
            time,
            distance,
            steps,
        };

        Ok((current_run_stats, bytes))
    }

    fn parse_header(bytes: &[u8]) -> Result<&[u8]> {
        let (header, bytes) = parse!(u8, bytes)?;

        if header == MESSAGE_HEADER {
            Ok(bytes)
        } else {
            Err(ProtocolError::InvalidResponseHeader(header))
        }
    }

    fn parse_response_type(bytes: &[u8]) -> Result<(ResponseType, &[u8])> {
        let (val, bytes) = parse!(u8, bytes)?;

        Ok((val.try_into()?, bytes))
    }

    fn parse_footer(bytes: &[u8]) -> Result<()> {
        let (footer, bytes) = parse!(u8, bytes)?;

        if footer == MESSAGE_FOOTER {
            if !bytes.is_empty() {
                Err(ProtocolError::BytesAfterFooter)
            } else {
                Ok(())
            }
        } else {
            Err(ProtocolError::InvalidResponseFooter(footer))
        }
    }
}

