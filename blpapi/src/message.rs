use crate::{correlation_id::CorrelationId, errors::Error, element::Element, name::Name};
use blpapi_sys::*;
use std::ffi::CStr;
use std::fmt::{Display, Debug, Formatter};
use std::os::raw::c_int;

/// A message
pub struct Message(pub(crate) *mut blpapi_Message_t);

impl Message {
    /// Get topic name
    pub fn topic_name(&self) -> String {
        unsafe {
            let name = blpapi_Message_topicName(self.0);
            CStr::from_ptr(name).to_string_lossy().into_owned()
        }
    }

    /// Get type string
    pub fn type_string(&self) -> String {
        unsafe {
            let name = blpapi_Message_typeString(self.0);
            CStr::from_ptr(name).to_string_lossy().into_owned()
        }
    }

    /// Get message type
    pub fn message_type(&self) -> Name {
        unsafe {
            let ptr = blpapi_Message_messageType(self.0);
            Name(ptr)
        }
    }

    /// Get number of correlation ids
    pub fn num_correlation_ids(&self) -> usize {
        unsafe { blpapi_Message_numCorrelationIds(self.0) as usize }
    }

    /// Get correlation id
    pub fn correlation_id(&self, index: usize) -> Option<CorrelationId> {
        if index > self.num_correlation_ids() {
            None
        } else {
            unsafe {
                let ptr = blpapi_Message_correlationId(self.0, index);
                Some(CorrelationId(ptr))
            }
        }
    }

    /// Get corresponding element
    pub fn element(&self) -> Element {
        let elements = unsafe { blpapi_Message_elements(self.0) };
        Element { ptr: elements }
    }

    /// Format this Message to the specified formatter at the
    /// (absolute value of) the optionally specified indentation
    /// 'indent_level'. If 'indent_level' is specified, optionally
    /// specify 'spaces_per_level', the number of spaces per indentation
    /// level for this and all of its nested objects. If 'indent_level'
    /// is negative, suppress indentation of the first line. If
    /// 'spaces_per_level' is negative, format the entire output on
    /// one line, suppressing all but the initial indentation (as
    /// governed by 'indent_level').
    pub fn print(&self, f: &mut Formatter<'_>, indent_level: isize, spaces_per_level: isize) -> Result<(), Error> {
        let res = unsafe {
            let stream = std::mem::transmute(f);
            blpapi_Message_print(
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

impl Clone for Message {
    fn clone(&self) -> Self {
        unsafe { blpapi_Message_addRef(self.0) };
        Message(self.0)
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        unsafe { blpapi_Message_release(self.0) };
    }
}

impl Debug for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Message[messageType={}]", self.message_type().to_string_lossy()))
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.print(f, 0, 4).map_err(|_| std::fmt::Error)
    }
}
