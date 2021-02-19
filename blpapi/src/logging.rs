use blpapi_sys::*;
use crate::datetime::Datetime;
use crate::errors::Error;
use std::os::raw::{c_int, c_char};
use std::ffi::CStr;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoggingSeverity {
    OFF,
    FATAL,
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE
}

impl Into<blpapi_Logging_Severity_t> for LoggingSeverity {
    fn into(self) -> blpapi_Logging_Severity_t {
        match self {
            LoggingSeverity::OFF => blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_OFF,
            LoggingSeverity::FATAL => blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_FATAL,
            LoggingSeverity::ERROR => blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_ERROR,
            LoggingSeverity::WARN => blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_WARN,
            LoggingSeverity::INFO => blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_INFO,
            LoggingSeverity::DEBUG => blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_DEBUG,
            LoggingSeverity::TRACE => blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_TRACE,
        }
    }
}

impl From<blpapi_Logging_Severity_t> for LoggingSeverity {
    #![allow(non_upper_case_globals)]
    fn from(severity: blpapi_Logging_Severity_t) -> Self {
        match severity {
            blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_OFF => LoggingSeverity::OFF,
            blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_FATAL => LoggingSeverity::FATAL,
            blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_ERROR => LoggingSeverity::ERROR,
            blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_WARN => LoggingSeverity::WARN,
            blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_INFO => LoggingSeverity::INFO,
            blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_DEBUG => LoggingSeverity::DEBUG,
            blpapi_Logging_Severity_t_blpapi_Logging_SEVERITY_TRACE => LoggingSeverity::TRACE,
            _ => panic!(format!("invalid severity: {}", severity))
        }
    }
}

static mut RUST_CALLBACK: Option<Box<dyn Fn(u64, LoggingSeverity, Datetime, &str, &str) + 'static>> = None;

type LoggingFunc = unsafe extern "C" fn(thread_id: blpapi_UInt64_t, severity: c_int, timestamp: blpapi_Datetime_t, category: *const c_char, message: *const c_char);
unsafe extern "C" fn c_callback(thread_id: blpapi_UInt64_t, severity: c_int, timestamp: blpapi_Datetime_t, category: *const c_char, message: *const c_char) {
    match RUST_CALLBACK.as_ref() {
        Some(callback) => {
            let category = CStr::from_ptr(category).to_str().unwrap();
            let message = CStr::from_ptr(message).to_str().unwrap();
            if let Err(err) = catch_unwind(AssertUnwindSafe(|| {
                callback(thread_id, LoggingSeverity::from(severity as blpapi_Logging_Severity_t), Datetime(timestamp), category, message);
            })) {
                eprintln!("{:?}", err);
                std::process::abort();
            }
        },
        None => {},
    }
}

/// Register the specified 'callback' that will be called for all log
/// messages with severity greater than or equal to the specified
/// 'thresholdSeverity'.  The callback needs to be registered before the
/// start of all sessions.  If this function is called multiple times, only
/// the last registered callback will take effect.  Registering with a
/// 'None' callback will de-register the callback.
/// '0' is returned if callback is registered and a non-zero otherwise.
pub fn register_callback(callback: Option<impl Fn(u64, LoggingSeverity, Datetime, &str, &str) + 'static>, threshold_severity: LoggingSeverity) -> Result<(), Error> {
    let res = unsafe {
        let c_callback = callback.as_ref().and(Some(c_callback as LoggingFunc));
        RUST_CALLBACK = callback.map(|cb| Box::new(cb) as _);
        blpapi_Logging_registerCallback(c_callback, threshold_severity.into())
    };
    Error::check(res)
}