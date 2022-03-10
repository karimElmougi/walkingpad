use std::convert::{TryFrom, TryInto};
use std::mem::size_of;

use super::*;

macro_rules! parse {
    ($int_type:ty, $bytes:ident) => {
        $bytes
            .try_into()
            .map(<$int_type>::from_be_bytes)
            .map_err(|_| ProtocolError::ResponseTooShort)
            .map(|val| (val, &$bytes[size_of::<$int_type>()..]))
    };
    ($int_type:ty, $to_type:ty, $bytes:ident) => {
        $bytes
            .try_into()
            .map(<$int_type>::from_be_bytes)
            .map_err(|_| ProtocolError::ResponseTooShort)
            .and_then(|val| <$to_type>::try_from(val))
            .map(|val| (val, &$bytes[size_of::<$int_type>()..]))
    };
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
enum ResponseType {
    CurrentRun = 0xa2,
    Settings = 0xa6,
    PreviousRun = 0xa7,
}

impl TryFrom<u8> for ResponseType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self> {
        use ResponseType::*;

        const VARIANTS: [ResponseType; 3] = [CurrentRun, Settings, PreviousRun];
        VARIANTS
            .iter()
            .copied()
            .find(|&variant| variant as u8 == value)
            .ok_or(ProtocolError::InvalidResponseType(value))
    }
}

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
        lock: bool,
        unit: Unit,
    },
    PreviousRuns,
}

impl Response {
    pub fn parse(bytes: &[u8]) -> Result<Response> {
        let bytes = Response::parse_header(bytes)?;
        let (response_type, bytes) = Response::parse_response_type(bytes)?;

        let response = Response::PreviousRuns;
        // match response_type {
        //     ResponseType::CurrentRun => Response::parse_current_run(bytes)?,
        //     ResponseType::Settings => (),
        //     ResponseType::PreviousRun => (),
        // }

        Response::parse_footer(bytes)?;

        Ok(response)
    }

    fn parse_current_run(bytes: &[u8]) -> Result<(Response, &[u8])> {
        let (state, bytes) = parse!(u8, bytes)?;
        let (speed, bytes) = parse!(u8, Speed, bytes)?;
        let (mode, bytes) = parse!(u8, bytes)?;
        let (time, bytes) = parse!(u32, bytes)?;
        let (distance, bytes) = parse!(u32, bytes)?;
        let (steps, bytes) = parse!(u32, bytes)?;

        let current_run_stats = Response::CurrentRunStats {
            state,
            speed,
            mode,
            time,
            distance,
            steps,
        };

        Ok((current_run_stats, bytes))
    }

    fn parse_header(bytes: &[u8]) -> Result<&[u8]> {
        let (header, bytes) = parse!(u8, bytes)?;

        if header == MESSAGE_HEADER {
            Ok(bytes)
        } else {
            Err(ProtocolError::InvalidResponseHeader(header))
        }
    }

    fn parse_response_type(bytes: &[u8]) -> Result<(ResponseType, &[u8])> {
        let (val, bytes) = parse!(u8, bytes)?;

        Ok((val.try_into()?, bytes))
    }

    fn parse_footer(bytes: &[u8]) -> Result<()> {
        let (footer, bytes) = parse!(u8, bytes)?;

        if footer == MESSAGE_FOOTER {
            if !bytes.is_empty() {
                Err(ProtocolError::BytesAfterFooter)
            } else {
                Ok(())
            }
        } else {
            Err(ProtocolError::InvalidResponseFooter(footer))
        }
    }
}
