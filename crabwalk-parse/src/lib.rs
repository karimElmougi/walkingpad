use walkingpad_protocol::request;
use walkingpad_protocol::{Mode, Request, Speed};

use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    MissingArgument,
    InvalidArgument(String, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::MissingArgument => write!(f, "Missing arg"),
            Error::InvalidArgument(arg, kind) => write!(f, "{} is not a valid {}", arg, kind),
        }
    }
}

impl std::error::Error for Error {}

pub fn parse(input: &str) -> Result<Request> {
    let mut tokens = input.trim().split_whitespace();

    match tokens.next() {
        Some("get") => get(&mut tokens),
        Some("set") => set(&mut tokens),
        Some("start") => Ok(request::start()),
        Some("stop") => Ok(request::stop()),
        Some(cmd) => Err(Error::InvalidArgument(
            cmd.to_string(),
            "command".to_string(),
        )),
        None => Err(Error::MissingArgument),
    }
}

fn get<'a>(tokens: &mut impl Iterator<Item = &'a str>) -> Result<Request> {
    match tokens.next() {
        Some("state") => Ok(request::get::state()),
        Some("settings") => Ok(request::get::settings()),
        Some(_) => Err(Error::MissingArgument),
        None => Err(Error::MissingArgument),
    }
}

fn set<'a>(tokens: &mut impl Iterator<Item = &'a str>) -> Result<Request> {
    match tokens.next() {
        Some("speed") => Ok(request::set::speed(parse_speed(tokens)?)),
        Some("max-speed") => Ok(request::set::max_speed(parse_speed(tokens)?)),
        Some("start-speed") => Ok(request::set::start_speed(parse_speed(tokens)?)),
        Some("calibration") => Ok(request::set::calibration_mode(parse_bool(tokens)?)),
        Some("auto-start") => Ok(request::set::auto_start(parse_bool(tokens)?)),
        Some("mode") => Ok(request::set::mode(parse_mode(tokens)?)),
        Some(_) => Err(Error::MissingArgument),
        None => Err(Error::MissingArgument),
    }
}

fn parse_speed<'a>(tokens: &mut impl Iterator<Item = &'a str>) -> Result<Speed> {
    let val_input = tokens.next().ok_or(Error::MissingArgument)?;

    str::parse::<f64>(val_input)
        .map(|speed| (speed * 10.0).round() as u8)
        .map(Speed::from_hm_per_hour)
        .map_err(|_| Error::InvalidArgument(val_input.to_string(), "speed".to_string()))
}

fn parse_bool<'a>(tokens: &mut impl Iterator<Item = &'a str>) -> Result<bool> {
    let val_input = tokens.next().ok_or(Error::MissingArgument)?;

    str::parse::<bool>(val_input)
        .map_err(|_| Error::InvalidArgument(val_input.to_string(), "bool".to_string()))
}

fn parse_mode<'a>(tokens: &mut impl Iterator<Item = &'a str>) -> Result<Mode> {
    match tokens.next() {
        Some("auto") => Ok(Mode::Auto),
        Some("manual") => Ok(Mode::Manual),
        Some("sleep") => Ok(Mode::Sleep),
        Some("calibration") => Ok(Mode::Calibration),
        Some(other) => Err(Error::InvalidArgument(
            other.to_string(),
            "mode".to_string(),
        )),
        None => Err(Error::MissingArgument),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let input = "set speed 7";
        assert_eq!(
            parse(input).unwrap(),
            request::set::speed(Speed::from_hm_per_hour(60))
        );
    }
}
