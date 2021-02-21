use crate::correlation_id::CorrelationId;
use crate::errors::Error;
use blpapi_sys::*;
use std::fmt::{Debug, Formatter};
use std::ffi::CString;
use std::ptr;
use std::ops::Range;

/// Contains a list of subscriptions used when subscribing and
/// unsubscribing.
pub struct SubscriptionList(pub(crate) *mut blpapi_SubscriptionList_t);

impl SubscriptionList {
    /// Create an empty 'SubscriptionList'.
    pub fn new() -> Self {
        let ptr = unsafe { blpapi_SubscriptionList_create() };
        SubscriptionList(ptr)
    }

    /// Append the specified 'subscriptionString', with the specified
    /// 'fields' and the specified 'options', to this 'SubscriptionList'
    /// object, associating the specified 'correlationId' with it.
    pub fn add(
        &mut self,
        subscription_string: &str,
        fields: Option<Vec<String>>,
        options: Option<Vec<String>>,
        correlation_id: Option<CorrelationId>
    ) -> Result<(), Error> {
        let subscription_string = CString::new(subscription_string).map_err(|err| Error::StringConversionError(Box::new(err)))?;
        let correlation_id = correlation_id.unwrap_or_else(|| CorrelationId::new_empty());
        let res = match (fields, options) {
            (Some(fields), Some(options)) => {
                let fields: Vec<CString> = fields.iter().map(|field| CString::new(field.clone()).unwrap()).collect();
                let options: Vec<CString> = options.iter().map(|option| CString::new(option.clone()).unwrap()).collect();
                unsafe {
                    let mut fields: Vec<_> = fields.iter().map(|field| field.as_ptr()).collect();
                    let mut options: Vec<_> = options.iter().map(|option| option.as_ptr()).collect();
                    blpapi_SubscriptionList_add(
                        self.0,
                        subscription_string.as_ptr(),
                        &correlation_id.0,
                        fields.as_mut_slice().as_mut_ptr(),
                        options.as_mut_slice().as_mut_ptr(),
                        fields.len(),
                        options.len()
                    )
                }
            },
            _ => {
                unsafe {
                    blpapi_SubscriptionList_add(
                        self.0,
                        subscription_string.as_ptr(),
                        &correlation_id.0,
                        ptr::null_mut(),
                        ptr::null_mut(),
                        0,
                        0
                    )
                }
            }
        };
        Error::check(res)
    }

    /// Add the specified 'subscriptionString' to this 'SubscriptionList'
    /// object, associating the specified 'correlationId' with it.  The
    /// subscription string may include options.  The behavior of this
    /// function, and of functions operating on this 'SubscriptionList'
    /// object, is undefined unless 'subscriptionString' is a
    /// fully-resolved subscription string; clients that cannot provide
    /// fully-resolved subscription strings should use
    /// 'SubscriptionList::add' instead.  Note that it is at the discretion
    /// of each function operating on a 'SubscriptionList' whether to
    /// perform resolution on this subscription.
    pub fn add_resolved(&mut self, subscription_string: &str, correlation_id: Option<CorrelationId>) -> Result<(), Error> {
        let subscription_string = CString::new(subscription_string).map_err(|err| Error::StringConversionError(Box::new(err)))?;
        let correlation_id = correlation_id.unwrap_or_else(|| CorrelationId::new_empty());
        let res = unsafe {
            blpapi_SubscriptionList_addResolved(
                self.0,
                subscription_string.as_ptr(),
                &correlation_id.0
            )
        };
        Error::check(res)
    }

    /// Remove all entries from this object.
    pub fn clear(&mut self) -> Result<(), Error> {
        let res = unsafe { blpapi_SubscriptionList_clear(self.0) };
        Error::check(res)
    }

    /// Extend this object by appending a copy of each entry in the
    /// specified 'other'.  Note that this function adds 'other.size()' new
    /// entries to this object.  Note also that this function is alias-safe;
    /// i.e. 'x.append(x)' has well-defined behavior.
    pub fn append(&mut self, other: &SubscriptionList) -> Result<(), Error> {
        let res = unsafe { blpapi_SubscriptionList_append(self.0, other.0) };
        Error::check(res)
    }

    /// Return the number of entries in this object.
    pub fn size(&self) -> usize {
        unsafe { blpapi_SubscriptionList_size(self.0) as usize }
    }

    pub fn correlation_ids(&self) -> CorrelationIdsIterator {
        CorrelationIdsIterator { subscription_list: self, indices: 0..self.size() }
    }
}

impl Drop for SubscriptionList {
    fn drop(&mut self) {
        unsafe { blpapi_SubscriptionList_destroy(self.0); }
    }
}

impl Debug for SubscriptionList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("SubscriptionList[size={}]", self.size()))
    }
}

pub struct CorrelationIdsIterator<'a> {
    subscription_list: &'a SubscriptionList,
    indices: Range<usize>,
}

impl<'a> Iterator for CorrelationIdsIterator<'a> {
    type Item = CorrelationId;

    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().map(|index| {
            let mut correlation_id = CorrelationId::new_empty();
            let res = unsafe { blpapi_SubscriptionList_correlationIdAt(self.subscription_list.0, &mut correlation_id.0, index) };
            Error::check(res).unwrap();
            correlation_id
        })
    }
}
