/*!
    Module for producing the bytes representing the different requests the WalkingPad accepts.

    # Examples

    ```rust
    use walkingpad_protocol::{request, Speed};

    let start_command = request::start();
    let get_settings = request::get::settings();
    let set_speed = request::set::speed(Speed::from_hm_per_hour(25));
*/

use core::fmt::Debug;
use core::mem::size_of;

use super::*;

/// Clears all data associated with past runs stored on the WalkingPad.
pub fn clear_stats() -> Request {
    Request::from_u8(0xaa, Subject::StoredStats, 0u8)
}

pub fn start() -> Request {
    Request::from_u8(4, Subject::State, true as u8)
}

pub fn stop() -> Request {
    Request::from_u8(4, Subject::State, false as u8)
}

pub mod get {
    use super::*;

    pub const fn state() -> Request {
        Request::from_u8(0, Subject::State, 0u8)
    }

    /// Request for the WalkingPad's current settings.
    pub const fn settings() -> Request {
        Request::from_u32(0, Subject::Settings, 0u32)
    }

    /// Request for retrieving the stored run stats associated with the most recent run.
    pub const fn latest_stored_stats() -> Request {
        const LATEST_STATS: u8 = 255;
        Request::from_u8(0xaa, Subject::StoredStats, LATEST_STATS)
    }

    /// Request for retrieving the stored run stats associated with a specific ID.
    pub const fn stored_stats(id: u8) -> Request {
        Request::from_u8(0xaa, Subject::StoredStats, id)
    }
}

pub mod set {
    use super::*;

    pub const fn speed(speed: Speed) -> Request {
        Request::from_u8(1, Subject::State, speed.hm_per_hour())
    }

    pub const fn mode(mode: Mode) -> Request {
        Request::from_u8(2, Subject::State, mode as u8)
    }

    pub const fn calibration_mode(enabled: bool) -> Request {
        Request::from_u32(2, Subject::Settings, enabled as u32)
    }

    pub const fn max_speed(speed: Speed) -> Request {
        Request::from_u32(3, Subject::Settings, speed.hm_per_hour() as u32)
    }

    pub const fn start_speed(speed: Speed) -> Request {
        Request::from_u32(4, Subject::Settings, speed.hm_per_hour() as u32)
    }

    pub const fn auto_start(enabled: bool) -> Request {
        Request::from_u32(5, Subject::Settings, enabled as u32)
    }

    pub const fn sensitivity(sensitivity: Sensitivity) -> Request {
        Request::from_u32(6, Subject::Settings, sensitivity as u32)
    }

    pub const fn display(flags: InfoFlags) -> Request {
        Request::from_u32(7, Subject::Settings, flags.bits() as u32)
    }

    pub const fn units(units: Units) -> Request {
        Request::from_u32(8, Subject::Settings, units as u32)
    }

    pub const fn locked(is_locked: bool) -> Request {
        Request::from_u32(9, Subject::Settings, is_locked as u32)
    }
}

const REQUEST_HEADER: u8 = 0xf7;

#[derive(Clone, PartialEq, Eq)]
pub struct Request(RequestVariant);

impl Request {
    const fn from_u8(request_type: u8, subject: Subject, param: u8) -> Request {
        Request(RequestVariant::U8(RawRequest::new(
            request_type,
            subject,
            [param],
        )))
    }

    const fn from_u32(request_type: u8, subject: Subject, param: u32) -> Request {
        let param = param.to_be_bytes();
        Request(RequestVariant::U32(RawRequest::new(
            request_type,
            subject,
            param,
        )))
    }

    pub fn as_bytes(&self) -> &[u8] {
        match &self.0 {
            RequestVariant::U8(req) => req.as_bytes(),
            RequestVariant::U32(req) => req.as_bytes(),
        }
    }
}

impl Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request")
            .field("subject", &self.0.subject())
            .field("request_type", &self.0.request_type())
            .field("param", &self.0.param())
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
enum RequestVariant {
    U8(RawRequest<1>),
    U32(RawRequest<4>),
}

impl RequestVariant {
    fn subject(&self) -> Subject {
        match self {
            RequestVariant::U8(r) => r.subject.try_into().unwrap(),
            RequestVariant::U32(r) => r.subject.try_into().unwrap(),
        }
    }

    fn request_type(&self) -> u8 {
        match self {
            RequestVariant::U8(r) => r.request_type,
            RequestVariant::U32(r) => r.request_type,
        }
    }

    fn param(&self) -> u32 {
        match self {
            RequestVariant::U8(r) => r.param[0] as u32,
            RequestVariant::U32(r) => u32::from_be_bytes(r.param),
        }
    }
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
struct RawRequest<const N: usize> {
    header: u8,
    subject: u8,
    request_type: u8,
    param: [u8; N],
    crc: u8,
    footer: u8,
}

impl<const N: usize> RawRequest<N> {
    const fn new(request_type: u8, subject: Subject, param: [u8; N]) -> RawRequest<N> {
        let base_size = size_of::<RawRequest<0>>();

        assert!(base_size == 5);
        assert!(size_of::<Self>() == base_size + N);

        let req = RawRequest {
            header: REQUEST_HEADER,
            subject: subject as u8,
            request_type,
            param,
            crc: 0,
            footer: MESSAGE_FOOTER,
        };

        let mut crc = 0u8;

        crc = crc.wrapping_add(req.subject);
        crc = crc.wrapping_add(req.request_type);

        let mut i = 0;
        while i < N {
            crc = crc.wrapping_add(param[i]);
            i += 1;
        }

        RawRequest { crc, ..req }
    }

    fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const RawRequest<N> as *const u8;
        unsafe { core::slice::from_raw_parts(ptr, size_of::<Self>()) }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
            Request::from_u8(8, Subject::State, 1u8).as_bytes(),
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
            Request::from_u32(8, Subject::State, u32::from_be_bytes([1, 2, 3, 4])).as_bytes(),
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
