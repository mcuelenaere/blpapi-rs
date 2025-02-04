use crate::Error;
use crate::tls_options::TlsOptions;
use blpapi_sys::*;
use std::ffi::{CStr, CString};
use std::os::raw::c_int;

/// A SessionOptions
///
/// Behaves like a `Session` builder
pub struct SessionOptions(pub(crate) *mut blpapi_SessionOptions_t);

impl SessionOptions {
    /// Get client mode
    pub fn client_mode(&self) -> Result<ClientMode, Error> {
        let mode = unsafe { blpapi_SessionOptions_clientMode(self.0) };
        Error::check(mode)?;
        match mode as u32 {
            BLPAPI_CLIENTMODE_AUTO => Ok(ClientMode::Auto),
            BLPAPI_CLIENTMODE_DAPI => Ok(ClientMode::DApi),
            BLPAPI_CLIENTMODE_SAPI => Ok(ClientMode::SApi),
            _ => Err(Error::Generic(mode)),
        }
    }

    /// Set client mode
    pub fn set_client_mode(&mut self, mode: ClientMode) {
        let mode = match mode {
            ClientMode::Auto => BLPAPI_CLIENTMODE_AUTO,
            ClientMode::DApi => BLPAPI_CLIENTMODE_DAPI,
            ClientMode::SApi => BLPAPI_CLIENTMODE_SAPI,
        };
        unsafe {
            blpapi_SessionOptions_setClientMode(self.0, mode as c_int);
        }
    }

    /// Get server host
    pub fn server_host(&self) -> String {
        let chost = unsafe { CStr::from_ptr(blpapi_SessionOptions_serverHost(self.0)) };
        chost.to_string_lossy().into_owned()
    }

    /// Set server host
    pub fn with_server_host(self, host: &str) -> Result<Self, Error> {
        let chost = CString::new(host).unwrap();
        let res = unsafe { blpapi_SessionOptions_setServerHost(self.0, chost.as_ptr()) };
        Error::check(res)?;
        Ok(self)
    }

    /// Get server port
    pub fn server_port(&self) -> u16 {
        unsafe { blpapi_SessionOptions_serverPort(self.0) as u16 }
    }

    /// Set server port
    pub fn with_server_port(self, port: u16) -> Result<Self, Error> {
        let res = unsafe { blpapi_SessionOptions_setServerPort(self.0, port) };
        Error::check(res)?;
        Ok(self)
    }

    /// Set TLS options
    pub fn with_tls_options(self, tls_options: &TlsOptions) -> Self {
        unsafe { blpapi_SessionOptions_setTlsOptions(self.0, tls_options.0) }
        self
    }

    /// Set authentication options
    pub fn with_authentication_options(self, auth_options: &str) -> Self {
        let auth_options = CString::new(auth_options).unwrap();
        unsafe { blpapi_SessionOptions_setAuthenticationOptions(self.0, auth_options.as_ptr()) };
        self
    }
}

impl Drop for SessionOptions {
    fn drop(&mut self) {
        unsafe { blpapi_SessionOptions_destroy(self.0) }
    }
}

impl Clone for SessionOptions {
    fn clone(&self) -> Self {
        let cloned = SessionOptions::default();
        unsafe {
            blpapi_SessionOptions_copy(self.0, cloned.0);
        }
        cloned
    }
}

impl Default for SessionOptions {
    fn default() -> Self {
        unsafe { SessionOptions(blpapi_SessionOptions_create()) }
    }
}

unsafe impl Send for SessionOptions {}
unsafe impl Sync for SessionOptions {}

/// ClientMode
#[derive(Debug, Clone, Copy)]
pub enum ClientMode {
    /// Automatic
    Auto,
    /// Desktop API
    DApi,
    /// Server API
    SApi,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_host() {
        let host = "localhost";
        let options = SessionOptions::default().with_server_host(host).unwrap();
        assert_eq!(host, options.server_host());
    }
}
