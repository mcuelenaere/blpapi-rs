use crate::{
    element::{Element, SetValue},
    name::Name,
    Error,
};
use blpapi_sys::*;
use std::ffi::CStr;
use std::ptr;
use std::os::raw::c_char;
use std::fmt::{Debug, Formatter};

/// A `Request`
///
/// A `Request` dereferences to an element
pub struct Request {
    pub(crate) ptr: *mut blpapi_Request_t,
    elements: *mut blpapi_Element_t,
}

impl Request {
    /// Create a new request
    pub(crate) unsafe fn new(ptr: *mut blpapi_Request_t) -> Self {
        let elements = blpapi_Request_elements(ptr);
        Request { ptr, elements }
    }

    /// Return the request's id if one exists, otherwise return None.
    ///
    /// If there are issues with this request, the request id
    /// can be reported to Bloomberg for troubleshooting purposes.
    ///
    /// Note that request id is not the same as correlation
    /// id and should not be used for correlation purposes.
    pub fn request_id(&self) -> Result<Option<String>, Error> {
        let mut request_id: *const c_char = ptr::null();
        let res = unsafe { blpapi_Request_getRequestId(self.ptr, &mut request_id) };
        Error::check(res)?;

        if request_id.is_null() {
            Ok(None)
        } else {
            unsafe { CStr::from_ptr(request_id) }
                .to_owned()
                .into_string()
                .map(|s| Some(s))
                .map_err(|err| Error::StringConversionError(Box::new(err)))
        }
    }

    /// Convert the request to an Element
    pub fn element(&self) -> Element {
        Element { ptr: self.elements }
    }

    /// Append a new value to the existing inner Element sequence defined by name
    pub fn append<V: SetValue>(&mut self, name: &str, value: V) -> Result<(), Error> {
        let mut element = self
            .element()
            .get_element(name)
            .ok_or_else(|| Error::NotFound(name.to_owned()))?;
        element.append(value)
    }

    /// Append a new value to the existing inner Element sequence defined by name
    pub fn append_named<V: SetValue>(&mut self, name: &Name, value: V) -> Result<(), Error> {
        self.element()
            .get_named_element(name)
            .ok_or_else(|| Error::NotFound(name.to_string()))?
            .append(value)
    }
}

impl Drop for Request {
    fn drop(&mut self) {
        unsafe { blpapi_Request_destroy(self.ptr) }
    }
}

impl Debug for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let request_id = match self.request_id() {
            Ok(Some(request_id)) => request_id,
            Ok(None) => "<None>".to_string(),
            Err(_) => "<Err>".to_string(),
        };
        f.write_fmt(format_args!("Request[requestId={}]", request_id))
    }
}

unsafe impl Send for Request {}
unsafe impl Sync for Request {}
