use crate::{correlation_id::CorrelationId, element::Element, name::Name};
use blpapi_sys::*;
use std::ffi::CStr;

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
