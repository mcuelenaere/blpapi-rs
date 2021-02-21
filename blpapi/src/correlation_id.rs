use blpapi_sys::*;
use std::os::raw::c_uint;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialOrd, PartialEq)]
pub enum CorrelationType {
    Unset,
    Int,
    Pointer,
    Autogen,
}

impl From<u32> for CorrelationType {
    fn from(correlation_type: u32) -> Self {
        match correlation_type {
            blpapi_sys::BLPAPI_CORRELATION_TYPE_UNSET => CorrelationType::Unset,
            blpapi_sys::BLPAPI_CORRELATION_TYPE_INT => CorrelationType::Int,
            blpapi_sys::BLPAPI_CORRELATION_TYPE_POINTER => CorrelationType::Pointer,
            blpapi_sys::BLPAPI_CORRELATION_TYPE_AUTOGEN => CorrelationType::Autogen,
            _ => CorrelationType::Unset,
        }
    }
}

/// A Correlation Id
pub struct CorrelationId(pub(crate) blpapi_CorrelationId_t);

impl CorrelationId {
    pub fn new_empty() -> Self {
        let inner = blpapi_CorrelationId_t_::default();
        CorrelationId(inner)
    }

    pub fn new_int(value: u64, class_id: Option<usize>) -> Self {
        let mut inner = blpapi_CorrelationId_t_::default();
        inner.set_size(std::mem::size_of::<blpapi_CorrelationId_t>() as c_uint);
        inner.set_valueType(BLPAPI_CORRELATION_TYPE_INT);
        inner.set_classId(class_id.unwrap_or(0) as c_uint);
        inner.value.intValue = value;

        CorrelationId(inner)
    }

    pub fn value_type(&self) -> CorrelationType {
        CorrelationType::from(self.0.valueType())
    }
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

impl PartialEq for CorrelationId {
    fn eq(&self, other: &Self) -> bool {
        if self.0.valueType() != other.0.valueType() {
            return false;
        }

        if self.0.classId() != other.0.classId() {
            return false;
        }

        unsafe {
            if self.0.valueType() == BLPAPI_CORRELATION_TYPE_POINTER {
                if self.0.value.ptrValue.pointer != other.0.value.ptrValue.pointer {
                    return false;
                }
            } else if self.0.value.intValue != other.0.value.intValue {
                return false;
            }
        }

        return true;
    }
}

impl Eq for CorrelationId {}

impl Hash for CorrelationId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.0.valueType());
        state.write_u32(self.0.classId());
        unsafe {
            if self.0.valueType() == BLPAPI_CORRELATION_TYPE_POINTER {
                state.write_usize(self.0.value.ptrValue.pointer as usize);
            } else {
                state.write_u64(self.0.value.intValue);
            }
        }
    }
}

impl Clone for CorrelationId {
    fn clone(&self) -> Self {
        // TODO: if type is pointer, we should do some extra magic
        Self(self.0)
    }
}

unsafe impl Send for CorrelationId {}
unsafe impl Sync for CorrelationId {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlation_u64() {
        let id = CorrelationId::new_int(1, None);
        assert_eq!(unsafe { id.0.value.intValue }, 1);
    }
}
