use serde::Deserialize;
use crate::element::{Element, DataType, Elements};
use crate::name::Name;
use serde::de::{Visitor, SeqAccess, DeserializeSeed, MapAccess};
use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // One or more variants that can be created by data structures through the
    // `ser::Error` and `de::Error` traits. For example the Serialize impl for
    // Mutex<T> might return an error because the mutex is poisoned, or the
    // Deserialize impl for a struct may return an error because a required
    // field is missing.
    Message(String),

    ElementNotFoundAtIndex(Element, Option<usize>),
    ElementNotFoundAtField(Element, Name),
    UnsupportedType,
    ExpectedArrayOrComplexType,
    ExpectedNull,
    ExpectedValue,
    BlpApiError(crate::errors::Error),
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::ElementNotFoundAtField(element, field) =>
                formatter.write_fmt(format_args!("no element found in {:?} with field {:?}", element, field)),
            Error::ElementNotFoundAtIndex(element, index) =>
                formatter.write_fmt(format_args!(
                    "no element found in {:?} at index {}",
                    element,
                    index.map_or("<none>".to_string(), |index| index.to_string()
                ))),
            Error::UnsupportedType => formatter.write_str("unsupported type"),
            Error::ExpectedNull => formatter.write_str("expected null value"),
            Error::ExpectedValue => formatter.write_str("expected value in map"),
            Error::ExpectedArrayOrComplexType => formatter.write_str("expected array or complex type"),
            Error::BlpApiError(err) => formatter.write_fmt(format_args!("blpapi error: {}", err)),
        }
    }
}

#[derive(Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum FieldValue<T>
{
    /// Field is present, containing value `T`
    Present(T),
    /// Field is missing
    Missing,
}

impl<T> Default for FieldValue<T> {
    fn default() -> Self {
        FieldValue::Missing
    }
}

impl<T: Clone> Clone for FieldValue<T> {
    fn clone(&self) -> Self {
        match self {
            FieldValue::Present(x) => FieldValue::Present(x.clone()),
            FieldValue::Missing => FieldValue::Missing,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (FieldValue::Present(to), FieldValue::Present(from)) => to.clone_from(from),
            (to, from) => *to = from.clone(),
        }
    }
}

impl<'de, T: Deserialize<'de>> serde::Deserialize<'de> for FieldValue<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        match T::deserialize(deserializer) {
            Ok(value) => Ok(FieldValue::Present(value)),
            //Err(Error::ElementNotFoundAtField(_, _)) => Ok(FieldValue::Missing),
            Err(error) => {
                // we have to resort to this hack until specialization lands in stable
                let formatted_error = format!("{}", error);
                if formatted_error.starts_with("no element found in ") && formatted_error.contains(" with field ") {
                    Ok(FieldValue::Missing)
                } else {
                    Err(error)
                }
            },
        }
    }
}

pub struct ElementDeserializer {
    input: Element,
    value_index: Option<usize>,
}

pub fn from_element<'de, T>(input: Element) -> Result<T>
    where T: Deserialize<'de>
{
    let mut deserializer = ElementDeserializer { input, value_index: None };
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

macro_rules! impl_deserialize {
    ($deserialize:ident, $visit:ident, $blapi_type:ty) => {
        fn $deserialize<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
            V: Visitor<'de> {
            visitor.$visit(
                self.input
                    .get_at::<$blapi_type>(self.value_index.unwrap_or(0))
                    .ok_or(Error::ElementNotFoundAtIndex(self.input.clone(), self.value_index))?
            )
        }
    };
    ($deserialize:ident, $visit:ident, $blapi_type:ty, $dest_type:ty) => {
        fn $deserialize<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
            V: Visitor<'de> {
            visitor.$visit(
                self.input
                    .get_at::<$blapi_type>(self.value_index.unwrap_or(0))
                    .ok_or(Error::ElementNotFoundAtIndex(self.input.clone(), self.value_index))?
                as $dest_type
            )
        }
    };
}

macro_rules! unsupported_type {
    ($deserialize:ident) => {
        fn $deserialize<V>(self, _: V) -> Result<<V as Visitor<'de>>::Value> where
            V: Visitor<'de> {
            Err(Error::UnsupportedType)
        }
    };
}

impl ElementDeserializer {
    fn is_null(&self) -> Result<bool> {
        match self.value_index {
            Some(index) => self.input.is_null_value(index).map_err(|err| Error::BlpApiError(err)),
            None => self.input.is_null().map_err(|err| Error::BlpApiError(err)),
        }
    }
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut ElementDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        if self.is_null()? {
            return visitor.visit_none();
        }

        match self.input.data_type() {
            DataType::Bool => self.deserialize_bool(visitor),
            DataType::Char => self.deserialize_i8(visitor),
            DataType::Int32 => self.deserialize_i32(visitor),
            DataType::Int64 => self.deserialize_i64(visitor),
            DataType::Float32 => self.deserialize_f32(visitor),
            DataType::Float64 => self.deserialize_f64(visitor),
            DataType::String => self.deserialize_string(visitor),
            DataType::Sequence => self.deserialize_seq(visitor),
            DataType::Choice => self.deserialize_seq(visitor),
            _ => Err(Error::UnsupportedType),
        }
    }

    impl_deserialize!(deserialize_i8, visit_i8, i8);
    unsupported_type!(deserialize_i16);
    impl_deserialize!(deserialize_i32, visit_i32, i32);
    impl_deserialize!(deserialize_i64, visit_i64, i64);

    impl_deserialize!(deserialize_u8, visit_u8, i8, u8);
    unsupported_type!(deserialize_u16);
    impl_deserialize!(deserialize_u32, visit_u32, i32, u32);
    impl_deserialize!(deserialize_u64, visit_u64, i64, u64);

    impl_deserialize!(deserialize_f32, visit_f32, f32);
    impl_deserialize!(deserialize_f64, visit_f64, f64);

    impl_deserialize!(deserialize_bool, visit_bool, bool);
//impl_deserialize!(deserialize_char, visit_char, i8, char);
    unsupported_type!(deserialize_char);
    impl_deserialize!(deserialize_str, visit_string, String);
    impl_deserialize!(deserialize_string, visit_string, String);

    unsupported_type!(deserialize_bytes);
    unsupported_type!(deserialize_byte_buf);

    fn deserialize_option<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        if self.is_null()? {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        if self.is_null()? {
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedNull)
        }
    }

    fn deserialize_unit_struct<V>(self, _: &'static str, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _: &'static str, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        if self.input.is_array() {
            let len = self.input.num_values();
            visitor.visit_seq(IndexBased { de: self, indices: 0..len, use_values: true })
        } else if self.input.is_complex_type() {
            let len = self.input.num_elements();
            visitor.visit_seq(IndexBased { de: self, indices: 0..len, use_values: false })
        } else {
            Err(Error::ExpectedArrayOrComplexType)
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        if self.input.is_array() {
            visitor.visit_seq(IndexBased { de: self, indices: 0..len, use_values: true })
        } else if self.input.is_complex_type() {
            visitor.visit_seq(IndexBased { de: self, indices: 0..len, use_values: false })
        } else {
            Err(Error::ExpectedArrayOrComplexType)
        }
    }

    fn deserialize_tuple_struct<V>(self, _: &'static str, len: usize, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        if !self.input.is_complex_type() {
            return Err(Error::UnsupportedType);
        }

        visitor.visit_map(ElementsIterator { it: self.input.elements(), current_element: None })
    }

    fn deserialize_struct<V>(self, _: &'static str, fields: &'static [&'static str], visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        let element = match self.value_index {
            Some(index) => self.input
                .get_at::<Element>(index)
                .ok_or_else(|| Error::ElementNotFoundAtIndex(self.input.clone(), Some(index)))?,
            None => self.input.clone(),
        };
        visitor.visit_seq(FieldBased { element, fields: fields.iter() })
    }

    fn deserialize_enum<V>(self, _: &'static str, variants: &'static [&'static str], visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        visitor.visit_seq(FieldBased { element: self.input.clone(), fields: variants.iter() })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        self.deserialize_any(visitor)
    }
}

struct NameDeserializer {
    input: Name,
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut NameDeserializer {
    type Error = Error;

    unsupported_type!(deserialize_any);
    unsupported_type!(deserialize_bool);
    unsupported_type!(deserialize_i8);
    unsupported_type!(deserialize_i16);
    unsupported_type!(deserialize_i32);
    unsupported_type!(deserialize_i64);
    unsupported_type!(deserialize_u8);
    unsupported_type!(deserialize_u16);
    unsupported_type!(deserialize_u32);
    unsupported_type!(deserialize_u64);
    unsupported_type!(deserialize_f32);
    unsupported_type!(deserialize_f64);
    unsupported_type!(deserialize_char);
    unsupported_type!(deserialize_bytes);
    unsupported_type!(deserialize_byte_buf);
    unsupported_type!(deserialize_option);
    unsupported_type!(deserialize_unit);

    fn deserialize_str<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        visitor.visit_string(self.input.to_string())
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        visitor.visit_string(self.input.to_string())
    }

    unsupported_type!(deserialize_seq);
    fn deserialize_unit_struct<V>(self, _: &'static str, _: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        Err(Error::UnsupportedType)
    }
    fn deserialize_newtype_struct<V>(self, _: &'static str, _: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        Err(Error::UnsupportedType)
    }
    unsupported_type!(deserialize_map);
    fn deserialize_tuple<V>(self, _: usize, _: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        Err(Error::UnsupportedType)
    }
    fn deserialize_tuple_struct<V>(self, _: &'static str, _: usize, _: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        Err(Error::UnsupportedType)
    }
    unsupported_type!(deserialize_identifier);
    unsupported_type!(deserialize_ignored_any);
    fn deserialize_struct<V>(self, _: &'static str, _: &'static [&'static str], _: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        Err(Error::UnsupportedType)
    }

    fn deserialize_enum<V>(self, _: &'static str, _: &'static [&'static str], _: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        Err(Error::UnsupportedType)
    }
}

struct ErrorDeserializer<F: Fn() -> Error> {
    generate_error: F,
}

macro_rules! visit_err {
    ($deserialize:ident) => {
        fn $deserialize<V>(self, _: V) -> Result<<V as Visitor<'de>>::Value> where
            V: Visitor<'de> {
            let err = (self.generate_error)();
            Err(err)
        }
    };
}

impl<'de, 'a, F> serde::Deserializer<'de> for &'a mut ErrorDeserializer<F>
    where F: Fn() -> Error
{
    type Error = Error;

    visit_err!(deserialize_any);
    visit_err!(deserialize_bool);
    visit_err!(deserialize_i8);
    visit_err!(deserialize_i16);
    visit_err!(deserialize_i32);
    visit_err!(deserialize_i64);
    visit_err!(deserialize_u8);
    visit_err!(deserialize_u16);
    visit_err!(deserialize_u32);
    visit_err!(deserialize_u64);
    visit_err!(deserialize_f32);
    visit_err!(deserialize_f64);
    visit_err!(deserialize_char);
    visit_err!(deserialize_bytes);
    visit_err!(deserialize_byte_buf);
    visit_err!(deserialize_option);
    visit_err!(deserialize_unit);
    visit_err!(deserialize_str);
    visit_err!(deserialize_string);
    visit_err!(deserialize_seq);
    fn deserialize_unit_struct<V>(self, _: &'static str, _: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        let err = (self.generate_error)();
        Err(err)
    }
    fn deserialize_newtype_struct<V>(self, _: &'static str, _: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        let err = (self.generate_error)();
        Err(err)
    }
    visit_err!(deserialize_map);
    fn deserialize_tuple<V>(self, _: usize, _: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        let err = (self.generate_error)();
        Err(err)
    }
    fn deserialize_tuple_struct<V>(self, _: &'static str, _: usize, _: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        let err = (self.generate_error)();
        Err(err)
    }
    visit_err!(deserialize_identifier);
    visit_err!(deserialize_ignored_any);
    fn deserialize_struct<V>(self, _: &'static str, _: &'static [&'static str], _: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        let err = (self.generate_error)();
        Err(err)
    }

    fn deserialize_enum<V>(self, _: &'static str, _: &'static [&'static str], _: V) -> Result<<V as Visitor<'de>>::Value> where
        V: Visitor<'de> {
        let err = (self.generate_error)();
        Err(err)
    }
}

struct ElementsIterator<'e> {
    it: Elements<'e>,
    current_element: Option<Element>,
}

impl<'e, 'de> MapAccess<'de> for ElementsIterator<'e> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<<K as DeserializeSeed<'de>>::Value>> where
        K: DeserializeSeed<'de> {
        match self.it.next() {
            Some(element) => {
                self.current_element = Some(element.clone());
                let mut de = NameDeserializer { input: element.name() };
                seed.deserialize(&mut de).map(Some)
            },
            None => {
                self.current_element = None;
                Ok(None)
            },
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<<V as DeserializeSeed<'de>>::Value> where
        V: DeserializeSeed<'de> {
        match self.current_element.as_ref() {
            Some(element) => {
                let mut de = ElementDeserializer { input: element.clone(), value_index: None };
                seed.deserialize(&mut de)
            },
            None => Err(Error::ExpectedValue),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.it.size_hint().1
    }
}

struct FieldBased {
    element: Element,
    // TODO: this should use Name instead
    fields: std::slice::Iter<'static, &'static str>,
}

impl<'de> SeqAccess<'de> for FieldBased {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<<T as DeserializeSeed<'de>>::Value>> where
        T: DeserializeSeed<'de>
    {
        match self.fields.next() {
            Some(field) => {
                let element = if self.element.has_element(field, false) {
                    self.element.get_element(field)
                } else {
                    None
                };

                match element {
                    Some(element) => {
                        let mut de = ElementDeserializer { input: element, value_index: None };
                        seed.deserialize(&mut de).map(Some)
                    },
                    None => {
                        let mut de = ErrorDeserializer {
                            generate_error: || Error::ElementNotFoundAtField(self.element.clone(), Name::new(field)),
                        };
                        seed.deserialize(&mut de).map(Some)
                    },
                }
            },
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.fields.size_hint().1
    }
}

struct IndexBased<'a> {
    de: &'a mut ElementDeserializer,
    indices: std::ops::Range<usize>,
    use_values: bool,
}

impl<'de, 'a> SeqAccess<'de> for IndexBased<'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<<T as DeserializeSeed<'de>>::Value>> where
        T: DeserializeSeed<'de>
    {
        match self.indices.next() {
            Some(index) => {
                if self.use_values {
                    let mut de = ElementDeserializer { input: self.de.input.clone(), value_index: Some(index) };
                    seed.deserialize(&mut de).map(Some)
                } else {
                    match self.de.input.get_element_at(index) {
                        Some(element) => {
                            let mut de = ElementDeserializer { input: element, value_index: None };
                            seed.deserialize(&mut de).map(Some)
                        },
                        None => {
                            let mut de = ErrorDeserializer {
                                generate_error: || Error::ElementNotFoundAtIndex(self.de.input.clone(), Some(index)),
                            };
                            seed.deserialize(&mut de).map(Some)
                        },
                    }
                }
            },
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.indices.size_hint().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Error;
    use crate::event::{Event, EventType};
    use crate::testutil::EventBuilder;
    use std::result::Result;
    use std::collections::HashMap;

    #[derive(Deserialize, PartialEq, Debug)]
    struct Reason {
        source: String,
        #[serde(rename="errorCode")]
        error_code: i32,
        category: String,
        description: String,
        subcategory: String,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Exception {
        #[serde(rename="fieldId")]
        field_id: String,
        reason: Reason,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct ReceivedFrom {
        address: String,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct SubscriptionStarted {
        exceptions: Vec<Exception>,
        #[serde(rename="resubscriptionId")]
        resubscription_id: i32,
        #[serde(rename="streamIds")]
        stream_ids: Vec<String>,
        #[serde(rename="receivedFrom")]
        received_from: ReceivedFrom,
        reason: String,
    }

    #[test]
    fn test_subscription_started() -> Result<(), Error> {
        let msg_contents = r#"
            {
                "exceptions": [
                    {
                        "fieldId": "field",
                        "reason": {
                            "source":      "TestUtil",
                            "errorCode":   -1,
                            "category":    "CATEGORY",
                            "description": "for testing",
                            "subcategory": "SUBCATEGORY"
                        }
                    }
                ],
                "resubscriptionId": 123,
                "streamIds": [
                    "123",
                    "456"
                ],
                "receivedFrom": { "address": "12.34.56.78:8194" },
                "reason":      "TestUtil"
            }
        "#;

        let event = EventBuilder::new(EventType::SubscriptionData)?
            .append_message_from_json(Name::new("SubscriptionStarted"), None, msg_contents)?
            .build();

        let msg = event.messages().next().unwrap();
        let result = from_element::<SubscriptionStarted>(msg.element()).unwrap();

        assert_eq!(result, SubscriptionStarted {
            exceptions: vec![Exception {
                field_id: "field".to_string(),
                reason: Reason {
                    source: "TestUtil".to_string(),
                    error_code: -1,
                    category: "CATEGORY".to_string(),
                    description: "for testing".to_string(),
                    subcategory: "SUBCATEGORY".to_string()
                }
            }],
            resubscription_id: 123,
            stream_ids: vec!["123".to_string(), "456".to_string()],
            received_from: ReceivedFrom {
                address: "12.34.56.78:8194".to_string(),
            },
            reason: "TestUtil".to_string()
        });

        Ok(())
    }

    #[test]
    fn test_subelement() -> Result<(), Error> {
        let msg_contents = r#"
            {
                "exceptions": [
                    {
                        "fieldId": "field1",
                        "reason": {
                            "source":      "TestUtil",
                            "errorCode":   -1,
                            "category":    "CATEGORY",
                            "description": "for testing",
                            "subcategory": "SUBCATEGORY"
                        }
                    },
                    {
                        "fieldId": "field2",
                        "reason": {
                            "source":      "TestUtil",
                            "errorCode":   -2,
                            "category":    "CATEGORY2",
                            "description": "for testing2",
                            "subcategory": "SUBCATEGORY2"
                        }
                    }
                ]
            }
        "#;

        let event = EventBuilder::new(EventType::SubscriptionData)?
            .append_message_from_json(Name::new("SubscriptionStarted"), None, msg_contents)?
            .build();

        let element = event.messages().next().unwrap().element();
        let exceptions: Vec<_> = element
            .get_element("exceptions").unwrap()
            .values::<Element>()
            .map(|value| from_element::<Exception>(value).unwrap())
            .collect()
        ;

        assert_eq!(
            exceptions,
            vec![
                Exception {
                    field_id: "field1".to_string(),
                    reason: Reason {
                        source: "TestUtil".to_string(),
                        error_code: -1,
                        category: "CATEGORY".to_string(),
                        description: "for testing".to_string(),
                        subcategory: "SUBCATEGORY".to_string()
                    }
                },
                Exception {
                    field_id: "field2".to_string(),
                    reason: Reason {
                        source: "TestUtil".to_string(),
                        error_code: -2,
                        category: "CATEGORY2".to_string(),
                        description: "for testing2".to_string(),
                        subcategory: "SUBCATEGORY2".to_string()
                    }
                },
            ]
        );

        Ok(())
    }

    #[test]
    fn test_map() -> Result<(), Error> {
        let msg_contents = r#"
            {
                "exceptions": [
                    {
                        "fieldId": "field1",
                        "reason": {
                            "source":      "TestUtil",
                           "errorCode":   -1,
                            "category":    "CATEGORY",
                            "description": "for testing",
                            "subcategory": "SUBCATEGORY"
                        }
                    }
                ]
            }
        "#;

        let event = EventBuilder::new(EventType::SubscriptionData)?
            .append_message_from_json(Name::new("SubscriptionStarted"), None, msg_contents)?
            .build();

        #[derive(Deserialize, PartialEq, Debug)]
        struct ExceptionWithMap {
            #[serde(rename="fieldId")]
            field_id: String,
            reason: HashMap<String, String>,
        }

        let element = event.messages().next().unwrap().element();
        let exception = element
            .get_element("exceptions").unwrap()
            .values::<Element>()
            .map(|value| from_element::<ExceptionWithMap>(value).unwrap())
            .next().unwrap()
        ;

        assert_eq!(
            exception,
            ExceptionWithMap {
                field_id: "field1".to_string(),
                reason: [
                    ("source", "TestUtil"),
                    ("errorCode", "-1"),
                    ("category", "CATEGORY"),
                    ("description", "for testing"),
                    ("subcategory", "SUBCATEGORY"),
                ].iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<HashMap<String, String>>(),
            },
        );

        Ok(())
    }

    fn build_subscription_data_event(msg_contents: &str) -> Result<Event, Error> {
        let event = EventBuilder::new(EventType::SubscriptionData)?
            .append_message_from_json(Name::new("SubscriptionStarted"), None, msg_contents)?
            .build();
        Ok(event)
    }

    #[test]
    fn test_missing_fields() -> Result<(), Error> {
        #[derive(Deserialize, PartialEq, Debug)]
        struct SubscriptionStarted {
            exceptions: FieldValue<Vec<Exception>>,
            #[serde(rename="resubscriptionId")]
            resubscription_id: FieldValue<i32>,
            #[serde(rename="streamIds")]
            stream_ids: FieldValue<Vec<String>>,
            #[serde(rename="receivedFrom")]
            received_from: FieldValue<Option<ReceivedFrom>>,
            reason: FieldValue<String>,
        }

        let event = build_subscription_data_event(r#"
            {
                "resubscriptionId": 123,
                "streamIds": [
                    "123",
                    "456"
                ],
                "reason":      "TestUtil"
            }
        "#)?;

        let element = event.messages().next().unwrap().element();
        let msg = from_element::<SubscriptionStarted>(element).unwrap();

        assert_eq!(
            msg,
            SubscriptionStarted {
                exceptions: FieldValue::Missing,
                resubscription_id: FieldValue::Present(123),
                stream_ids: FieldValue::Present(vec!["123".to_string(), "456".to_string()]),
                received_from: FieldValue::Present(None),
                reason: FieldValue::Present("TestUtil".to_string()),
            }
        );

        Ok(())
    }
}
