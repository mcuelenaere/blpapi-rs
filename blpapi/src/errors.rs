use blpapi_sys::*;
use std::ffi::CStr;

#[derive(Debug)]
pub enum BlpApiError {
    // Specific errors
    IllegalArg(String),
    IllegalAccess(String),
    InvalidSession(String),
    DuplicateCorrelationID(String),
    InternalError(String),
    ResolveFailed(String),
    ConnectFailed(String),
    IllegalState(String),
    CodecFailure(String),
    IndexOutOfRange(String),
    InvalidConversion(String),
    ItemNotFound(String),
    IoError(String),
    CorrelationNotFound(String),
    ServiceNotFound(String),
    LogonLookupFailed(String),
    DsLookupFailed(String),
    UnsupportedOperation(String),
    DsPropertyNotFound(String),
    MsgTooLarge(String),
    // Class errors
    InvalidStateClassError(String),
    InvalidArgumentClassError(String),
    InvalidConversionClassError(String),
    IndexOutOfRangeClassError(String),
    FieldNotFoundClassError(String),
    UnsupportedOperationClassError(String),
    NotFoundClassError(String),
    UnknownClassError(String),
}

impl BlpApiError {
    pub(crate) fn from_code(error_code: u32) -> Self {
        let error_msg_ptr = unsafe { blpapi_getLastErrorDescription(error_code as i32) };
        let error_msg = if error_msg_ptr.is_null() {
            "Unknown".to_string()
        } else {
            unsafe { CStr::from_ptr(error_msg_ptr) }.to_string_lossy().to_string()
        };

        match error_code {
            BLPAPI_ERROR_ILLEGAL_ARG => BlpApiError::IllegalArg(error_msg),
            BLPAPI_ERROR_ILLEGAL_ACCESS => BlpApiError::IllegalAccess(error_msg),
            BLPAPI_ERROR_INVALID_SESSION => BlpApiError::InvalidSession(error_msg),
            BLPAPI_ERROR_DUPLICATE_CORRELATIONID => BlpApiError::DuplicateCorrelationID(error_msg),
            BLPAPI_ERROR_INTERNAL_ERROR => BlpApiError::InternalError(error_msg),
            BLPAPI_ERROR_RESOLVE_FAILED => BlpApiError::ResolveFailed(error_msg),
            BLPAPI_ERROR_CONNECT_FAILED => BlpApiError::ConnectFailed(error_msg),
            BLPAPI_ERROR_ILLEGAL_STATE => BlpApiError::IllegalState(error_msg),
            BLPAPI_ERROR_CODEC_FAILURE => BlpApiError::CodecFailure(error_msg),
            BLPAPI_ERROR_INDEX_OUT_OF_RANGE => BlpApiError::IndexOutOfRange(error_msg),
            BLPAPI_ERROR_INVALID_CONVERSION => BlpApiError::InvalidConversion(error_msg),
            BLPAPI_ERROR_ITEM_NOT_FOUND => BlpApiError::ItemNotFound(error_msg),
            BLPAPI_ERROR_IO_ERROR => BlpApiError::IoError(error_msg),
            BLPAPI_ERROR_CORRELATION_NOT_FOUND => BlpApiError::CorrelationNotFound(error_msg),
            BLPAPI_ERROR_SERVICE_NOT_FOUND => BlpApiError::ServiceNotFound(error_msg),
            BLPAPI_ERROR_LOGON_LOOKUP_FAILED => BlpApiError::LogonLookupFailed(error_msg),
            BLPAPI_ERROR_DS_LOOKUP_FAILED => BlpApiError::DsLookupFailed(error_msg),
            BLPAPI_ERROR_UNSUPPORTED_OPERATION => BlpApiError::UnsupportedOperation(error_msg),
            BLPAPI_ERROR_DS_PROPERTY_NOT_FOUND => BlpApiError::DsPropertyNotFound(error_msg),
            BLPAPI_ERROR_MSG_TOO_LARGE => BlpApiError::MsgTooLarge(error_msg),
            _ => {
                match error_code & 0xff0000 {
                    BLPAPI_INVALIDSTATE_CLASS => BlpApiError::InvalidStateClassError(error_msg),
                    BLPAPI_INVALIDARG_CLASS => BlpApiError::InvalidArgumentClassError(error_msg),
                    BLPAPI_CNVERROR_CLASS => BlpApiError::InvalidConversionClassError(error_msg),
                    BLPAPI_BOUNDSERROR_CLASS => BlpApiError::IndexOutOfRangeClassError(error_msg),
                    BLPAPI_FLDNOTFOUND_CLASS => BlpApiError::FieldNotFoundClassError(error_msg),
                    BLPAPI_UNSUPPORTED_CLASS => BlpApiError::UnsupportedOperationClassError(error_msg),
                    BLPAPI_NOTFOUND_CLASS => BlpApiError::NotFoundClassError(error_msg),
                    _ => BlpApiError::UnknownClassError(error_msg),
                }
            }
        }
    }
}

/// Error converted from `c_int`
#[derive(Debug)]
pub enum Error {
    /// Generic blpapi error return
    Generic(i32),
    /// Some element were not found
    NotFound(String),
    /// Timeout event
    TimeOut,
    StringConversionError(Box<dyn std::error::Error>),
    BlpApiError(BlpApiError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl Error {
    /// Check if response is an error(!=0)
    pub(crate) fn check(res: i32) -> Result<(), Error> {
        if res == 0 {
            Ok(())
        } else {
            Err(Error::BlpApiError(BlpApiError::from_code(res as u32)))
        }
    }
}
