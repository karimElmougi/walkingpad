use super::*;

pub mod raw;

use core::fmt::{Debug, Display, Formatter};

use measurements::Distance;

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
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct State {
    /// The state of the motor.
    pub motor_state: MotorState,

    /// The current speed.
    pub speed: Speed,

    /// The current operational mode.
    pub mode: Mode,

    /// The distance currently traveled.
    pub distance: Distance,

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
            nb_steps: raw.nb_steps,
        }
    }
}

/// Represents the settings stored on the WalkingPad.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
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

#[cfg(not(feature = "std"))]
pub use raw::StoredStats;

/// Represents the statistics of a past run stored on the device.
#[cfg(feature = "std")]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct StoredStats {
    /// The start time of the run.
    pub start_time: std::time::SystemTime,

    /// The duration of the run.
    pub duration: Duration,

    /// The distance traveled during the run, in decimeters (10 meters).
    pub distance: Distance,

    /// The number of steps recorded during the run.
    pub nb_steps: u32,
}

#[cfg(feature = "std")]
impl Display for StoredStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let start_time = time::OffsetDateTime::from(self.start_time())
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();

        write!(f, "StoredStats {{ ")?;
        write!(f, "start_time: {}, ", start_time)?;
        write!(f, "duration: {:?}, ", self.duration)?;
        write!(f, "distance: {}, ", self.distance)?;
        write!(f, "nb_steps: {:?}, ", self.nb_steps)?;
        write!(f, "}}")
    }
}

#[cfg(feature = "std")]
impl From<raw::StoredStats> for StoredStats {
    fn from(raw: raw::StoredStats) -> StoredStats {
        let elapsed = (raw.current_time - raw.start_time) as u64;
        let elapsed = std::time::Duration::from_secs(elapsed);
        let start_time = std::time::SystemTime::now() - elapsed;

        StoredStats {
            start_time,
            duration: raw.duration(),
            distance: raw.distance(),
            nb_steps: raw.nb_steps,
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

#[cfg(feature = "std")]
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

            #[cfg(feature = "std")]
            Response::StoredStats(inner) => Display::fmt(inner, f),

            #[cfg(not(feature = "std"))]
            Response::StoredStats(inner) => Debug::fmt(inner, f),
        }
    }
}
