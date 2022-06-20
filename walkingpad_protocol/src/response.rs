use super::*;

/// Responses kept mostly as-is from the WalkpingPad, as opposed to the cleaned
/// up versions found in the rest of the [response] module.
pub mod raw;

use core::fmt::{Debug, Display, Formatter};

use measurements::Distance;

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

    /// The distance currently traveled.
    #[cfg_attr(feature = "serde", serde(with = "serde_distance"))]
    pub distance: Distance,

    /// Time in seconds of the current run on the WalkingPad's internal clock.
    pub run_time: u32,

    /// The number of steps counted so far.
    pub nb_steps: u32,
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "State {{ ")?;
        write!(f, "motor_state: {:?}, ", self.motor_state)?;
        write!(f, "speed: {}, ", self.speed)?;
        write!(f, "mode: {:?}, ", self.mode)?;
        write!(f, "distance: {}, ", self.distance)?;
        write!(f, "current_time: {}, ", self.run_time)?;
        write!(f, "nb_steps: {} ", self.nb_steps)?;
        write!(f, "}}")
    }
}

impl From<raw::State> for State {
    fn from(raw: raw::State) -> State {
        State {
            motor_state: raw.motor_state,
            speed: raw.speed,
            mode: raw.mode,
            distance: raw.distance(),
            run_time: raw.current_time,
            nb_steps: raw.nb_steps,
        }
    }
}

/// Represents the settings stored on the WalkingPad.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Settings {
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

    /// The units of measurement used on the WalkingPad's display.
    pub units: Units,
}

impl Display for Settings {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

impl From<raw::Settings> for Settings {
    fn from(raw: raw::Settings) -> Settings {
        Settings {
            max_speed: raw.max_speed,
            start_speed: raw.start_speed,
            start_mode: raw.start_mode,
            sensitivity: raw.sensitivity,
            display: raw.display,
            units: raw.units,
        }
    }
}

/// Represents the statistics of a past run stored on the device.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StoredStats {
    /// The duration of the run.
    #[cfg_attr(feature = "serde", serde(with = "serde_duration"))]
    pub duration: core::time::Duration,

    /// The distance traveled during the run, in decimeters (10 meters).
    #[cfg_attr(feature = "serde", serde(with = "serde_distance"))]
    pub distance: Distance,

    /// The number of steps recorded during the run.
    pub nb_steps: u32,

    #[cfg_attr(feature = "serde", serde(skip))]
    /// The id of the next record.
    pub next_id: Option<u8>,
}

impl Display for StoredStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "StoredStats {{ ")?;
        write!(f, "duration: {:?}, ", self.duration)?;
        write!(f, "distance: {}, ", self.distance)?;
        write!(f, "nb_steps: {:?}, ", self.nb_steps)?;
        write!(f, "}}")
    }
}

impl From<raw::StoredStats> for StoredStats {
    fn from(raw: raw::StoredStats) -> StoredStats {
        StoredStats {
            duration: raw.duration(),
            distance: raw.distance(),
            nb_steps: raw.nb_steps,
            next_id: raw.next_id,
        }
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

impl From<raw::State> for Response {
    fn from(state: raw::State) -> Response {
        Response::State(state.into())
    }
}

impl From<raw::Settings> for Response {
    fn from(settings: raw::Settings) -> Response {
        Response::Settings(settings.into())
    }
}

impl From<raw::StoredStats> for Response {
    fn from(stored_stats: raw::StoredStats) -> Self {
        Response::StoredStats(stored_stats.into())
    }
}

impl From<raw::Response> for Response {
    fn from(raw: raw::Response) -> Response {
        match raw {
            raw::Response::State(s) => s.into(),
            raw::Response::Settings(s) => s.into(),
            raw::Response::StoredStats(s) => s.into(),
        }
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

#[cfg(feature = "serde")]
mod serde_distance {
    use measurements::Distance;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(distance: &Distance, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(distance.as_meters() as u32)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Distance, D::Error>
    where
        D: Deserializer<'de>,
    {
        let distance: u32 = Deserialize::deserialize(deserializer)?;
        Ok(Distance::from_meters(distance as f64))
    }
}

#[cfg(feature = "serde")]
mod serde_duration {
    use arrayvec::ArrayString;
    use serde::{de, ser};
    use serde::{Deserialize, Deserializer, Serializer};

    use core::fmt::Write;
    use core::time::Duration;

    pub fn serialize<S>(dur: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use ser::Error;

        const MINUTE: Duration = Duration::from_secs(60);
        const HOUR: Duration = Duration::from_secs(60 * 60);

        let mut buffer = ArrayString::<64>::new();
        if dur > &HOUR {
            let hours = dur.as_secs() / HOUR.as_secs();
            let minutes = (dur.as_secs() % HOUR.as_secs()) / 60;

            core::write!(buffer, "{}h{}m", hours, minutes).map_err(S::Error::custom)?;
        } else if dur > &MINUTE {
            core::write!(buffer, "{}m", dur.as_secs() / 60).map_err(S::Error::custom)?;
        } else {
            core::write!(buffer, "{}s", dur.as_secs()).map_err(S::Error::custom)?;
        }

        serializer.serialize_str(buffer.as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        use de::Error;

        let dur: &str = Deserialize::deserialize(deserializer)?;
        parse_duration::parse(dur).map_err(D::Error::custom)
    }
}
