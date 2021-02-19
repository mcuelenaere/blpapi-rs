use blpapi_sys::*;
use std::ffi::{CString};

pub struct TlsOptions(pub(crate) *mut blpapi_TlsOptions_t);

impl TlsOptions {
    /// Creates a TlsOptions using a DER encoded client credentials in
    /// PKCS#12 format and DER encoded trust material in PKCS#7 format from
    /// the specified files.
    pub fn create_from_files(client_credentials_file_name: &str, client_credentials_password: &str, trusted_certificates_file_name: &str) -> Option<TlsOptions> {
        let client_credentials_file_name = CString::new(client_credentials_file_name).unwrap();
        let client_credentials_password = CString::new(client_credentials_password).unwrap();
        let trusted_certificates_file_name = CString::new(trusted_certificates_file_name).unwrap();
        let ret = unsafe { blpapi_TlsOptions_createFromFiles(
            client_credentials_file_name.as_ptr(),
            client_credentials_password.as_ptr(),
            trusted_certificates_file_name.as_ptr()
        ) };
        if ret.is_null() {
            None
        } else {
            Some(TlsOptions(ret))
        }
    }

    /// Create a TlsOptions using DER encoded client credentials in PKCS#12
    /// format and DER encoded trust material in PKCS#7 format from the
    /// specified raw data.
    pub fn create_from_blobs(client_credentials_raw_data: &[u8], client_credentials_password: &str, trusted_certificates_raw_data: &[u8]) -> Option<TlsOptions> {
        let client_credentials_password = CString::new(client_credentials_password).unwrap();
        let client_credentials_raw_data_ptr = client_credentials_raw_data.as_ptr();
        let client_credentials_raw_data_length = client_credentials_raw_data.len();
        let trusted_certificates_raw_data_ptr = trusted_certificates_raw_data.as_ptr();
        let trusted_certificates_raw_data_length = trusted_certificates_raw_data.len();
        let ret = unsafe { blpapi_TlsOptions_createFromBlobs(
            client_credentials_raw_data_ptr as *const i8,
            client_credentials_raw_data_length as i32,
            client_credentials_password.as_ptr(),
            trusted_certificates_raw_data_ptr as *const i8,
            trusted_certificates_raw_data_length as i32) };
        if ret.is_null() {
            None
        } else {
            Some(TlsOptions(ret))
        }
    }

    /// Set the TLS handshake timeout to the specified
    /// 'tls_handshake_timeout_ms'. The default is 10,000 milliseconds.
    /// The TLS handshake timeout will be set to the default if
    /// the specified 'tls_handshake_timeout_ms' is not positive.
    pub fn set_tls_handshake_timeout_ms(&mut self, tls_handshake_timeout_ms: i32) {
        unsafe { blpapi_TlsOptions_setTlsHandshakeTimeoutMs(self.0, tls_handshake_timeout_ms) }
    }

    /// Set the CRL fetch timeout to the specified
    /// 'crl_fetch_timeout_ms'. The default is 20,000 milliseconds.
    /// The TLS handshake timeout will be set to the default if
    /// the specified 'crl_fetch_timeout_ms' is not positive.
    pub fn set_crl_fetch_timeout_ms(&mut self, crl_fetch_timeout_ms: i32) {
        unsafe { blpapi_TlsOptions_setCrlFetchTimeoutMs(self.0, crl_fetch_timeout_ms) }
    }
}

impl Drop for TlsOptions {
    fn drop(&mut self) {
        unsafe { blpapi_TlsOptions_destroy(self.0) }
    }
}

impl Clone for TlsOptions {
    fn clone(&self) -> Self {
        let inner = unsafe { blpapi_TlsOptions_duplicate(self.0) };
        TlsOptions(inner)
    }
}

impl Default for TlsOptions {
    fn default() -> Self {
        let inner = unsafe { blpapi_TlsOptions_create() };
        TlsOptions(inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_tls_handshake_timeout_ms() {
        TlsOptions::default().set_tls_handshake_timeout_ms(5000);
    }
}
