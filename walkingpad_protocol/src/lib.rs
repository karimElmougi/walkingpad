/*!
    Structs and functions for implementing the WalkingPad A1 Pro protocol.

    The WalkingPad communicates over Bluetooth Low Energy, so a library like btleplug may be
    used in conjunction with this one to control and query the pad.
*/

#![no_std]

pub mod request;
pub mod response;

pub use response::Response;

use core::convert::TryFrom;
use core::fmt;
use core::ops;

use bitflags::bitflags;
use strum_macros::FromRepr;

#[macro_use]
extern crate impl_ops;

const MESSAGE_FOOTER: u8 = 0xfd;

type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidSpeed(u8),
    InvalidType(u8, &'static str),
    InvalidResponseHeader(u8),
    InvalidResponseFooter(u8),
    BytesAfterFooter,
    ResponseTooShort,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            InvalidSpeed(speed) => write!(
                f,
                "{} hm/h is greater than the maximum supported speed of 60 hm/h",
                speed
            ),
            InvalidType(byte, typename) => write!(f, "{} isn't a valid {} type", byte, typename),
            InvalidResponseHeader(byte) => write!(f, "{} isn't a valid response header", byte),
            InvalidResponseFooter(byte) => write!(f, "{} isn't a valid response footer", byte),
            BytesAfterFooter => write!(f, "the response continues past footer"),
            ResponseTooShort => write!(f, "the response is missing bytes"),
        }
    }
}

/// Represents the speed values used in requests and responses.
/// The WalkingPad displays speeds in kilometers per second, but stores them internally in
/// hectometers (100 meters) per seconds to represent fractional values.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd)]
pub struct Speed {
    inner: u8,
}

impl Speed {
    const MAX: u8 = 60;

    /// Clamps to the highest speed of 6 km/h.
    pub const fn from_km_per_hour(value: u8) -> Speed {
        match Speed::try_from_km_per_hour(value) {
            Ok(speed) => speed,
            Err(_) => Speed {
                inner: Speed::MAX / 10,
            },
        }
    }

    pub const fn try_from_km_per_hour(value: u8) -> Result<Speed> {
        Speed::try_from_hm_per_hour(value * 10)
    }

    /// Clamps to the highest speed of 60 hm/h.
    pub const fn from_hm_per_hour(value: u8) -> Speed {
        match Speed::try_from_hm_per_hour(value) {
            Ok(speed) => speed,
            Err(_) => Speed { inner: Speed::MAX },
        }
    }

    pub const fn try_from_hm_per_hour(value: u8) -> Result<Speed> {
        if value <= Speed::MAX {
            Ok(Speed { inner: value })
        } else {
            Err(Error::InvalidSpeed(value))
        }
    }

    pub const fn hm_per_hour(self) -> u8 {
        self.inner
    }

    /// Does an integer division of the inner hectometer value.
    pub const fn km_per_hour(self) -> u8 {
        self.hm_per_hour() / 10
    }
}

impl Default for Speed {
    fn default() -> Self {
        // This is the default speed when the device is first turned on from the factory.
        Self { inner: 20 }
    }
}

impl_op_ex!(+ |a: &Speed, b: &u8| -> Speed { Speed::from_hm_per_hour(a.inner + b) });
impl_op_ex!(+ |a: &Speed, b: &Speed| -> Speed { ops::Add::add(a, b.inner) });

impl<T> ops::Add<T> for &mut Speed
where
    Speed: ops::Add<T>,
{
    type Output = <Speed as ops::Add<T>>::Output;

    fn add(self, rhs: T) -> Self::Output {
        ops::Add::add(*self, rhs)
    }
}

impl<T> ops::Add<&mut T> for &'_ Speed
where
    T: Copy,
    Speed: ops::Add<T>,
{
    type Output = <Speed as ops::Add<T>>::Output;

    fn add(self, rhs: &mut T) -> Self::Output {
        ops::Add::add(*self, *rhs)
    }
}

impl<T> ops::Add<&mut T> for Speed
where
    T: Copy,
    Self: ops::Add<T>,
{
    type Output = <Speed as ops::Add<T>>::Output;

    fn add(self, rhs: &mut T) -> Self::Output {
        ops::Add::add(self, *rhs)
    }
}

impl<T> ops::AddAssign<T> for Speed
where
    Speed: ops::Add<T, Output = Speed>,
{
    fn add_assign(&mut self, rhs: T) {
        *self = ops::Add::add(*self, rhs);
    }
}

impl_op_ex!(-|a: &Speed, b: &u8| -> Speed {
    Speed {
        inner: a.inner.saturating_sub(*b),
    }
});
impl_op_ex!(-|a: &Speed, b: &Speed| -> Speed { ops::Sub::sub(a, b.inner) });

impl<T> ops::Sub<T> for &mut Speed
where
    Speed: ops::Sub<T>,
{
    type Output = <Speed as ops::Sub<T>>::Output;

    fn sub(self, rhs: T) -> Self::Output {
        ops::Sub::sub(*self, rhs)
    }
}

impl<T> ops::Sub<&mut T> for &'_ Speed
where
    T: Copy,
    Speed: ops::Sub<T>,
{
    type Output = <Speed as ops::Sub<T>>::Output;

    fn sub(self, rhs: &mut T) -> Self::Output {
        ops::Sub::sub(*self, *rhs)
    }
}

impl<T> ops::Sub<&mut T> for Speed
where
    T: Copy,
    Self: ops::Sub<T>,
{
    type Output = <Speed as ops::Sub<T>>::Output;

    fn sub(self, rhs: &mut T) -> Self::Output {
        ops::Sub::sub(self, *rhs)
    }
}

impl<T> ops::SubAssign<T> for Speed
where
    Speed: ops::Sub<T, Output = Speed>,
{
    fn sub_assign(&mut self, rhs: T) {
        *self = ops::Sub::sub(*self, rhs);
    }
}

/// Defines the subjects which can be queried or set on the WalkingPad.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
enum Subject {
    State = 0xa2,
    Settings = 0xa6,
    StoredStats = 0xa7,
}

impl TryFrom<u8> for Subject {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Self::from_repr(value).ok_or(Error::InvalidType(value, "subject"))
    }
}

/// Defines the operational modes the WalkingPad can be in.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
pub enum Mode {
    /// In the Automatic mode, the WalkingPad will automatically adjust the belt speed to keep the
    /// user roughly within the center.
    Auto = 0,

    /// In the Manual mode, the WalkingPad works as expected, with all speed adjustments happening
    /// through either the remote or a Bluetooth command.
    Manual = 1,

    Sleep = 2,

    /// In the Calibration mode, the WalkingPad simply runs continuously at a speed of 4 km/h.
    Calibration = 4,
}

impl TryFrom<u8> for Mode {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Self::from_repr(value).ok_or(Error::InvalidType(value, "mode"))
    }
}

/// Defines the sensitivy levels for the WalkingPad's Automatic mode.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
pub enum Sensitivity {
    High = 1,
    Medium = 2,
    Low = 3,
}

impl TryFrom<u8> for Sensitivity {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Self::from_repr(value).ok_or(Error::InvalidType(value, "sensitivity"))
    }
}

/// Defines the units of measure used by the display on the WalkingPad.
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, FromRepr)]
pub enum Units {
    Metric = 0,
    Imperial = 1,
}

impl TryFrom<u8> for Units {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Self::from_repr(value).ok_or(Error::InvalidType(value, "units"))
    }
}

bitflags! {
    /// Defines the kinds of statistics the WalkingPad will cycle through on its display.
    ///
    /// ```rust
    /// use walkingpad_protocol::Request;
    /// use walkingpad_protocol::InfoFlags;
    ///
    /// let request = Request::set().display(InfoFlags::TIME | InfoFlags::SPEED);
    /// ```
    pub struct InfoFlags: u8 {
        const NONE = 0b0;
        const TIME = 0b1;
        const SPEED = 0b10;
        const DISTANCE = 0b100;
        const CALORIE = 0b1000;
        const STEP = 0b10000;
    }
}

impl TryFrom<u8> for InfoFlags {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        Self::from_bits(value).ok_or(Error::InvalidType(value, "InfoFlags"))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut a = Speed::from_hm_per_hour(10);
        let mut b = Speed::from_hm_per_hour(55);
        assert_eq!(0, (a - b).hm_per_hour());
        assert_eq!(45, (b - a).hm_per_hour());
        assert_eq!(60, (b + a).hm_per_hour());
        assert_eq!(60, (a + b).hm_per_hour());

        assert_eq!(60, (&mut a + &mut b).hm_per_hour());
        assert_eq!(60, (&mut a + &b).hm_per_hour());
        assert_eq!(60, (&a + &mut b).hm_per_hour());
        assert_eq!(60, (&a + &b).hm_per_hour());
        assert_eq!(60, (&a + b).hm_per_hour());
        assert_eq!(60, (a + &b).hm_per_hour());
        assert_eq!(60, (a + b).hm_per_hour());
        assert_eq!(60, (&mut a + b).hm_per_hour());
        assert_eq!(60, (a + &mut b).hm_per_hour());

        assert_eq!(0, (&mut a - &mut b).hm_per_hour());
        assert_eq!(0, (&mut a - &b).hm_per_hour());
        assert_eq!(0, (&a - &mut b).hm_per_hour());
        assert_eq!(0, (&a - &b).hm_per_hour());
        assert_eq!(0, (&a - b).hm_per_hour());
        assert_eq!(0, (a - &b).hm_per_hour());
        assert_eq!(0, (a - b).hm_per_hour());
        assert_eq!(0, (&mut a - b).hm_per_hour());
        assert_eq!(0, (a - &mut b).hm_per_hour());

        a += 5;
        assert_eq!(15, a.hm_per_hour());
        a += Speed::default();
        assert_eq!(35, a.hm_per_hour());

        b -= 5;
        assert_eq!(50, b.hm_per_hour());
        b -= Speed::default();
        assert_eq!(30, b.hm_per_hour());
    }
}
