use crate::{request::Request, Error};
use blpapi_sys::*;
use std::ffi::CStr;
use std::fmt::{Debug, Display, Formatter};
use std::os::raw::c_int;

/// A `Service`
/// created from a `Session::get_service`
pub struct Service(pub(crate) *mut blpapi_Service_t);

impl Service {
    /// Get service name
    pub fn name(&self) -> String {
        let name = unsafe { CStr::from_ptr(blpapi_Service_name(self.0)) };
        name.to_string_lossy().into_owned()
    }

    /// Create a new request
    pub fn create_request(&self, operation: &str) -> Result<Request, Error> {
        Request::new(self, operation)
    }
    /// Format this Service schema to the specified formatter' at
    /// (absolute value specified for) the optionally specified indentation
    /// 'indent_level'. If 'level' is specified, optionally specify 'spaces_per_level',
    /// the number of spaces per indentation level for this and all of its
    /// nested objects. If 'indent_level' is negative, suppress indentation
    /// of the first line. If 'spaces_per_level' is negative, format
    /// the entire output on one line, suppressing all but the
    /// initial indentation (as governed by 'indent_level').
    pub fn print(&self, f: &mut Formatter<'_>, indent_level: isize, spaces_per_level: isize) -> Result<(), Error> {
        let res = unsafe {
            let stream = std::mem::transmute(f);
            blpapi_Service_print(
                self.0,
                Some(crate::utils::stream_writer),
                stream,
                indent_level as c_int,
                spaces_per_level as c_int
            )
        };
        Error::check(res)
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        unsafe { blpapi_Service_release(self.0) }
    }
}

impl Debug for Service {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Service[name={}]", self.name()))
    }
}

impl Display for Service {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.print(f, 0, 4).map_err(|_| std::fmt::Error)
    }
}