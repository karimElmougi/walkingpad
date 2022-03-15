use super::*;

const REQUEST_HEADER: u8 = 0xf7;

/// Struct for producing the bytes representing the different requests the WalkingPad accepts.
///
/// # Examples
///
/// ```rust
/// use walkingpad_protocol::{Request, Speed};
///
/// let start_command = Request::start();
/// let get_settings = Request::get().settings();
/// let set_speed = Request::set().speed(Speed::from_hm_per_hour(25));
/// ```
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, PartialOrd)]
pub struct Request;

impl Request {
    pub const fn get() -> Get {
        Get
    }

    pub const fn set() -> Set {
        Set
    }

    /// Clears all data associated with past runs stored on the WalkingPad.
    pub const fn clear_stats() -> [u8; 6] {
        encode_u8_param(0xaa, Subject::StoredStats, 0)
    }

    pub const fn start() -> [u8; 6] {
        encode_u8_param(4, Subject::State, true as u8)
    }

    pub const fn stop() -> [u8; 6] {
        encode_u8_param(4, Subject::State, false as u8)
    }
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, PartialOrd)]
pub struct Get;

impl Get {
    /// Request for the WalkingPad's current state.
    pub const fn state(self) -> [u8; 6] {
        encode_u8_param(0, Subject::State, 0)
    }

    /// Request for the WalkingPad's current settings.
    pub const fn settings(self) -> [u8; 6] {
        encode_u8_param(0, Subject::Settings, 0)
    }

    /// Request for retrieving the stored run stats associated with the most recent run.
    pub const fn latest_stored_stats(self) -> [u8; 6] {
        const LATEST_STATS: u8 = 255;
        encode_u8_param(0xaa, Subject::StoredStats, LATEST_STATS)
    }

    /// Request for retrieving the stored run stats associated with a specific ID.
    pub const fn stored_stats(self, id: u8) -> [u8; 6] {
        encode_u8_param(0xaa, Subject::StoredStats, id)
    }
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, PartialOrd)]
pub struct Set;

impl Set {
    pub const fn speed(self, speed: Speed) -> [u8; 6] {
        encode_u8_param(1, Subject::State, speed.hm_per_hour())
    }

    pub const fn mode(self, mode: Mode) -> [u8; 6] {
        encode_u8_param(2, Subject::State, mode as u8)
    }

    pub const fn calibration_mode(self, enabled: bool) -> [u8; 9] {
        encode_u32_param(2, Subject::Settings, enabled as u32)
    }

    pub const fn max_speed(self, speed: Speed) -> [u8; 9] {
        encode_u32_param(3, Subject::Settings, speed.hm_per_hour() as u32)
    }

    pub const fn start_speed(self, speed: Speed) -> [u8; 9] {
        encode_u32_param(4, Subject::Settings, speed.hm_per_hour() as u32)
    }

    pub const fn auto_start(self, enabled: bool) -> [u8; 9] {
        encode_u32_param(5, Subject::Settings, enabled as u32)
    }

    pub const fn sensitivity(self, sensitivity: Sensitivity) -> [u8; 9] {
        encode_u32_param(6, Subject::Settings, sensitivity as u32)
    }

    pub const fn display(self, flags: InfoFlags) -> [u8; 9] {
        encode_u32_param(7, Subject::Settings, flags.bits() as u32)
    }

    pub const fn units(self, units: Units) -> [u8; 9] {
        encode_u32_param(8, Subject::Settings, units as u32)
    }

    pub const fn locked(self, is_locked: bool) -> [u8; 9] {
        encode_u32_param(9, Subject::Settings, is_locked as u32)
    }
}

/// Computes the simplistic CRC checksum scheme of the message's contents.
///
/// The checksum excludes the header and footer values, and is placed in the
/// second-to-last position in the array.
const fn crc<const N: usize>(mut bytes: [u8; N]) -> [u8; N] {
    // Skip header, footer and crc byte
    let crc_i = N - 2;
    let mut i = 1;

    let mut crc = 0u8;
    while i < crc_i {
        crc = crc.wrapping_add(bytes[i]);
        i += 1;
    }

    bytes[crc_i] = crc;
    bytes
}

const fn encode_u8_param(request_type: u8, subject: Subject, param: u8) -> [u8; 6] {
    crc([
        REQUEST_HEADER,
        subject as u8,
        request_type,
        param,
        0,
        MESSAGE_FOOTER,
    ])
}

const fn encode_u32_param(request_type: u8, subject: Subject, param: u32) -> [u8; 9] {
    let param = param.to_be_bytes();
    crc([
        REQUEST_HEADER,
        subject as u8,
        request_type,
        param[0],
        param[1],
        param[2],
        param[3],
        0,
        MESSAGE_FOOTER,
    ])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
            encode_u8_param(8, Subject::State, 1),
            [
                REQUEST_HEADER,
                Subject::State as u8,
                8,
                1,
                171,
                MESSAGE_FOOTER
            ]
        );
        assert_eq!(
            encode_u32_param(8, Subject::State, u32::from_be_bytes([1, 2, 3, 4])),
            [
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
