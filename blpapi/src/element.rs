use crate::{datetime::Datetime, name::Name, Error};
use blpapi_sys::*;
use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    os::raw::c_int,
    ptr,
};
use std::fmt::{Display, Debug, Formatter};

#[derive(Debug, PartialEq)]
pub enum DataType {
    /// Bool
    Bool,
    /// Char
    Char,
    /// Unsigned 8 bit value
    Byte,
    /// 32 bit Integer
    Int32,
    /// 64 bit Integer
    Int64,
    /// 32 bit Floating point - IEEE
    Float32,
    /// 64 bit Floating point - IEEE
    Float64,
    /// ASCIIZ string
    String,
    /// Opaque binary data
    ByteArray,
    /// Date
    Date,
    /// Timestamp
    Time,
    ///
    Decimal,
    /// Date and time
    DateTime,
    /// An opaque enumeration
    Enumeration,
    /// Sequence type
    Sequence,
    /// Choice type
    Choice,
    /// Used for some internal messages
    CorrelationId,
}

impl From<blpapi_DataType_t> for DataType {
    fn from(data_type: blpapi_DataType_t) -> Self {
        match data_type {
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_BOOL => DataType::Bool,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_CHAR => DataType::Char,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_BYTE => DataType::Byte,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_INT32 => DataType::Int32,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_INT64 => DataType::Int64,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_FLOAT32 => DataType::Float32,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_FLOAT64 => DataType::Float64,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_STRING => DataType::String,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_BYTEARRAY => DataType::ByteArray,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_DATE => DataType::Date,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_TIME => DataType::Time,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_DECIMAL => DataType::Decimal,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_DATETIME => DataType::DateTime,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_ENUMERATION => DataType::Enumeration,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_SEQUENCE => DataType::Sequence,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_CHOICE => DataType::Choice,
            blpapi_sys::blpapi_DataType_t_BLPAPI_DATATYPE_CORRELATION_ID => DataType::CorrelationId,
            _ => panic!("unsupported data type"),
        }
    }
}

/// An element
#[derive(Clone)]
pub struct Element<'a> {
    pub(crate) ptr: *mut blpapi_Element_t,
    pub(crate) _marker: PhantomData<&'a ()>,
}

impl Element<'_> {
    /// name
    pub fn string_name(&self) -> String {
        self.name().to_string_lossy()
    }

    /// name
    pub fn name(&self) -> Name {
        let name = unsafe { blpapi_Element_name(self.ptr) };
        Name(name)
    }

    /// Data type
    pub fn data_type(&self) -> DataType {
        let data_type: blpapi_DataType_t = unsafe { blpapi_Element_datatype(self.ptr) as u32 };
        DataType::from(data_type)
    }

    /// Has element
    pub fn has_element(&self, name: &str, exclude_null_elements: bool) -> bool {
        let name = CString::new(name).unwrap();
        let named = ptr::null();
        if exclude_null_elements {
            unsafe { blpapi_Element_hasElementEx(self.ptr, name.as_ptr(), named, exclude_null_elements as i32, 0) != 0 }
        } else {
            unsafe { blpapi_Element_hasElement(self.ptr, name.as_ptr(), named) != 0 }
        }
    }

    /// Has element
    pub fn has_named_element(&self, named: &Name, exclude_null_elements: bool) -> bool {
        let name = ptr::null();
        if exclude_null_elements {
            unsafe { blpapi_Element_hasElementEx(self.ptr, name, named.0, exclude_null_elements as i32, 0) != 0 }
        } else {
            unsafe { blpapi_Element_hasElement(self.ptr, name, named.0) != 0 }
        }
    }

    /// Number of values
    pub fn num_values(&self) -> usize {
        unsafe { blpapi_Element_numValues(self.ptr) }
    }

    /// Number of elements
    pub fn num_elements(&self) -> usize {
        unsafe { blpapi_Element_numElements(self.ptr) }
    }

    /// Get element from its name
    pub fn get_element(&self, name: &str) -> Result<Element, Error> {
        let mut element = ptr::null_mut();
        let name = CString::new(name).unwrap();
        let res = unsafe {
            blpapi_Element_getElement(
                self.ptr,
                &mut element,
                name.as_ptr(),
                ptr::null(),
            )
        };
        Error::check(res)?;

        Ok(Element { ptr: element, _marker: PhantomData })
    }

    /// Get element from its name
    pub fn get_named_element(&self, named_element: &Name) -> Result<Element, Error> {
        let mut element = ptr::null_mut();
        let res = unsafe {
            blpapi_Element_getElement(
                self.ptr,
                &mut element,
                ptr::null(),
                named_element.0,
            )
        };
        Error::check(res)?;

        Ok(Element { ptr: element, _marker: PhantomData })
    }

    /// Get element at index
    pub fn get_element_at(&self, index: usize) -> Result<Element, Error> {
        let mut element = ptr::null_mut();
        let res = unsafe { blpapi_Element_getElementAt(self.ptr, &mut element, index) };
        Error::check(res)?;

        Ok(Element { ptr: element, _marker: PhantomData })
    }

    /// Append a new element
    pub fn append_element(&mut self) -> Result<Element, Error> {
        unsafe {
            let mut ptr = ptr::null_mut();
            Error::check(blpapi_Element_appendElement(self.ptr, &mut ptr as *mut _))?;
            Ok(Element { ptr, _marker: PhantomData })
        }
    }

    /// Append a new element with `value`
    pub fn append<V: SetValue>(&mut self, value: V) -> Result<(), Error> {
        value.append_to(self)
    }

    /// Return true if the value of the sub-element at the specified
    /// 'position' in a sequence or choice element is a null value. An
    /// error is returned if 'position >= numElements()'.
    pub fn is_null_value(&self, index: usize) -> Result<bool, Error> {
        let res = unsafe { blpapi_Element_isNullValue(self.ptr, index) };
        if res != 0 && res != 1 {
            Error::check(res)?
        }

        Ok(res != 0)
    }

    /// Return true if this element has a null value, and false otherwise.
    pub fn is_null(&self) -> Result<bool, Error> {
        let res = unsafe { blpapi_Element_isNull(self.ptr) };
        if res != 0 && res != 1 {
            Error::check(res)?
        }

        Ok(res != 0)
    }

    /// Get value at given index
    pub fn get_at<'e, V: GetValue<'e>>(&'e self, index: usize) -> Result<V, Error> {
        V::get_at(self, index)
    }

    /// Set value at given index
    pub fn set_at<V: SetValue>(&mut self, index: usize, value: V) -> Result<(), Error> {
        value.set_at(self, index)
    }

    /// Set value from element defined by name
    pub fn set<V: SetValue>(&mut self, name: &str, value: V) -> Result<(), Error> {
        value.set(self, name)
    }

    /// Set value from named element
    pub fn set_named<V: SetValue>(&mut self, name: &Name, value: V) -> Result<(), Error> {
        value.set_named(self, name)
    }

    /// Get current element value (index at 0)
    pub fn value<'e, V: GetValue<'e>>(&'e self) -> Result<V, Error> {
        self.get_at(0)
    }

    /// Get an iterator over the values
    pub fn values<'e, V: GetValue<'e>>(&'e self) -> Values<V> {
        Values {
            len: self.num_values(),
            element: self,
            i: 0,
            _phantom: PhantomData,
        }
    }

    /// Get an iterator over the elements
    pub fn elements(&self) -> Elements {
        Elements {
            len: self.num_elements(),
            element: self,
            i: 0,
        }
    }

    /// Return true if 'elementDefinition().maxValues() > 1' or
    /// 'elementDefinition().maxValues() == UNBOUNDED', and false otherwise.
    pub fn is_array(&self) -> bool {
        let res = unsafe { blpapi_Element_isArray(self.ptr) };
        res != 0
    }

    /// Return true if the DataType is either SEQUENCE or CHOICE and the
    /// element is not an array element.  Return false otherwise.
    pub fn is_complex_type(&self) -> bool {
        let res = unsafe { blpapi_Element_isComplexType(self.ptr) };
        res != 0
    }

    /// Format this Element to the specified output 'stream' at the
    /// (absolute value of) the optionally specified indentation 'level' and
    /// return a reference to 'stream'. If 'level' is specified, optionally
    /// specify 'spacesPerLevel', the number of spaces per indentation level
    /// for this and all of its nested objects. If 'level' is negative,
    /// suppress indentation of the first line. If 'spacesPerLevel' is
    /// negative, format the entire output on one line, suppressing all but
    /// the initial indentation (as governed by 'level').
    pub fn print(&self, f: &mut Formatter<'_>, level: isize, spaces_per_level: isize) -> Result<(), Error> {
        let res = unsafe {
            let stream = std::mem::transmute(f);
            blpapi_Element_print(
                self.ptr,
                Some(crate::utils::stream_writer),
                stream,
                level as c_int,
                spaces_per_level as c_int
            )
        };
        Error::check(res)
    }
}

impl Debug for Element<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Element[name={:?} data_type={:?}]", self.name(), self.data_type()))
    }
}

impl Display for Element<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.print(f, 0, 4).map_err(|_| std::fmt::Error)
    }
}

unsafe impl Send for Element<'_> {}
unsafe impl Sync for Element<'_> {}

/// A trait to represent an Element value
pub trait GetValue<'e>: Sized {
    /// Get value from elements by index
    fn get_at(element: &'e Element, index: usize) -> Result<Self, Error>;
}

/// A trait to represent an Element value
pub trait SetValue: Sized {
    /// Set value from elements at index
    fn set_at(self, element: &mut Element, index: usize) -> Result<(), Error>;
    /// Set value from element at name
    fn set(self, element: &mut Element, name: &str) -> Result<(), Error>;
    /// Set value from element at name
    fn set_named(self, element: &mut Element, name: &Name) -> Result<(), Error>;

    /// Append a new value to a element
    ///
    /// Return an error if the element doesn't accept appending
    fn append_to(self, element: &mut Element) -> Result<(), Error> {
        Self::set_at(self, element, BLPAPI_ELEMENT_INDEX_END as usize)
    }
}

macro_rules! impl_value {
    ($ty:ty, $start:expr, $get_at:path, $set_at:path, $set:path) => {
        impl<'e> GetValue<'e> for $ty {
            fn get_at(element: &'e Element, index: usize) -> Result<Self, Error> {
                let mut tmp = $start;
                let res = unsafe { $get_at(element.ptr, &mut tmp as *mut _, index) };
                Error::check(res)?;

                Ok(tmp)
            }
        }

        impl SetValue for $ty {
            fn set_at(self, element: &mut Element, index: usize) -> Result<(), Error> {
                unsafe {
                    let res = $set_at(element.ptr, self, index);
                    Error::check(res)
                }
            }
            fn set(self, element: &mut Element, name: &str) -> Result<(), Error> {
                unsafe {
                    let named_element = ptr::null();
                    let name = CString::new(name).unwrap();
                    let res = $set(element.ptr, name.as_ptr(), named_element, self);
                    Error::check(res)
                }
            }
            fn set_named(self, element: &mut Element, named_element: &Name) -> Result<(), Error> {
                unsafe {
                    let name = ptr::null();
                    let res = $set(element.ptr, name, named_element.0, self);
                    Error::check(res)
                }
            }
        }
    };
    ($ty:ty, $get_at:path, $set_at:path, $set:path, $from_bbg: expr, $to_bbg: expr) => {
        impl<'e> GetValue<'e> for $ty {
            fn get_at(element: &'e Element, index: usize) -> Result<Self, Error> {
                let tmp = ptr::null_mut();
                let res = unsafe { $get_at(element.ptr, tmp, index) };
                Error::check(res)?;

                Ok($from_bbg(unsafe { *tmp }))
            }
        }

        impl SetValue for $ty {
            fn set_at(self, element: &mut Element, index: usize) -> Result<(), Error> {
                unsafe {
                    let res = $set_at(element.ptr, $to_bbg(self), index);
                    Error::check(res)
                }
            }
            fn set(self, element: &mut Element, name: &str) -> Result<(), Error> {
                unsafe {
                    let named_element = ptr::null();
                    let name = CString::new(name).unwrap();
                    let res = $set(element.ptr, name.as_ptr(), named_element, $to_bbg(self));
                    Error::check(res)
                }
            }
            fn set_named(self, element: &mut Element, named_element: &Name) -> Result<(), Error> {
                unsafe {
                    let name = ptr::null();
                    let res = $set(element.ptr, name, named_element.0, $to_bbg(self));
                    Error::check(res)
                }
            }
        }
    };
}

impl_value!(
    i64,
    0,
    blpapi_Element_getValueAsInt64,
    blpapi_Element_setValueInt64,
    blpapi_Element_setElementInt64
);
impl_value!(
    i32,
    0,
    blpapi_Element_getValueAsInt32,
    blpapi_Element_setValueInt32,
    blpapi_Element_setElementInt32
);
impl_value!(
    f64,
    0.,
    blpapi_Element_getValueAsFloat64,
    blpapi_Element_setValueFloat64,
    blpapi_Element_setElementFloat64
);
impl_value!(
    f32,
    0.,
    blpapi_Element_getValueAsFloat32,
    blpapi_Element_setValueFloat32,
    blpapi_Element_setElementFloat32
);
impl_value!(
    i8,
    0,
    blpapi_Element_getValueAsChar,
    blpapi_Element_setValueChar,
    blpapi_Element_setElementChar
);
impl_value!(
    bool,
    blpapi_Element_getValueAsBool,
    blpapi_Element_setValueBool,
    blpapi_Element_setElementBool,
    |bbg: blpapi_Bool_t| bbg != 0,
    |rust| if rust { 1 } else { 0 }
);
impl_value!(
    Name,
    blpapi_Element_getValueAsName,
    blpapi_Element_setValueFromName,
    blpapi_Element_setElementFromName,
    |bbg: *mut blpapi_Name_t| Name(bbg),
    |rust: Name| rust.0
);

impl<'e> GetValue<'e> for String {
    fn get_at(element: &'e Element, index: usize) -> Result<Self, Error> {
        let mut tmp = ptr::null();
        let res = unsafe { blpapi_Element_getValueAsString(element.ptr, &mut tmp, index) };
        Error::check(res)?;

        let str = unsafe { CStr::from_ptr(tmp) };
        Ok(str.to_string_lossy().into_owned())
    }
}

impl<'e> GetValue<'e> for &'e CStr {
    fn get_at(element: &'e Element, index: usize) -> Result<Self, Error> {
        let mut tmp = ptr::null();
        let res = unsafe { blpapi_Element_getValueAsString(element.ptr, &mut tmp, index) };
        Error::check(res)?;

        let str = unsafe { CStr::from_ptr(tmp) };
        Ok(str)
    }
}

impl<'a> SetValue for &'a str {
    fn set_at(self, element: &mut Element, index: usize) -> Result<(), Error> {
        let value = CString::new(self).unwrap();
        unsafe {
            let res = blpapi_Element_setValueString(element.ptr, value.as_ptr(), index);
            Error::check(res)
        }
    }
    fn set(self, element: &mut Element, name: &str) -> Result<(), Error> {
        let value = CString::new(self).unwrap();
        unsafe {
            let named_element = ptr::null();
            let name = CString::new(name).unwrap();
            let res = blpapi_Element_setElementString(
                element.ptr,
                name.as_ptr(),
                named_element,
                value.as_ptr(),
            );
            Error::check(res)
        }
    }
    fn set_named(self, element: &mut Element, named_element: &Name) -> Result<(), Error> {
        let value = CString::new(self).unwrap();
        unsafe {
            let name = ptr::null();
            let res =
                blpapi_Element_setElementString(element.ptr, name, named_element.0, value.as_ptr());
            Error::check(res)
        }
    }
}

impl<'e> GetValue<'e> for Datetime {
    fn get_at(element: &'e Element, index: usize) -> Result<Self, Error> {
        let mut tmp = Datetime::default();
        let res = unsafe { blpapi_Element_getValueAsDatetime(element.ptr, &mut tmp.0, index) };
        Error::check(res)?;

        Ok(tmp)
    }
}

impl<'e, T: GetValue<'e>> GetValue<'e> for Option<T> {
    fn get_at(element: &'e Element, index: usize) -> Result<Self, Error> {
        T::get_at(element, index).map(Some)
    }
}

impl<'e, T: GetValue<'e>> GetValue<'e> for Vec<T> {
    fn get_at(element: &'e Element, index: usize) -> Result<Self, Error> {
        Ok(element.values::<T>().skip(index).collect())
    }
}

impl<'e> GetValue<'e> for Element<'e> {
    fn get_at(element: &'e Element, index: usize) -> Result<Self, Error> {
        let mut ptr = ptr::null_mut();
        let res = unsafe { blpapi_Element_getValueAsElement(element.ptr, &mut ptr, index) };
        Error::check(res)?;

        Ok(Element { ptr, _marker: PhantomData })
    }
}

impl<'e, T: GetValue<'e> + std::hash::Hash + Eq> GetValue<'e> for std::collections::HashSet<T> {
    fn get_at(element: &'e Element, index: usize) -> Result<Self, Error> {
        Ok(element.values::<T>().skip(index).collect())
    }
}

impl<'a> SetValue for &'a Datetime {
    fn set_at(self, element: &mut Element, index: usize) -> Result<(), Error> {
        unsafe {
            let res = blpapi_Element_setValueDatetime(element.ptr, &self.0 as *const _, index);
            Error::check(res)
        }
    }
    fn set(self, element: &mut Element, name: &str) -> Result<(), Error> {
        unsafe {
            let named_element = ptr::null();
            let name = CString::new(name).unwrap();
            let res = blpapi_Element_setElementDatetime(
                element.ptr,
                name.as_ptr(),
                named_element,
                &self.0 as *const _,
            );
            Error::check(res)
        }
    }
    fn set_named(self, element: &mut Element, named_element: &Name) -> Result<(), Error> {
        unsafe {
            let name = ptr::null();
            let res = blpapi_Element_setElementDatetime(
                element.ptr,
                name,
                named_element.0,
                &self.0 as *const _,
            );
            Error::check(res)
        }
    }
}

/// An iterator over values
pub struct Values<'e, V> {
    element: &'e Element<'e>,
    i: usize,
    len: usize,
    _phantom: PhantomData<V>,
}

impl<'e, V: GetValue<'e>> Iterator for Values<'e, V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.len {
            return None;
        }
        let v = self.element.get_at::<V>(self.i);
        self.i += 1;
        v.ok()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len - self.i, Some(self.len - self.i))
    }
}

/// An iterator over elements
pub struct Elements<'e> {
    element: &'e Element<'e>,
    i: usize,
    len: usize,
}

impl<'e> Iterator for Elements<'e> {
    type Item = Element<'e>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.len {
            return None;
        }
        let v = self.element.get_element_at(self.i);
        self.i += 1;
        v.ok()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len - self.i, Some(self.len - self.i))
    }
}
