use crate::{
    correlation_id::CorrelationId,
    event::{Event, EventQueue},
    eventdispatcher::EventDispatcher,
    identity::Identity,
    request::Request,
    service::Service,
    session_options::SessionOptions,
    Error,
};
use blpapi_sys::*;
use std::{ffi::CString, ptr};
use std::os::raw::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use crate::subscriptionlist::SubscriptionList;
use std::marker::PhantomData;
use std::ffi::CStr;
use std::pin::Pin;

type EventHandlerFn<'a> = dyn FnMut(&Event) -> () + 'a + Send;
type EventHandlerCallback = unsafe extern "C" fn(*mut blpapi_Event_t, *mut blpapi_Session_t, *mut c_void);

unsafe extern "C" fn event_handler_callback(event: *mut blpapi_Event_t, _: *mut blpapi_Session_t, user_data: *mut c_void) {
    let event_handler: &mut Box<EventHandlerFn> = std::mem::transmute(user_data);
    let event = Event(event);
    if let Err(err) = catch_unwind(AssertUnwindSafe(move || (*event_handler)(&event))) {
        eprintln!("{:?}", err);
        std::process::abort();
    }
}

/// This class provides a consumer session for making requests for Bloomberg
/// services.
///
/// Sessions manage access to services either by requests and
/// responses or subscriptions. A Session can dispatch events and
/// replies in either a synchronous or asynchronous mode. The mode
/// of a Session is determined when it is constructed and cannot be
/// changed subsequently.
///
/// A Session is asynchronous if an EventHandler object is
/// supplied when it is constructed. All incoming events are delivered to
/// the EventHandler supplied on construction.  Calls to 'nextEvent'
/// on an asynchronous session will fail.
///
/// A Session is synchronous if an EventHandler object is not
/// supplied when it is constructed. The nextEvent() method must be
/// called to read incoming events.
///
/// Several methods in Session take a CorrelationId parameter. The
/// application may choose to supply its own CorrelationId values
/// or allow the Session to create values. If the application
/// supplies its own CorrelationId values it must manage their
/// lifetime such that the same value is not reused for more than
/// one operation at a time. The lifetime of a CorrelationId begins
/// when it is supplied in a method invoked on a Session and ends
/// either when it is explicitly cancelled using cancel() or
/// unsubscribe(), when a RESPONSE Event (not a PARTIAL_RESPONSE)
/// containing it is received or when a SUBSCRIPTION_STATUS Event
/// which indicates that the subscription it refers to has been
/// terminated is received.
///
/// When using an asynchronous Session the application must be
/// aware that because the callbacks are generated from another
/// thread they may be processed before the call which generates
/// them has returned. For example, the SESSION_STATUS Event
/// generated by a startAsync() may be processed before
/// startAsync() has returned (even though startAsync() itself will
/// not block).
///
/// This becomes more significant when Session generated
/// CorrelationIds are in use. For example, if a call to
/// subscribe() which returns a Session generated CorrelationId has
/// not completed before the first Events which contain that
/// CorrelationId arrive the application may not be able to
/// interpret those events correctly. For this reason, it is
/// preferable to use user generated CorrelationIds when using
/// asynchronous Sessions. This issue does not arise when using a
/// synchronous Session as long as the calls to subscribe() etc. are
/// made on the same thread as the calls to nextEvent().
#[allow(dead_code)]
pub struct Session<'a>
{
    pub(crate) ptr: *mut blpapi_Session_t,
    event_handler_fn: Option<Box<EventHandlerFn<'a>>>,
}

impl<'a> Session<'a> {
    /// Construct a Session using the optionally specified
    /// 'options', the optionally specified 'eventHandler' and the
    /// optionally specified 'eventDispatcher'.
    ///
    /// See the SessionOptions documentation for details on what
    /// can be specified in the 'options'.
    ///
    /// If 'eventHandler' is not null then this Session will operation
    /// in asynchronous mode, otherwise the Session will operate in
    /// synchronous mode.
    ///
    /// If 'eventDispatcher' is null then the Session will create a
    /// default EventDispatcher for this Session which will use a
    /// single thread for dispatching events. For more control over
    /// event dispatching a specific instance of EventDispatcher
    /// can be supplied. This can be used to share a single
    /// EventDispatcher amongst multiple Session objects.
    ///
    /// If an 'eventDispatcher' is supplied which uses more than
    /// one thread the Session will ensure that events which should
    /// be ordered are passed to callbacks in a correct order. For
    /// example, partial response to a request or updates to a
    /// single subscription.
    ///
    /// The behavior is undefined if 'eventHandler' is null and the
    /// 'eventDispatcher' is not null.
    ///
    /// Each EventDispatcher uses its own thread or pool of
    /// threads so if you want to ensure that a session which
    /// receives very large messages and takes a long time to
    /// process them does not delay a session that receives small
    /// messages and processes each one very quickly then give each
    /// one a separate EventDispatcher.
    pub fn create(options: SessionOptions, event_handler: Option<impl FnMut(&Event) -> () + Send + 'a>, event_dispatcher: Option<EventDispatcher>) -> Pin<Box<Self>> {
        let event_dispatcher = event_dispatcher.map_or(ptr::null_mut(), |event_dispatcher| event_dispatcher.0);
        let mut session = Box::pin(Session {
            ptr: ptr::null_mut(),
            event_handler_fn: event_handler.map(|event_handler_fn| Box::new(event_handler_fn) as _)
        });
        session.ptr = unsafe {
            match session.event_handler_fn.as_ref() {
                Some(callback_user_data_ref) => {
                    blpapi_Session_create(
                        options.0,
                        Some(event_handler_callback as EventHandlerCallback),
                        event_dispatcher,
                        std::mem::transmute(callback_user_data_ref)
                    )
                },
                None => {
                    blpapi_Session_create(
                        options.0,
                        None,
                        event_dispatcher,
                        ptr::null_mut(),
                    )
                }
            }
        };

        session
    }

    /// Attempt to start this Session and block until the Session
    /// has started or failed to start. If the Session is started
    /// successfully 'true' is returned, otherwise 'false' is
    /// returned. Before start() returns a SESSION_STATUS Event is
    /// generated. If this is an asynchronous Session then the
    /// SESSION_STATUS may be processed by the registered
    /// EventHandler before start() has returned. A Session may
    /// only be started once.
    pub fn start(&mut self) -> bool {
        let res = unsafe { blpapi_Session_start(self.ptr) };
        res != 0
    }

    /// Attempt to begin the process to start this Session and
    /// return 'true' if successful, otherwise return 'false'. The
    /// application must monitor events for a SESSION_STATUS Event
    /// which will be generated once the Session has started or if
    /// it fails to start. If this is an asynchronous Session then
    /// the SESSION_STATUS Event may be processed by the registered
    /// EventHandler before startAsync() has returned. A Session may
    /// only be started once.
    pub fn start_async(&mut self) -> bool {
        let res = unsafe { blpapi_Session_startAsync(self.ptr) };
        res != 0
    }

    /// Stop operation of this session and block until all callbacks to
    /// EventHandler objects relating to this Session which are currently in
    /// progress have completed (including the callback to handle the
    /// SESSION_STATUS Event with SessionTerminated message this call
    /// generates). Once this returns no further callbacks to EventHandlers
    /// will occur. If stop() is called from within an EventHandler callback
    /// the behavior is undefined and may result in a deadlock. Once a
    /// Session has been stopped it can only be destroyed.
    pub fn stop(&mut self) {
        unsafe { blpapi_Session_stop(self.ptr) };
    }

    /// Begin the process to stop this Session and return immediately. The
    /// application must monitor events for a SESSION_STATUS Event with
    /// SessionTerminated message which will be generated once the
    /// Session has been stopped. After this SESSION_STATUS Event no further
    /// callbacks to EventHandlers will occur. This method can be called
    /// from within an EventHandler callback to stop Sessions using
    /// non-default (external) EventDispatcher. Once a Session has been
    /// stopped it can only be destroyed.
    pub fn stop_async(&mut self) {
        unsafe { blpapi_Session_stopAsync(self.ptr) };
    }

    /// Attempt to open the service identified by the specified
    /// 'serviceIdentifier' and block until the service is either opened
    /// successfully or has failed to be opened. Return 'true' if
    /// the service is opened successfully and 'false' if the
    /// service cannot be successfully opened.
    ///
    /// The 'serviceIdentifier' must contain a fully qualified service name.
    /// That is, it must be of the form '//<namespace>/<local-name>'.
    ///
    /// Before openService() returns a SERVICE_STATUS Event is
    /// generated. If this is an asynchronous Session then this
    /// Event may be processed by the registered EventHandler
    /// before openService() has returned.
    pub fn open_service(&mut self, service: &str) -> bool {
        let service = CString::new(service).unwrap();
        let res = unsafe { blpapi_Session_openService(self.ptr, service.as_ptr()) };
        res != 0
    }

    /// Begin the process to open the service identified by the
    /// specified 'serviceIdentifier' and return immediately. The optional
    /// specified 'correlationId' is used to track Events generated
    /// as a result of this call. The actual correlationId which
    /// will identify Events generated as a result of this call is
    /// returned.
    ///
    /// The 'serviceIdentifier' must contain a fully qualified service name.
    /// That is, it must be of the form '//<namespace>/<local-name>'.
    ///
    /// The application must monitor events for a SERVICE_STATUS
    /// Event which will be generated once the service has been
    /// successfully opened or the opening has failed.
    pub fn open_service_async(&mut self, service: &str, correlation_id: Option<CorrelationId>) -> Result<CorrelationId, Error> {
        let service = CString::new(service).unwrap();
        let mut correlation_id = correlation_id.unwrap_or_else(|| CorrelationId::new_empty());
        let res = unsafe { blpapi_Session_openServiceAsync(self.ptr, service.as_ptr(), &mut correlation_id.0) };
        Error::check(res)?;

        Ok(correlation_id)
    }

    /// Return a Service object representing the service
    /// identified by the specified 'serviceIdentifier'
    ///
    /// The 'serviceIdentifier' must contain a fully qualified service name.
    /// That is, it must be of the form '//<namespace>/<local-name>'.
    ///
    /// If the service identified by 'serviceIdentifier' is not open or
    /// registered already then a 'NotFoundException' is thrown.
    pub fn get_service(&self, service: &str) -> Result<Option<Service>, Error> {
        let name = CString::new(service).unwrap();
        let mut service: *mut blpapi_Service_t = ptr::null_mut();
        let res =
            unsafe { blpapi_Session_getService(self.ptr, &mut service, name.as_ptr()) };
        Error::check(res)?;

        let result = if service.is_null() { None } else { Some(Service(service)) };
        Ok(result)
    }

    /// Return a Identity which is valid but has not been
    /// authorized.
    pub fn create_identity(&mut self) -> Identity {
        let identity = unsafe { blpapi_Session_createIdentity(self.ptr) };
        Identity(identity)
    }

    /// Generate a token to be used for authorization.
    /// If invalid authentication option is specified in session option or
    /// there is failure to get authentication information based on
    /// authentication option, or if the authentication mode is 'MANUAL' for
    /// a user or user and application authentication, then an
    /// InvalidArgumentException is thrown.
    pub fn generate_token(
        &mut self,
        correlation_id: Option<CorrelationId>,
        event_queue: Option<&EventQueue>
    ) -> Result<CorrelationId, Error> {
        let mut correlation_id = correlation_id.unwrap_or_else(|| CorrelationId::new_empty());
        let event_queue = event_queue.map_or(ptr::null_mut(), |event_queue| event_queue.0);
        let res = unsafe {
            blpapi_Session_generateToken(
                self.ptr,
                &mut correlation_id.0,
                event_queue
            )
        };
        Error::check(res)?;
        Ok(correlation_id)
    }

    /// Send the specified 'request' using the specified 'identity' for
    /// authorization. If the optionally specified 'correlationId' is
    /// supplied use it otherwise create a CorrelationId. The actual
    /// CorrelationId used is returned. If the optionally specified
    /// 'eventQueue' is supplied all events relating to this Request will
    /// arrive on that EventQueue. If the optional 'requestLabel' and
    /// 'requestLabelLen' are provided they define a string which will be
    /// recorded along with any diagnostics for this operation. There must
    /// be at least 'requestLabelLen' printable characters at the location
    /// 'requestLabel'.
    ///
    /// A successful request will generate zero or more PARTIAL_RESPONSE
    /// Messages followed by exactly one RESPONSE Message. Once the final
    /// RESPONSE Message has been received the CorrelationId associated with
    /// this request may be re-used. If the request fails at any stage a
    /// REQUEST_STATUS will be generated after which the CorrelationId
    /// associated with the request may be re-used.
    pub fn send_request(
        &mut self,
        request: Request,
        identity: Option<&Identity>,
        event_queue: Option<&EventQueue>,
        correlation_id: Option<CorrelationId>,
    ) -> Result<CorrelationId, Error> {
        let mut correlation_id = correlation_id.unwrap_or_else(|| CorrelationId::new_empty());
        let identity = identity.map_or(ptr::null_mut(), |identity| identity.0);
        let event_queue = event_queue.map_or(ptr::null_mut(), |event_queue| event_queue.0);
        let request_label = ptr::null_mut();
        let request_label_len = 0;
        let res = unsafe {
            blpapi_Session_sendRequest(
                self.ptr,
                request.ptr,
                &mut correlation_id.0,
                identity,
                event_queue,
                request_label,
                request_label_len,
            )
        };
        Error::check(res)?;

        Ok(correlation_id)
    }

    /// Send the specified 'authorizationRequest' and update the
    /// specified 'identity' with the results. If the optionally
    /// specified 'correlationId' is supplied, it is used; otherwise
    /// create a CorrelationId. The actual CorrelationId used is
    /// returned. If the optionally specified 'eventQueue' is
    /// supplied all Events relating to this Request will arrive on
    /// that EventQueue.
    ///
    /// The underlying user information must remain valid until the
    /// Request has completed successfully or failed.
    ///
    /// A successful request will generate zero or more
    /// PARTIAL_RESPONSE Messages followed by exactly one RESPONSE
    /// Message. Once the final RESPONSE Message has been received
    /// the specified 'identity' will have been updated to contain
    /// the users entitlement information and the CorrelationId
    /// associated with the request may be re-used. If the request
    /// fails at any stage a REQUEST_STATUS will be generated, the
    /// specified 'identity' will not be modified and the
    /// CorrelationId may be re-used.
    ///
    /// The 'identity' supplied must have been returned from this
    /// Session's 'createIdentity()' method.
    pub fn send_authorization_request(
        &mut self,
        request: &Request,
        identity: &Identity,
        correlation_id: Option<CorrelationId>,
        event_queue: Option<&EventQueue>
    ) -> Result<CorrelationId, Error> {
        let mut correlation_id = correlation_id.unwrap_or_else(|| CorrelationId::new_empty());
        let event_queue = event_queue.map_or(ptr::null_mut(), |event_queue| event_queue.0);
        let request_label = ptr::null_mut();
        let request_label_len = 0;
        let res = unsafe {
            blpapi_Session_sendAuthorizationRequest(
                self.ptr,
                request.ptr,
                identity.0,
                &mut correlation_id.0,
                event_queue,
                request_label,
                request_label_len,
            )
        };
        Error::check(res)?;

        Ok(correlation_id)
    }
}

impl Drop for Session<'_> {
    fn drop(&mut self) {
        unsafe { blpapi_Session_destroy(self.ptr) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_request() -> Result<(), Error> {
        let session_options = SessionOptions::default()
            .with_server_host("localhost")?
            .with_server_port(8194)?
        ;

        let session = Session::create(
            session_options,
            None::<fn (&Event) -> ()>,
            None
        );

        //session.start()?;
        //session.open_service("//blp/refdata")?;

        Ok(())
    }
}
