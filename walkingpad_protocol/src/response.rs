use super::*;

use std::convert::TryInto;
use std::fmt::{Debug, Formatter};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub struct State {
    pub state: MotorState,
    pub speed: Speed,
    pub mode: Mode,
    pub time: u32,
    pub distance: u32,
    pub steps: u32,
    pub unknown: [u8; 4],
}

impl State {
    fn parse(reader: &mut impl Iterator<Item = u8>) -> Result<State> {
        Ok(State {
            state: read_u8(reader)?.into(),
            speed: read_u8(reader)?.try_into()?,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub struct Settings {
    pub goal_type: u8,
    pub goal: u32,
    pub calibration: u8,
    pub max_speed: Speed,
    pub start_speed: Speed,
    pub start_mode: Mode,
    pub sensitivity: Sensitivity,
    pub display: InfoFlags,
    pub is_locked: bool,
    pub units: Units,
    pub unknown: [u8; 4],
}

impl Settings {
    fn parse(reader: &mut impl Iterator<Item = u8>) -> Result<Settings> {
        Ok(Settings {
            goal_type: read_u8(reader)?,
            goal: read_u32(reader)?,
            calibration: read_u8(reader)?,
            max_speed: read_u8(reader)?.try_into()?,
            start_speed: read_u8(reader)?.try_into()?,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
pub struct StoredStats {
    pub time: u32,
    pub start_time: u32,
    pub duration: u32,
    pub distance: u32,
    pub nb_steps: u32,
    pub next_id: Option<u8>,
}

impl StoredStats {
    fn parse(reader: &mut impl Iterator<Item = u8>) -> Result<StoredStats> {
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

        Ok(response)
    }

    fn parse_header(reader: &mut impl Iterator<Item = u8>) -> Result<()> {
        let byte = read_u8(reader)?;

        const RESPONSE_HEADER: u8 = 0xf8;

        (byte == RESPONSE_HEADER)
            .then(|| ())
            .ok_or(ProtocolError::InvalidResponseHeader(byte))
    }

    fn parse_footer(reader: &mut impl Iterator<Item = u8>) -> Result<()> {
        let byte = read_u8(reader)?;

        (byte == MESSAGE_FOOTER)
            .then(|| ())
            .ok_or(ProtocolError::InvalidResponseFooter(byte))
    }
}

/// Because the Wakling Pad uses 3-bytes long integer counters
fn read_u32(reader: &mut impl Iterator<Item = u8>) -> Result<u32> {
    Ok(u32::from_be_bytes([
        0,
        read_u8(reader)?,
        read_u8(reader)?,
        read_u8(reader)?,
    ]))
}

fn read_u8(reader: &mut impl Iterator<Item = u8>) -> Result<u8> {
    reader.next().ok_or(ProtocolError::ResponseTooShort)
}
