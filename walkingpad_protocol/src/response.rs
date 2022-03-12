use super::*;

use std::convert::TryInto;
use std::fmt::{Debug, Formatter};
use std::io::Cursor;

use byteorder::ReadBytesExt;

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
    fn parse(reader: &mut impl ReadBytesExt) -> Result<State> {
        Ok(State {
            state: reader.read_u8()?.into(),
            speed: reader.read_u8()?.try_into()?,
            mode: reader.read_u8()?.try_into()?,
            time: read_u32(reader)?,
            distance: read_u32(reader)?,
            steps: read_u32(reader)?,
            unknown: reader.read_u32::<byteorder::BigEndian>()?.to_be_bytes(),
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
    fn parse(reader: &mut impl ReadBytesExt) -> Result<Settings> {
        Ok(Settings {
            goal_type: reader.read_u8()?,
            goal: read_u32(reader)?,
            calibration: reader.read_u8()?,
            max_speed: reader.read_u8()?.try_into()?,
            start_speed: reader.read_u8()?.try_into()?,
            start_mode: reader.read_u8()?.try_into()?,
            sensitivity: reader.read_u8()?.try_into()?,
            display: reader.read_u8()?.try_into()?,
            is_locked: reader.read_u8()? != 0,
            units: reader.read_u8()?.try_into()?,
            unknown: reader.read_u32::<byteorder::BigEndian>()?.to_be_bytes(),
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
    fn parse(reader: &mut impl ReadBytesExt) -> Result<StoredStats> {
        Ok(StoredStats {
            time: read_u32(reader)?,
            start_time: read_u32(reader)?,
            duration: read_u32(reader)?,
            distance: read_u32(reader)?,
            nb_steps: read_u32(reader)?,
            next_id: reader
                .read_u8()
                .map(|n| if n == 0 { None } else { Some(n) })?,
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Response::State(inner) => inner.fmt(f),
            Response::Settings(inner) => inner.fmt(f),
            Response::StoredStats(inner) => inner.fmt(f),
        }
    }
}

impl Response {
    pub fn parse(bytes: &[u8]) -> Result<Response> {
        let mut bytes = Cursor::new(bytes);

        Response::parse_header(&mut bytes)?;

        let subject = bytes.read_u8()?.try_into()?;
        let response = match subject {
            Subject::State => State::parse(&mut bytes)?.into(),
            Subject::Settings => Settings::parse(&mut bytes)?.into(),
            Subject::StoredStats => StoredStats::parse(&mut bytes)?.into(),
        };

        let _crc = bytes.read_u8()?;

        Response::parse_footer(&mut bytes)?;

        Ok(response)
    }

    fn parse_header(reader: &mut impl ReadBytesExt) -> Result<()> {
        let byte = reader.read_u8()?;

        const RESPONSE_HEADER: u8 = 0xf8;

        (byte == RESPONSE_HEADER)
            .then(|| ())
            .ok_or(ProtocolError::InvalidResponseHeader(byte))
    }

    fn parse_footer(reader: &mut impl ReadBytesExt) -> Result<()> {
        let byte = reader.read_u8()?;

        (byte == MESSAGE_FOOTER)
            .then(|| ())
            .ok_or(ProtocolError::InvalidResponseFooter(byte))
    }
}

/// Because the Wakling Pad uses 3-byte long integer counters
fn read_u32(reader: &mut impl ReadBytesExt) -> Result<u32> {
    Ok(u32::from_be_bytes([
        0,
        reader.read_u8()?,
        reader.read_u8()?,
        reader.read_u8()?,
    ]))
}
