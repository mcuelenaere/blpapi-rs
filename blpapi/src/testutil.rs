use blpapi_sys::*;
use std::ptr;
use crate::event::{EventType, Event};
use crate::Error;
use crate::name::Name;
use std::ffi::CString;

pub struct MessageProperties(pub(crate) *mut blpapi_MessageProperties_t);

impl MessageProperties {
    pub fn new() -> Result<Self, Error> {
        let mut message_properties: *mut blpapi_MessageProperties_t = ptr::null_mut();
        let res = unsafe { blpapi_MessageProperties_create(&mut message_properties ) };
        Error::check(res)?;

        Ok(MessageProperties(message_properties))
    }
}

impl Drop for MessageProperties {
    fn drop(&mut self) {
        unsafe { blpapi_MessageProperties_destroy(self.0) };
    }
}

struct MessageFormatter(pub(crate) *mut blpapi_MessageFormatter_t);

impl MessageFormatter {
    pub fn format_message_json(&mut self, json: &str) -> Result<(), Error> {
        let json = CString::new(json).unwrap();
        let res = unsafe { blpapi_MessageFormatter_FormatMessageJson(self.0, json.as_ptr()) };
        Error::check(res)
    }

    pub fn format_message_xml(&mut self, xml: &str) -> Result<(), Error> {
        let xml = CString::new(xml).unwrap();
        let res = unsafe { blpapi_MessageFormatter_FormatMessageXml(self.0, xml.as_ptr()) };
        Error::check(res)
    }
}

impl Drop for MessageFormatter {
    fn drop(&mut self) {
        unsafe { blpapi_MessageFormatter_destroy(self.0) };
    }
}

pub struct EventBuilder {
    event: Event,
}

impl EventBuilder {
    pub fn new(event_type: EventType,) -> Result<Self, Error> {
        let mut event: *mut blpapi_Event_t = ptr::null_mut();
        let res = unsafe { blpapi_TestUtil_createEvent(&mut event, event_type.into()) };
        Error::check(res)?;

        Ok(EventBuilder { event: Event(event) })
    }

    fn append_message(&mut self, message_type: Name, message_properties: Option<MessageProperties>) -> Result<MessageFormatter, Error> {
        let mut schema_definition: *mut blpapi_SchemaElementDefinition_t = ptr::null_mut();
        let res = unsafe { blpapi_TestUtil_getAdminMessageDefinition(&mut schema_definition, message_type.0) };
        Error::check(res)?;

        let message_properties = message_properties.unwrap_or_else(|| MessageProperties::new().unwrap());
        let mut formatter: *mut blpapi_MessageFormatter_t = ptr::null_mut();
        let res = unsafe { blpapi_TestUtil_appendMessage(&mut formatter, self.event.0, schema_definition, message_properties.0) };
        Error::check(res)?;

        Ok(MessageFormatter(formatter))
    }

    pub fn append_message_from_json(mut self, message_type: Name, message_properties: Option<MessageProperties>, json: &str) -> Result<Self, Error> {
        let mut formatter = self.append_message(message_type, message_properties)?;
        formatter.format_message_json(json)?;

        Ok(self)
    }

    pub fn append_message_from_xml(mut self, message_type: Name, message_properties: Option<MessageProperties>, xml: &str) -> Result<Self, Error> {
        let mut formatter = self.append_message(message_type, message_properties)?;
        formatter.format_message_xml(xml)?;

        Ok(self)
    }

    pub fn build(self) -> Event {
        self.event
    }
}