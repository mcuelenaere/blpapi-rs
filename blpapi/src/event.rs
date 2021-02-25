use crate::errors::Error;
use crate::message::Message;
use blpapi_sys::*;
use std::os::raw::c_int;
use std::marker::PhantomData;
use std::fmt::{Debug, Formatter};
use std::ptr;

/// An event
pub struct Event(pub(crate) *mut blpapi_Event_t);

impl Event {
    /// Get event type
    pub fn event_type(&self) -> EventType {
        unsafe { blpapi_Event_eventType(self.0).into() }
    }

    /// Get an iterator over all messages of this event
    pub fn messages(&self) -> MessageIterator {
        let ptr = unsafe { blpapi_MessageIterator_create(self.0) };
        MessageIterator {
            ptr,
            _phantom: PhantomData,
        }
    }
}

impl Clone for Event {
    fn clone(&self) -> Self {
        unsafe { blpapi_Event_addRef(self.0) };
        Event(self.0)
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        unsafe { blpapi_Event_release(self.0); }
    }
}

impl Debug for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Event[eventType={:?}]", self.event_type()))
    }
}

unsafe impl Send for Event {}
unsafe impl Sync for Event {}

/// A message iterator
pub struct MessageIterator<'a> {
    pub(crate) ptr: *mut blpapi_MessageIterator_t,
    _phantom: PhantomData<&'a Event>,
}

impl<'a> Drop for MessageIterator<'a> {
    fn drop(&mut self) {
        unsafe { blpapi_MessageIterator_destroy(self.ptr) }
    }
}

impl<'a> Iterator for MessageIterator<'a> {
    type Item = Message;

    fn next(&mut self) -> Option<Message> {
        let mut ptr = ptr::null_mut();
        let res = unsafe { blpapi_MessageIterator_next(self.ptr, &mut ptr as *mut _) };
        if res == 0 {
            // Make sure to increment the refcount, so that we can safely drop the message
            // when we're done with it (or that it may outlive this MessageIterator).
            unsafe { blpapi_Message_addRef(ptr) };
            Some(Message(ptr))
        } else {
            None
        }
    }
}

unsafe impl Send for MessageIterator<'_> {}
unsafe impl Sync for MessageIterator<'_> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    Admin,
    SessionStatus,
    SubscriptionStatus,
    RequestStatus,
    Response,
    PartialResponse,
    SubscriptionData,
    ServiceStatus,
    Timeout,
    AuthorizationStatus,
    ResolutionStatus,
    TopicStatus,
    TokenStatus,
    Request,
    Unknown = -1,
}

impl From<c_int> for EventType {
    fn from(v: c_int) -> Self {
        match v as u32 {
            BLPAPI_EVENTTYPE_ADMIN => EventType::Admin,
            BLPAPI_EVENTTYPE_SESSION_STATUS => EventType::SessionStatus,
            BLPAPI_EVENTTYPE_SUBSCRIPTION_STATUS => EventType::SubscriptionStatus,
            BLPAPI_EVENTTYPE_REQUEST_STATUS => EventType::RequestStatus,
            BLPAPI_EVENTTYPE_RESPONSE => EventType::Response,
            BLPAPI_EVENTTYPE_PARTIAL_RESPONSE => EventType::PartialResponse,
            BLPAPI_EVENTTYPE_SUBSCRIPTION_DATA => EventType::SubscriptionData,
            BLPAPI_EVENTTYPE_SERVICE_STATUS => EventType::ServiceStatus,
            BLPAPI_EVENTTYPE_TIMEOUT => EventType::Timeout,
            BLPAPI_EVENTTYPE_AUTHORIZATION_STATUS => EventType::AuthorizationStatus,
            BLPAPI_EVENTTYPE_RESOLUTION_STATUS => EventType::ResolutionStatus,
            BLPAPI_EVENTTYPE_TOPIC_STATUS => EventType::TopicStatus,
            BLPAPI_EVENTTYPE_TOKEN_STATUS => EventType::TokenStatus,
            BLPAPI_EVENTTYPE_REQUEST => EventType::Request,
            _ => EventType::Unknown,
        }
    }
}

impl Into<c_int> for EventType {
    fn into(self) -> c_int {
        match self {
            EventType::Admin => BLPAPI_EVENTTYPE_ADMIN as c_int,
            EventType::SessionStatus => BLPAPI_EVENTTYPE_SESSION_STATUS as c_int,
            EventType::SubscriptionStatus => BLPAPI_EVENTTYPE_SUBSCRIPTION_STATUS as c_int,
            EventType::RequestStatus => BLPAPI_EVENTTYPE_REQUEST_STATUS as c_int,
            EventType::Response => BLPAPI_EVENTTYPE_RESPONSE as c_int,
            EventType::PartialResponse => BLPAPI_EVENTTYPE_PARTIAL_RESPONSE as c_int,
            EventType::SubscriptionData => BLPAPI_EVENTTYPE_SUBSCRIPTION_DATA as c_int,
            EventType::ServiceStatus => BLPAPI_EVENTTYPE_SERVICE_STATUS as c_int,
            EventType::Timeout => BLPAPI_EVENTTYPE_TIMEOUT as c_int,
            EventType::AuthorizationStatus => BLPAPI_EVENTTYPE_AUTHORIZATION_STATUS as c_int,
            EventType::ResolutionStatus => BLPAPI_EVENTTYPE_RESOLUTION_STATUS as c_int,
            EventType::TopicStatus => BLPAPI_EVENTTYPE_TOPIC_STATUS as c_int,
            EventType::TokenStatus => BLPAPI_EVENTTYPE_TOKEN_STATUS as c_int,
            EventType::Request => BLPAPI_EVENTTYPE_REQUEST as c_int,
            EventType::Unknown => 0,
        }
    }
}

pub struct EventQueue(pub(crate) *mut blpapi_EventQueue_t);

impl EventQueue {
    /// Construct an empty event queue.
    pub fn new() -> Self {
        let ptr = unsafe { blpapi_EventQueue_create() };
        EventQueue(ptr)
    }

    /// Returns the next Event available from the EventQueue. If
    /// the specified 'timeout' is zero this will wait forever for
    /// the next event. If the specified 'timeout' is non zero then
    /// if no Event is available within the specified 'timeout' in
    /// milliseconds an Event with a type() of TIMEOUT will be returned.
    pub fn next_event(&mut self, timeout: Option<isize>) -> Event {
        let timeout = timeout.unwrap_or(0) as c_int;
        let event = unsafe { blpapi_EventQueue_nextEvent(self.0, timeout) };
        Event(event)
    }

    /// If the EventQueue is non-empty, return the next Event available.
    /// If the EventQueue is empty, return None with no effect on the state
    /// of EventQueue. This method never blocks.
    pub fn try_next_event(&mut self) -> Result<Event, Error> {
        let mut event: *mut blpapi_Event_t = ptr::null_mut();
        let ret = unsafe { blpapi_EventQueue_tryNextEvent(self.0, &mut event) };
        Error::check(ret)?;

        Ok(Event(event))
    }

    /// Purges any Event objects in this EventQueue which have not
    /// been processed and cancel any pending requests linked to
    /// this EventQueue. The EventQueue can subsequently be
    /// re-used for a subsequent request.
    pub fn purge(&mut self) {
        unsafe { blpapi_EventQueue_purge(self.0); }
    }
}

impl Drop for EventQueue {
    /// Destroy this event queue and cancel any pending request
    /// that are linked to this queue.
    fn drop(&mut self) {
        unsafe { blpapi_EventQueue_destroy(self.0); }
    }
}