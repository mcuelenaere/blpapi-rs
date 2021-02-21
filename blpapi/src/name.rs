use blpapi_sys::*;
use std::ffi::{CStr, CString};
use std::cmp::Ordering;
use std::ops::Deref;
use std::hash::{Hash, Hasher};
use std::string::ToString;
use std::fmt::Debug;

// NOTE: blpapi_Name_duplicate() and blpapi_Name_destroy() are no-ops, so we can safely
// implement Copy.

// NOTE: blpapi_Name_duplicate() and blpapi_Name_destroy() are no-ops, so we can safely
// implement Copy.

/// A `Name`
#[derive(Copy, Clone)]
pub struct Name(pub(crate) *mut blpapi_Name_t);

impl Name {
    /// Construct a 'Name' from the specified 'name_string'. Note also that
    /// constructing a 'Name' is a relatively expensive operation. If a 'Name'
    /// will be used repeatedly it is preferable to create it once and re-use
    /// (or copy) the object.
    pub fn new(name_string: &str) -> Self {
        let name = CString::new(name_string).unwrap();
        let ptr = unsafe { blpapi_Name_create(name.as_ptr()) };
        Name(ptr)
    }

    /// If a 'Name' already exists which matches the specified
    /// 'name_string', then return a copy of that 'Name'; otherwise
    /// return None.
    pub fn find_name(name_string: &str) -> Option<Self> {
        let name = CString::new(name_string).unwrap();
        let ptr = unsafe { blpapi_Name_findName(name.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(Name(ptr))
        }
    }

    /// Name length
    pub fn len(&self) -> usize {
        unsafe { blpapi_Name_length(self.0) }
    }
}

impl Deref for Name {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let ptr = blpapi_Name_string(self.0);
            let len = blpapi_Name_length(self.0);
            let slice = std::slice::from_raw_parts(ptr as *const u8, len + 1);
            CStr::from_bytes_with_nul_unchecked(slice)
        }
    }
}

impl<S: AsRef<str>> PartialEq<S> for Name {
    fn eq(&self, other: &S) -> bool {
        let s = CString::new(other.as_ref()).unwrap();
        unsafe { blpapi_Name_equalsStr(self.0, s.as_ptr()) != 0 }
    }
}

impl PartialEq<Name> for Name {
    fn eq(&self, other: &Name) -> bool {
        self.0 == other.0
    }
}

impl Eq for Name {
}

impl PartialOrd<Name> for Name {
    fn partial_cmp(&self, other: &Name) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Hash for Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let addr = self.0 as usize;
        state.write_usize(addr);
    }
}

impl ToString for Name {
    fn to_string(&self) -> String {
        self.to_string_lossy().into_owned()
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Name[{}]", self.to_string())
    }
}

unsafe impl Send for Name {}
unsafe impl Sync for Name {}
