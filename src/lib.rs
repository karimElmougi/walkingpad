use std::convert::TryFrom;

use uuid::Uuid;

// Stole this from the btleplug crate
const BLUETOOTH_BASE_UUID: u128 = 0x00000000_0000_1000_8000_00805f9b34fb;

pub const TREADMILL_CHARACTERISTIC_UUID: Uuid = Uuid::from_u128(BLUETOOTH_BASE_UUID | ((0xfe02) << 96));
pub const WALKINGPAD_SERVICE_UUID: Uuid = Uuid::from_u128(0xfe00 << 96 | 0x1000 << 64 | 0x8000 << 48 | 0x805f9b34fb);

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Speed(u8);

impl Speed {
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
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= 60 {
            Ok(Speed(value))
        } else {
            Err("".to_string())
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

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Info {
    Time = 0b1,
    Speed = 0b10,
    Distance = 0b100,
    Calorie = 0b1000,
    Step = 0b10000,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum Command {
    Query,
    SetSpeed(Speed),
    SetMode(Mode),
    SetCalibrationMode(bool),
    SetMaxSpeed(Speed),
    Start, // This actually acts as a toggle
    Stop,
    SetStartSpeed(Speed),
    SetAutoStart(bool),
    SetSensitivity(Sensitivity),
    SetDisplayInfo(Info),
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
            SetCalibrationMode(_) => 2, // Not a typo, they really share the same code
            SetMaxSpeed(_) => 3,
            Start => 4,
            Stop => 4,
            SetStartSpeed(_) => 4, // Not a typo, they really share the same code
            SetAutoStart(_) => 5,
            SetSensitivity(_) => 6,
            SetDisplayInfo(_) => 7,
            SetUnit(_) => 8,
            SetLock(_) => 9,
        }
    }

    fn type_byte(&self) -> u8 {
        use Command::*;

        // This is my best guess as to what that particular byte in the command header represents,
        // but I truly have no idea
        const COMMAND_TYPE: u8 = 0xa2;
        const SETTINGS_TYPE: u8 = 0xa6;

        match self {
            Query => COMMAND_TYPE,
            SetSpeed(_) => COMMAND_TYPE,
            SetMode(_) => COMMAND_TYPE,
            SetCalibrationMode(_) => SETTINGS_TYPE,
            SetMaxSpeed(_) => SETTINGS_TYPE,
            Start => COMMAND_TYPE,
            Stop => COMMAND_TYPE,
            SetStartSpeed(_) => SETTINGS_TYPE,
            SetAutoStart(_) => SETTINGS_TYPE,
            SetSensitivity(_) => SETTINGS_TYPE,
            SetDisplayInfo(_) => SETTINGS_TYPE,
            SetUnit(_) => SETTINGS_TYPE,
            SetLock(_) => SETTINGS_TYPE,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        use Command::*;

        let mut param = match self {
            Query => unimplemented!(),
            SetSpeed(speed) => vec![speed.hm_per_hour()],
            SetMode(mode) => vec![*mode as u8],
            Start => vec![1],
            Stop => vec![0],

            SetCalibrationMode(enabled) => to_bytes(*enabled as u32),
            SetStartSpeed(speed) => to_bytes(speed.hm_per_hour() as u32),
            SetMaxSpeed(speed) => to_bytes(speed.hm_per_hour() as u32),
            SetSensitivity(sensitivity) => to_bytes(*sensitivity as u32),
            SetAutoStart(enabled) => to_bytes(*enabled as u32),
            SetDisplayInfo(info) => to_bytes(*info as u32),
            SetUnit(unit) => to_bytes(*unit as u32),
            SetLock(enabled) => to_bytes(*enabled as u32),
        };

        // 0xf7 is ostensibly some sort of header value
        let mut bytes = vec![0xf7, self.type_byte(), self.code()];

        bytes.append(&mut param);

        // A simplistic CRC
        bytes.push(bytes.iter().skip(1).sum());

        // Ostensibly a footer
        bytes.push(0xfd);

        bytes
    }
}

fn to_bytes(val: u32) -> Vec<u8> {
    val.to_be_bytes().to_vec()
}

