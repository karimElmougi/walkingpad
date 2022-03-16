use super::*;

use core::convert::TryInto;
use core::fmt::{Debug, Formatter};

/// Defines the state the WalkingPad's motor can be in.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub enum MotorState {
    Stopped,
    Running,
    Starting,
    Unknown(u8),
}

impl From<u8> for MotorState {
    fn from(value: u8) -> Self {
        use MotorState::*;

        match value {
            0b0000 => Stopped,
            0b0001 => Running,
            0b1001 => Starting,
            _ => Unknown(value),
        }
    }
}

/// Reprents the current state of the WalkingPad.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub struct State {
    /// The state of the motor.
    pub state: MotorState,

    /// The current speed.
    pub speed: Speed,

    /// The current operational mode.
    pub mode: Mode,

    /// The current time on the WalkingPad's internal clock.
    pub time: u32,

    /// The distance currently traveled.
    pub distance: u32,

    /// The number of steps counted so far.
    pub steps: u32,

    /// Bytes whose meaning is undetermined.
    /// The third byte appears to correspond to button presses from the remote.
    pub unknown: [u8; 4],
}

impl State {
    fn deserialize(reader: &mut impl Iterator<Item = u8>) -> Result<State> {
        Ok(State {
            state: read_u8(reader)?.into(),
            speed: read_u8(reader).and_then(Speed::try_from_hm_per_hour)?,
            mode: read_u8(reader)?.try_into()?,
            time: read_u32(reader)?,
            distance: read_u32(reader)?,
            steps: read_u32(reader)?,
            unknown: [
                read_u8(reader)?,
                read_u8(reader)?,
                read_u8(reader)?,
                read_u8(reader)?,
            ],
        })
    }
}

/// Represents the settings stored on the WalkingPad.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub struct Settings {
    /// The significance of this field is unclear.
    pub goal_type: u8, // TODO: What even is this?

    /// The significance of this field is unclear.
    pub goal: u32, // TODO: What even is this?

    /// This field may represent whether the WalkingPad is in calibration mode.
    pub calibration: u8, // TODO: is this a boolean, or something else?

    /// The maxmimum speed the WalkingPad can be set to.
    pub max_speed: Speed,

    /// The speed the WalkingPad starts any run at.
    pub start_speed: Speed,

    /// The mode the WalkingPad boots up into.
    pub start_mode: Mode,

    /// The default sensitivity of the Automatic mode.
    pub sensitivity: Sensitivity,

    /// The currently displayed statistics on the WalkingPad's on-board display.
    pub display: InfoFlags,

    /// Whether the WalkingPad's state is locked.
    pub is_locked: bool, // TODO: Need to confirm what this actually does

    /// The units of measurement used on the WalkingPad's display.
    pub units: Units,

    /// Bytes whose meaning is undetermined.
    pub unknown: [u8; 4], // TODO: Figure out what those are
}

impl Settings {
    fn deserialize(reader: &mut impl Iterator<Item = u8>) -> Result<Settings> {
        Ok(Settings {
            goal_type: read_u8(reader)?,
            goal: read_u32(reader)?,
            calibration: read_u8(reader)?,
            max_speed: read_u8(reader).and_then(Speed::try_from_hm_per_hour)?,
            start_speed: read_u8(reader).and_then(Speed::try_from_hm_per_hour)?,
            start_mode: read_u8(reader)?.try_into()?,
            sensitivity: read_u8(reader)?.try_into()?,
            display: read_u8(reader)?.try_into()?,
            is_locked: read_u8(reader)? != 0,
            units: read_u8(reader)?.try_into()?,
            unknown: [
                read_u8(reader)?,
                read_u8(reader)?,
                read_u8(reader)?,
                read_u8(reader)?,
            ],
        })
    }
}

/// Represents the records of statistics of past runs stored on the device.
/// These records effectively form a linked list through the `next_id` field, with the id 255
/// representing the latest record.
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub struct StoredStats {
    /// The current time on the WalkingPad's internal clock.
    /// It only tick while the belt is running and starts at 0 on first boot.
    pub time: u32,

    /// The start time of this run on the internal clock.
    pub start_time: u32,

    /// The duration of the run.
    pub duration: u32,

    /// The distance traveled during the run, in decimeters (10 meters).
    pub distance: u32,

    /// The number of steps recorded during the run.
    pub nb_steps: u32,

    /// The id of the next record.
    pub next_id: Option<u8>,
}

impl StoredStats {
    fn deserialize(reader: &mut impl Iterator<Item = u8>) -> Result<StoredStats> {
        Ok(StoredStats {
            time: read_u32(reader)?,
            start_time: read_u32(reader)?,
            duration: read_u32(reader)?,
            distance: read_u32(reader)?,
            nb_steps: read_u32(reader)?,
            next_id: read_u8(reader).map(|n| if n == 0 { None } else { Some(n) })?,
        })
    }
}

/// Defines the types of responses that can be received from the WalkingPad.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd)]
pub enum Response {
    State(State),
    Settings(Settings),
    StoredStats(StoredStats),
}

impl From<State> for Response {
    fn from(state: State) -> Self {
        Response::State(state)
    }
}

impl From<Settings> for Response {
    fn from(settings: Settings) -> Self {
        Response::Settings(settings)
    }
}

impl From<StoredStats> for Response {
    fn from(stored_stats: StoredStats) -> Self {
        Response::StoredStats(stored_stats)
    }
}

impl Debug for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Response::State(inner) => inner.fmt(f),
            Response::Settings(inner) => inner.fmt(f),
            Response::StoredStats(inner) => inner.fmt(f),
        }
    }
}

impl Response {
    pub fn deserialize(bytes: &[u8]) -> Result<Response> {
        let mut it = bytes.iter().copied();

        Response::parse_header(&mut it)?;

        let subject = read_u8(&mut it)?.try_into()?;
        let response = match subject {
            Subject::State => State::deserialize(&mut it)?.into(),
            Subject::Settings => Settings::deserialize(&mut it)?.into(),
            Subject::StoredStats => StoredStats::deserialize(&mut it)?.into(),
        };

        let _crc = read_u8(&mut it)?;

        Response::parse_footer(&mut it)?;
        if it.next().is_none() {
            Ok(response)
        } else {
            Err(Error::BytesAfterFooter)
        }
    }

    fn parse_header(reader: &mut impl Iterator<Item = u8>) -> Result<()> {
        let byte = read_u8(reader)?;

        const RESPONSE_HEADER: u8 = 0xf8;

        (byte == RESPONSE_HEADER)
            .then(|| ())
            .ok_or(Error::InvalidResponseHeader(byte))
    }

    fn parse_footer(reader: &mut impl Iterator<Item = u8>) -> Result<()> {
        let byte = read_u8(reader)?;

        (byte == MESSAGE_FOOTER)
            .then(|| ())
            .ok_or(Error::InvalidResponseFooter(byte))
    }
}

fn read_u32(reader: &mut impl Iterator<Item = u8>) -> Result<u32> {
    // Because the Wakling Pad uses 3-bytes long integer counters in respones
    Ok(u32::from_be_bytes([
        0,
        read_u8(reader)?,
        read_u8(reader)?,
        read_u8(reader)?,
    ]))
}

fn read_u8(reader: &mut impl Iterator<Item = u8>) -> Result<u8> {
    reader.next().ok_or(Error::ResponseTooShort)
}
