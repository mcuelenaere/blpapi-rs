use crate::errors::Error;
use crate::service::Service;
use blpapi_sys::*;
use std::os::raw::c_int;

pub enum SeatType {
    InvalidSeat,
    Bps,
    NonBps,
}

impl From<i32> for SeatType {
    fn from(seat_type: c_int) -> Self {
        match seat_type as u32 {
            BLPAPI_SEATTYPE_BPS => SeatType::Bps,
            BLPAPI_SEATTYPE_NONBPS => SeatType::NonBps,
            _ => SeatType::InvalidSeat,
        }
    }
}

pub struct Identity(pub(crate) *mut blpapi_Identity_t);

impl Identity {
    // TODO: blpapi_Identity_hasEntitlements

    /// Return true if this 'Identity' is authorized to consume the
    /// specified 'service'; otherwise return false.
    pub fn is_authorized(&self, service: &Service) -> bool {
        let ret = unsafe { blpapi_Identity_isAuthorized(self.0, service.0) };
        ret != 0
    }

    /// Return the seat type of this 'Identity'.
    pub fn get_seat_type(&self) -> Result<SeatType, Error> {
        let mut seat_type: c_int = BLPAPI_SEATTYPE_INVALID_SEAT;
        let ret = unsafe { blpapi_Identity_getSeatType(self.0, &mut seat_type) };
        Error::check(ret)?;
        Ok(SeatType::from(seat_type))
    }
}

impl Clone for Identity {
    fn clone(&self) -> Self {
        unsafe { blpapi_Identity_addRef(self.0) };
        Identity(self.0)
    }
}

impl Drop for Identity {
    /// Destructor. Destroying the last Identity for a specific
    /// user cancels any authorizations associated with it.
    fn drop(&mut self) {
        unsafe { blpapi_Identity_release(self.0); }
    }
}