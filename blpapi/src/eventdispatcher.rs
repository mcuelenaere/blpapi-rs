use crate::errors::Error;
use blpapi_sys::*;
use std::fmt::Debug;
use std::os::raw::c_int;

/// Dispatches events from one or more Sessions through callbacks
///
/// EventDispatcher objects are optionally specified when Session
/// objects are constructed. A single EventDispatcher can be shared
/// by multiple Session objects.
///
/// The EventDispatcher provides an event-driven interface,
/// generating callbacks from one or more internal threads for one
/// or more sessions.
#[derive(Debug)]
pub struct EventDispatcher(pub(crate) *mut blpapi_EventDispatcher_t);

impl EventDispatcher {
    /// Construct an EventDispatcher with the specified
    /// 'num_dispatcher_threads'. If 'num_dispatcher_threads' is 1 (the
    /// default) then a single internal thread is created to
    /// dispatch events. If 'num_dispatcher_threads' is greater than
    /// 1 then an internal pool of 'num_dispatcher_threads' threads
    /// is created to dispatch events. The behavior is undefined
    /// if 'num_dispatcher_threads' is 0.
    pub fn new(num_dispatcher_threads: usize) -> Self {
        let ptr = unsafe { blpapi_EventDispatcher_create(num_dispatcher_threads) };
        EventDispatcher(ptr)
    }

    /// Start generating callbacks for events from sessions
    /// associated with this EventDispatcher.
    pub fn start(&self) -> Result<(), Error> {
        let res = unsafe { blpapi_EventDispatcher_start(self.0) };
        Error::check(res)
    }

    /// Shutdown this event dispatcher object and stop generating
    /// callbacks for events from sessions associated with it.
    /// If the specified 'async_' is false (the default) then this
    /// method blocks until all current callbacks which were dispatched
    /// through this EventDispatcher have completed.
    ///
    /// Note: Calling stop with 'async_' of false from within a callback
    /// dispatched by this EventDispatcher is undefined and may result
    /// in a deadlock.
    pub fn stop(&self, async_: bool) -> Result<(), Error> {
        let res = unsafe { blpapi_EventDispatcher_stop(self.0, async_ as c_int) };
        Error::check(res)
    }
}

impl Drop for EventDispatcher {
    fn drop(&mut self) {
        unsafe { blpapi_EventDispatcher_destroy(self.0); }
    }
}