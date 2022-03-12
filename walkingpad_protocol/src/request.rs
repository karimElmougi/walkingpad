use super::*;

const REQUEST_HEADER: u8 = 0xf7;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Request {
    GetState,
    GetSettings,
    GetStoredStats,
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
    SetUnit(Units),
    SetLock(bool),
}

#[repr(u8)]
enum Subject {
    State = 0xa2,
    Settings = 0xa6,
    StoredStats = 0xa7,
}

impl Request {
    fn code(&self) -> u8 {
        use Request::*;

        match self {
            GetState => 0,
            GetSettings => 0,
            GetStoredStats => 0xaa,
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

    fn subject(&self) -> Subject {
        use Request::*;

        match self {
            GetState => Subject::State,
            GetSettings => Subject::Settings,
            GetStoredStats => Subject::StoredStats,
            SetSpeed(_) => Subject::State,
            SetMode(_) => Subject::State,
            SetCalibrationMode(_) => Subject::Settings,
            SetMaxSpeed(_) => Subject::Settings,
            Start => Subject::State,
            Stop => Subject::State,
            SetStartSpeed(_) => Subject::Settings,
            SetAutoStart(_) => Subject::Settings,
            SetSensitivity(_) => Subject::Settings,
            SetDisplayInfo(_) => Subject::Settings,
            SetUnit(_) => Subject::Settings,
            SetLock(_) => Subject::Settings,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        use Request::*;

        let mut param = match self {
            GetState => vec![0],
            GetSettings => to_bytes(0),
            GetStoredStats => vec![1],
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

        let mut bytes = vec![REQUEST_HEADER, self.subject() as u8, self.code()];

        bytes.append(&mut param);

        bytes.push(compute_crc(&bytes[1..]));

        bytes.push(MESSAGE_FOOTER);

        bytes
    }
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
