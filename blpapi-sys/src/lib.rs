#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[test]
fn test_historical() {
    use std::ffi::CString;

    unsafe {
        let session_options = blpapi_SessionOptions_create();
        assert!(!session_options.is_null());

        let host = CString::new("localhost").unwrap();
        let res = blpapi_SessionOptions_setServerHost(session_options, host.as_ptr());
        assert_eq!(2, res, "{}", res);

        blpapi_SessionOptions_destroy(session_options);
    }
}
