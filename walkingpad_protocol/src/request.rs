/// Module for producing the bytes representing the different requests the WalkingPad accepts.
///
/// # Examples
///
/// ```rust
/// use walkingpad_protocol::{request, Speed};
///
/// let start_command = request::start();
/// let get_settings = request::get::settings();
/// let set_speed = request::set::speed(Speed::from_hm_per_hour(25));
/// ```
use num_traits::PrimInt;

use core::mem::size_of;

use super::*;

const REQUEST_HEADER: u8 = 0xf7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request(RequestVariant);

impl Request {
    pub fn as_bytes(&self) -> &[u8] {
        match &self.0 {
            RequestVariant::U8(req) => req.as_bytes(),
            RequestVariant::U32(req) => req.as_bytes(),
        }
    }
}

impl From<RawRequest<u8>> for Request {
    fn from(req: RawRequest<u8>) -> Request {
        Request(RequestVariant::U8(req))
    }
}

impl From<RawRequest<u32>> for Request {
    fn from(req: RawRequest<u32>) -> Request {
        Request(RequestVariant::U32(req))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RequestVariant {
    U8(RawRequest<u8>),
    U32(RawRequest<u32>),
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRequest<T> {
    header: u8,
    subject: u8,
    request_type: u8,
    param: T,
    crc: u8,
    footer: u8,
}

impl<T: PrimInt> RawRequest<T> {
    fn new(request_type: u8, subject: Subject, param: T) -> RawRequest<T> {
        let base_size = size_of::<RawRequest<()>>();

        assert_eq!(base_size, 5);
        assert_eq!(size_of::<RawRequest<T>>(), base_size + size_of::<T>());

        let param = param.to_be();
        let mut req = RawRequest {
            header: REQUEST_HEADER,
            subject: subject as u8,
            request_type,
            param,
            crc: 0,
            footer: MESSAGE_FOOTER,
        };

        unsafe {
            let mut crc = 0u8;

            let ptr = &req as *const RawRequest<T> as *const u8;
            let mut begin = ptr.add(1);
            let end = begin.add(2).add(size_of::<T>());

            while begin < end {
                crc = crc.wrapping_add(*begin);
                begin = begin.add(1);
            }

            req.crc = crc;
        }

        req
    }

    pub fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const RawRequest<T> as *const u8;
        unsafe { core::slice::from_raw_parts(ptr, size_of::<RawRequest<T>>()) }
    }
}

/// Clears all data associated with past runs stored on the WalkingPad.
pub fn cleat_stats() -> Request {
    RawRequest::new(0xaa, Subject::StoredStats, 0u8).into()
}

pub fn start() -> Request {
    RawRequest::new(4, Subject::State, true as u8).into()
}

pub fn stop() -> Request {
    RawRequest::new(4, Subject::State, false as u8).into()
}

pub mod get {
    use super::*;

    pub fn state() -> Request {
        RawRequest::new(0, Subject::State, 0u8).into()
    }

    /// Request for the WalkingPad's current settings.
    pub fn settings() -> Request {
        RawRequest::new(0, Subject::Settings, 0u32).into()
    }

    /// Request for retrieving the stored run stats associated with the most recent run.
    pub fn latest_stored_stats() -> Request {
        const LATEST_STATS: u8 = 255;
        RawRequest::new(0xaa, Subject::StoredStats, LATEST_STATS).into()
    }

    /// Request for retrieving the stored run stats associated with a specific ID.
    pub fn stored_stats(id: u8) -> Request {
        RawRequest::new(0xaa, Subject::StoredStats, id).into()
    }
}

pub mod set {
    use super::*;

    pub fn speed(speed: Speed) -> Request {
        RawRequest::new(1, Subject::State, speed.hm_per_hour()).into()
    }

    pub fn mode(mode: Mode) -> Request {
        RawRequest::new(2, Subject::State, mode as u8).into()
    }

    pub fn calibration_mode(enabled: bool) -> Request {
        RawRequest::new(2, Subject::Settings, enabled as u32).into()
    }

    pub fn max_speed(speed: Speed) -> Request {
        RawRequest::new(3, Subject::Settings, speed.hm_per_hour() as u32).into()
    }

    pub fn start_speed(speed: Speed) -> Request {
        RawRequest::new(4, Subject::Settings, speed.hm_per_hour() as u32).into()
    }

    pub fn auto_start(enabled: bool) -> Request {
        RawRequest::new(5, Subject::Settings, enabled as u32).into()
    }

    pub fn sensitivity(sensitivity: Sensitivity) -> Request {
        RawRequest::new(6, Subject::Settings, sensitivity as u32).into()
    }

    pub fn display(flags: InfoFlags) -> Request {
        RawRequest::new(7, Subject::Settings, flags.bits() as u32).into()
    }

    pub fn units(units: Units) -> Request {
        RawRequest::new(8, Subject::Settings, units as u32).into()
    }

    pub fn locked(is_locked: bool) -> Request {
        RawRequest::new(9, Subject::Settings, is_locked as u32).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
            RawRequest::new(8, Subject::State, 1u8).as_bytes(),
            &[
                REQUEST_HEADER,
                Subject::State as u8,
                8,
                1,
                171,
                MESSAGE_FOOTER
            ]
        );
        assert_eq!(
            RawRequest::new(8, Subject::State, u32::from_be_bytes([1, 2, 3, 4])).as_bytes(),
            &[
                REQUEST_HEADER,
                Subject::State as u8,
                8,
                1,
                2,
                3,
                4,
                180,
                MESSAGE_FOOTER
            ]
        );
    }
}
