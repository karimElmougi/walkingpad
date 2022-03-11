use super::*;

use std::convert::{TryFrom, TryInto};
use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};
use strum_macros::FromRepr;

pub enum Response {
    CurrentRunLiveStats {
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
    StoredRunStats {
        time: u32,
        start_time: u32,
        duration: u32,
        distance: u32,
        nb_steps: u32,
        nb_remaining: u8,
    },
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
enum ResponseType {
    CurrentRunLiveStats = 0xa2,
    Settings = 0xa6,
    StoredRun = 0xa7,
}

impl_try_from!(u8, ResponseType);

impl Response {
    pub fn parse(bytes: &[u8]) -> Result<Response> {
        let mut bytes = Cursor::new(bytes);

        Response::parse_header(&mut bytes)?;
        let response_type = bytes.read_u8()?.try_into()?;

        let response = match response_type {
            ResponseType::CurrentRunLiveStats => Response::parse_current_run(&mut bytes)?,
            ResponseType::Settings => Response::parse_settings(&mut bytes)?,
            ResponseType::StoredRun => Response::parse_stored_run(&mut bytes)?,
        };

        Response::parse_footer(&mut bytes)?;

        Ok(response)
    }

    fn parse_current_run(reader: &mut impl ReadBytesExt) -> Result<Response> {
        Ok(Response::CurrentRunLiveStats {
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

    fn parse_stored_run(reader: &mut impl ReadBytesExt) -> Result<Response> {
        Ok(Response::StoredRunStats {
            time: reader.read_u32::<BigEndian>()?,
            start_time: reader.read_u32::<BigEndian>()?,
            duration: reader.read_u32::<BigEndian>()?,
            distance: reader.read_u32::<BigEndian>()?,
            nb_steps: reader.read_u32::<BigEndian>()?,
            nb_remaining: reader.read_u8()?,
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
