use super::*;

use std::convert::{TryFrom, TryInto};
use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};
use strum_macros::FromRepr;

pub enum Response {
    CurrentRunStats {
        state: u8,
        speed: Speed,
        mode: u8,
        time: u32,
        distance: u32,
        steps: u32,
    },
    Settings {
        goal_type: u8,
        goal: u8,
        calibration: u8,
        max_speed: Speed,
        start_speed: Speed,
        start_mode: Mode,
        sensitivity: Sensitivity,
        display: InfoFlags,
        is_locked: bool,
        units: Units,
    },
    PreviousRuns,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
enum ResponseType {
    CurrentRun = 0xa2,
    Settings = 0xa6,
    PreviousRun = 0xa7,
}

impl_try_from!(u8, ResponseType);

impl Response {
    pub fn parse(bytes: &[u8]) -> Result<Response> {
        let mut bytes = Cursor::new(bytes);

        Response::parse_header(&mut bytes)?;
        let response_type = bytes.read_u8()?.try_into()?;

        let response = match response_type {
            ResponseType::CurrentRun => Response::parse_current_run(&mut bytes)?,
            ResponseType::Settings => Response::parse_settings(&mut bytes)?,
            ResponseType::PreviousRun => Response::PreviousRuns,
        };

        Response::parse_footer(&mut bytes)?;

        Ok(response)
    }

    fn parse_current_run(reader: &mut impl ReadBytesExt) -> Result<Response> {
        Ok(Response::CurrentRunStats {
            state: reader.read_u8()?,
            speed: reader.read_u8()?.try_into()?,
            mode: reader.read_u8()?,
            time: reader.read_u32::<BigEndian>()?,
            distance: reader.read_u32::<BigEndian>()?,
            steps: reader.read_u32::<BigEndian>()?,
        })
    }

    fn parse_settings(reader: &mut impl ReadBytesExt) -> Result<Response> {
        Ok(Response::Settings {
            goal_type: reader.read_u8()?,
            goal: reader.read_u8()?,
            calibration: reader.read_u8()?,
            max_speed: reader.read_u8()?.try_into()?,
            start_speed: reader.read_u8()?.try_into()?,
            start_mode: reader.read_u8()?.try_into()?,
            sensitivity: reader.read_u8()?.try_into()?,
            display: reader.read_u8()?.try_into()?,
            is_locked: reader.read_u8()? != 0,
            units: reader.read_u8()?.try_into()?,
        })
    }

    fn parse_header(reader: &mut impl ReadBytesExt) -> Result<()> {
        let byte = reader.read_u8()?;

        (byte == MESSAGE_HEADER)
            .then(|| ())
            .ok_or(ProtocolError::InvalidResponseHeader(byte))
    }

    fn parse_footer(reader: &mut impl ReadBytesExt) -> Result<()> {
        let byte = reader.read_u8()?;

        (byte == MESSAGE_FOOTER)
            .then(|| ())
            .ok_or(ProtocolError::InvalidResponseHeader(byte))
    }
}
