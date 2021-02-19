use blpapi_sys::*;
use std::os::raw::c_uint;
use std::fmt::{Debug, Formatter};

const DEFAULT_CLASS_ID: c_uint = 0;

/// A Correlation Id
pub struct CorrelationId(pub(crate) blpapi_CorrelationId_t);

impl CorrelationId {
    pub fn new_u64(value: u64) -> Self {
        let mut inner = blpapi_CorrelationId_t_::default();
        inner.set_size(std::mem::size_of::<blpapi_CorrelationId_t>() as c_uint);
        inner.set_valueType(BLPAPI_CORRELATION_TYPE_INT);
        inner.set_classId(DEFAULT_CLASS_ID);
        inner.value.intValue = value;

        CorrelationId(inner)
    }

impl Debug for CorrelationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value_type = match self.0.valueType() {
            BLPAPI_CORRELATION_TYPE_UNSET => "UNSET",
            BLPAPI_CORRELATION_TYPE_INT => "INT",
            BLPAPI_CORRELATION_TYPE_POINTER => "POINTER",
            BLPAPI_CORRELATION_TYPE_AUTOGEN => "AUTOGEN",
            _ => "UNKNOWN",
        };

        f.write_fmt(format_args!("CorrelationId[valueType={} classId={} value=", value_type, self.0.classId()))?;

        if self.0.valueType() == BLPAPI_CORRELATION_TYPE_POINTER {
            f.write_fmt(format_args!("{:?}", unsafe { self.0.value.ptrValue.pointer }))?;
        } else {
            f.write_fmt(format_args!("{}", unsafe { self.0.value.intValue }))?;
        }
        f.write_str("]")
    }
}

#[test]
fn correlation_u64() {
    let id = CorrelationId::new_u64(1);
    assert_eq!(unsafe { id.0.value.intValue }, 1);
}
