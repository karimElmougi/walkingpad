use core::fmt::{Debug, Display, Formatter};
use core::time::Duration;

use super::{Error, InfoFlags, Mode, Result, Sensitivity, Speed, Subject, Units, MESSAGE_FOOTER};

/// Defines the state the WalkingPad's motor can be in.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct State {
    /// The state of the motor.
    pub motor_state: MotorState,

    /// The current speed.
    pub speed: Speed,

    /// The current operational mode.
    pub mode: Mode,

    /// Time in seconds of the current run on the WalkingPad's internal clock.
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    pub run_time: Duration,

    /// The distance traveled during the current run, in meters.
    pub distance: u32,

    /// The number of steps counted so far.
    pub nb_steps: u32,

    /// Bytes whose meaning is undetermined.
    /// The third byte appears to correspond to button presses from the remote.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub unknown: [u8; 4],
}

impl State {
    fn parse(reader: &mut impl Iterator<Item = u8>) -> Result<State> {
        Ok(State {
            motor_state: read_u8(reader)?.into(),
            speed: read_u8(reader).and_then(Speed::try_from_hm_per_hour)?,
            mode: read_u8(reader)?.try_into()?,
            run_time: Duration::from_secs(read_u32(reader)?.into()),
            distance: decameter_to_meter(read_u32(reader)?),
            nb_steps: read_u32(reader)?,
            unknown: [
                read_u8(reader)?,
                read_u8(reader)?,
                read_u8(reader)?,
                read_u8(reader)?,
            ],
        })
    }
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "State {{ ")?;
        write!(f, "motor_state: {:?}, ", self.motor_state)?;
        write!(f, "speed: {}, ", self.speed)?;
        write!(f, "mode: {:?}, ", self.mode)?;
        write!(f, "distance: {}m, ", self.distance)?;
        write!(f, "run_time: {:?}, ", self.run_time)?;
        write!(f, "nb_steps: {} ", self.nb_steps)?;
        write!(f, "}}")
    }
}

/// Represents the settings stored on the WalkingPad.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Settings {
    /// The significance of this field is unclear.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub goal_type: u8, // TODO: What even is this?

    /// The significance of this field is unclear.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub goal: u32, // TODO: What even is this?

    /// This field may represent whether the WalkingPad is in calibration mode.
    #[cfg_attr(feature = "serde", serde(skip))]
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
    #[cfg_attr(feature = "serde", serde(skip))]
    pub is_locked: bool, // TODO: Need to confirm what this actually does

    /// The units of measurement used on the WalkingPad's display.
    pub units: Units,

    /// Bytes whose meaning is undetermined.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub unknown: [u8; 4], // TODO: Figure out what those are
}

impl Settings {
    fn parse(reader: &mut impl Iterator<Item = u8>) -> Result<Settings> {
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

impl Display for Settings {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Settings {{ ")?;
        write!(f, "max_speed: {}, ", self.max_speed)?;
        write!(f, "start_speed: {}, ", self.start_speed)?;
        write!(f, "start_mode: {:?}, ", self.start_mode)?;
        write!(f, "sensitivity: {:?}, ", self.sensitivity)?;
        write!(f, "display: {:?}, ", self.display)?;
        write!(f, "units: {:?} ", self.units)?;
        write!(f, "}}")
    }
}

/// Represents the statistics of a past run stored on the device.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StoredStats {
    /// The current time on the WalkingPad's internal clock.
    /// It only ticks while the belt is running and starts at 0 on first boot.
    /// Seems essentially useless as a result.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub current_time: u32,

    /// The start time of this run on the internal clock.
    /// Suffers from a similar issue to [current_time], unclear good this is for other than for
    /// sorting.
    pub start_time: u32,

    /// The duration of the run.
    #[cfg_attr(feature = "serde", serde(with = "humantime_serde"))]
    pub duration: Duration,

    /// The distance traveled during the run, in meters.
    pub distance: u32,

    /// The number of steps recorded during the run.
    pub nb_steps: u32,

    /// The id of the next record.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub next_id: Option<u8>,
}

impl StoredStats {
    fn parse(reader: &mut impl Iterator<Item = u8>) -> Result<StoredStats> {
        Ok(StoredStats {
            current_time: read_u32(reader)?,
            start_time: read_u32(reader)?,
            duration: Duration::from_secs(read_u32(reader)?.into()),
            distance: decameter_to_meter(read_u32(reader)?),
            nb_steps: read_u32(reader)?,
            next_id: read_u8(reader).map(|n| if n == 0 { None } else { Some(n) })?,
        })
    }
}

impl Display for StoredStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "StoredStats {{ ")?;
        write!(f, "start_time: {}, ", self.start_time)?;
        write!(f, "duration: {:?}, ", self.duration)?;
        write!(f, "distance: {}m, ", self.distance)?;
        write!(f, "nb_steps: {:?}, ", self.nb_steps)?;
        write!(f, "}}")
    }
}

/// Defines the types of responses that can be received from the WalkingPad.
#[derive(Clone, Eq, PartialEq, PartialOrd)]
pub enum Response {
    State(State),
    Settings(Settings),
    StoredStats(StoredStats),
}

impl From<State> for Response {
    fn from(state: State) -> Response {
        Response::State(state)
    }
}

impl From<Settings> for Response {
    fn from(settings: Settings) -> Response {
        Response::Settings(settings)
    }
}

impl From<StoredStats> for Response {
    fn from(stored_stats: StoredStats) -> Response {
        Response::StoredStats(stored_stats)
    }
}

impl Debug for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Response::State(inner) => Debug::fmt(inner, f),
            Response::Settings(inner) => Debug::fmt(inner, f),
            Response::StoredStats(inner) => Debug::fmt(inner, f),
        }
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Response::State(inner) => Display::fmt(inner, f),
            Response::Settings(inner) => Display::fmt(inner, f),
            Response::StoredStats(inner) => Display::fmt(inner, f),
        }
    }
}

impl Response {
    pub fn parse(bytes: &[u8]) -> Result<Response> {
        let mut it = bytes.iter().copied();

        Response::parse_header(&mut it)?;

        let subject = read_u8(&mut it)?.try_into()?;
        let response = match subject {
            Subject::State => State::parse(&mut it)?.into(),
            Subject::Settings => Settings::parse(&mut it)?.into(),
            Subject::StoredStats => StoredStats::parse(&mut it)?.into(),
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
            .then_some(())
            .ok_or(Error::InvalidResponseHeader(byte))
    }

    fn parse_footer(reader: &mut impl Iterator<Item = u8>) -> Result<()> {
        let byte = read_u8(reader)?;

        (byte == MESSAGE_FOOTER)
            .then_some(())
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

fn decameter_to_meter(n: u32) -> u32 {
    n * 10
}
