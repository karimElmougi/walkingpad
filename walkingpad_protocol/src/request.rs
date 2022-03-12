use super::*;

const REQUEST_HEADER: u8 = 0xf7;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Command {
    QueryCurrentRunStats,
    QuerySettings,
    QueryStoredRuns,
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
enum RequestType {
    Command = 0xa2,
    SetSettings = 0xa6,

    Sync = 0xa7,
}

impl Command {
    fn code(&self) -> u8 {
        use Command::*;

        match self {
            QueryCurrentRunStats => 0,
            QuerySettings => 0,
            QueryStoredRuns => 0xaa,
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

    fn request_type(&self) -> u8 {
        use Command::*;

        // This is my best guess as to what that particular byte in the command header represents,
        // but I truly have no idea

        let mode = match self {
            QueryCurrentRunStats => RequestType::Command,
            QuerySettings => RequestType::SetSettings,
            QueryStoredRuns => RequestType::Sync,
            SetSpeed(_) => RequestType::Command,
            SetMode(_) => RequestType::Command,
            SetCalibrationMode(_) => RequestType::SetSettings,
            SetMaxSpeed(_) => RequestType::SetSettings,
            Start => RequestType::Command,
            Stop => RequestType::Command,
            SetStartSpeed(_) => RequestType::SetSettings,
            SetAutoStart(_) => RequestType::SetSettings,
            SetSensitivity(_) => RequestType::SetSettings,
            SetDisplayInfo(_) => RequestType::SetSettings,
            SetUnit(_) => RequestType::SetSettings,
            SetLock(_) => RequestType::SetSettings,
        };

        mode as u8
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        use Command::*;

        let mut param = match self {
            QueryCurrentRunStats => vec![0],
            QuerySettings => to_bytes(0),
            QueryStoredRuns => vec![1],
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

        let mut bytes = vec![REQUEST_HEADER, self.request_type(), self.code()];

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
