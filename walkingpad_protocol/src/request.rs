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

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Req<T> {
    header: u8,
    subject: u8,
    request_type: u8,
    param: T,
    crc: u8,
    footer: u8,
}

impl<T: PrimInt> Req<T> {
    fn new(request_type: u8, subject: Subject, param: T) -> Req<T> {
        let base_size = size_of::<Req<()>>();

        assert_eq!(base_size, 5);
        assert_eq!(size_of::<Req<T>>(), base_size + size_of::<T>());

        let param = param.to_be();
        let mut req = Req {
            header: REQUEST_HEADER,
            subject: subject as u8,
            request_type,
            param,
            crc: 0,
            footer: MESSAGE_FOOTER,
        };

        unsafe {
            let mut crc = 0u8;

            let ptr = &req as *const Req<T> as *const u8;
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
        let ptr = self as *const Req<T> as *const u8;
        unsafe { core::slice::from_raw_parts(ptr, size_of::<Req<T>>()) }
    }
}

/// Clears all data associated with past runs stored on the WalkingPad.
pub fn cleat_stats() -> Req<u8> {
    Req::new(0xaa, Subject::StoredStats, 0)
}

pub fn start() -> Req<u8> {
    Req::new(4, Subject::State, true as u8)
}

pub fn stop() -> Req<u8> {
    Req::new(4, Subject::State, false as u8)
}

pub mod get {
    use super::*;

    pub fn state() -> Req<u8> {
        Req::new(0, Subject::State, 0)
    }

    /// Request for the WalkingPad's current settings.
    pub fn settings() -> Req<u32> {
        Req::new(0, Subject::Settings, 0)
    }

    /// Request for retrieving the stored run stats associated with the most recent run.
    pub fn latest_stored_stats() -> Req<u8> {
        const LATEST_STATS: u8 = 255;
        Req::new(0xaa, Subject::StoredStats, LATEST_STATS)
    }

    /// Request for retrieving the stored run stats associated with a specific ID.
    pub fn stored_stats(id: u8) -> Req<u8> {
        Req::new(0xaa, Subject::StoredStats, id)
    }
}

pub mod set {
    use super::*;

    pub fn speed(speed: Speed) -> Req<u8> {
        Req::new(1, Subject::State, speed.hm_per_hour())
    }

    pub fn mode(mode: Mode) -> Req<u8> {
        Req::new(2, Subject::State, mode as u8)
    }

    pub fn calibration_mode(enabled: bool) -> Req<u32> {
        Req::new(2, Subject::Settings, enabled as u32)
    }

    pub fn max_speed(speed: Speed) -> Req<u32> {
        Req::new(3, Subject::Settings, speed.hm_per_hour() as u32)
    }

    pub fn start_speed(speed: Speed) -> Req<u32> {
        Req::new(4, Subject::Settings, speed.hm_per_hour() as u32)
    }

    pub fn auto_start(enabled: bool) -> Req<u32> {
        Req::new(5, Subject::Settings, enabled as u32)
    }

    pub fn sensitivity(sensitivity: Sensitivity) -> Req<u32> {
        Req::new(6, Subject::Settings, sensitivity as u32)
    }

    pub fn display(flags: InfoFlags) -> Req<u32> {
        Req::new(7, Subject::Settings, flags.bits() as u32)
    }

    pub fn units(units: Units) -> Req<u32> {
        Req::new(8, Subject::Settings, units as u32)
    }

    pub fn locked(is_locked: bool) -> Req<u32> {
        Req::new(9, Subject::Settings, is_locked as u32)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
            Req::new(8, Subject::State, 1u8).as_bytes(),
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
            Req::new(8, Subject::State, u32::from_be_bytes([1, 2, 3, 4])).as_bytes(),
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
