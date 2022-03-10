use super::*;

use std::convert::{TryFrom, TryInto};
use std::mem::size_of;

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
    ($int_type:ty as bool, $bytes:ident) => {
        $bytes
            .try_into()
            .map(<$int_type>::from_be_bytes)
            .map_err(|_| ProtocolError::ResponseTooShort)
            .map(|val| val != 0)
            .map(|val| (val, &$bytes[size_of::<$int_type>()..]))
    };
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
        let bytes = Response::parse_header(bytes)?;
        let (response_type, bytes) = parse!(u8, ResponseType, bytes)?;

        let (response, bytes) = match response_type {
            ResponseType::CurrentRun => Response::parse_current_run(bytes)?,
            ResponseType::Settings => Response::parse_settings(bytes)?,
            ResponseType::PreviousRun => (Response::PreviousRuns, bytes),
        };

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

    fn parse_settings(bytes: &[u8]) -> Result<(Response, &[u8])> {
        let (goal_type, bytes) = parse!(u8, bytes)?;
        let (goal, bytes) = parse!(u8, bytes)?;
        let (calibration, bytes) = parse!(u8, bytes)?;
        let (max_speed, bytes) = parse!(u8, Speed, bytes)?;
        let (start_speed, bytes) = parse!(u8, Speed, bytes)?;
        let (start_mode, bytes) = parse!(u8, Mode, bytes)?;
        let (sensitivity, bytes) = parse!(u8, Sensitivity, bytes)?;
        let (display, bytes) = parse!(u8, InfoFlags, bytes)?;
        let (is_locked, bytes) = parse!(u8 as bool, bytes)?;
        let (units, bytes) = parse!(u8, Units, bytes)?;

        let settings = Response::Settings {
            goal_type,
            goal,
            calibration,
            max_speed,
            start_speed,
            start_mode,
            sensitivity,
            display,
            is_locked,
            units,
        };

        Ok((settings, bytes))
    }

    fn parse_header(bytes: &[u8]) -> Result<&[u8]> {
        let (header, bytes) = parse!(u8, bytes)?;

        if header == MESSAGE_HEADER {
            Ok(bytes)
        } else {
            Err(ProtocolError::InvalidResponseHeader(header))
        }
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
