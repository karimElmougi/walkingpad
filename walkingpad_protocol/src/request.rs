use super::*;

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
