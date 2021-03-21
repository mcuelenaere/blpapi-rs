use blpapi_sys::*;
use crate::errors::Error;
use std::os::raw::{c_int, c_uint};
use std::fmt::{Formatter, Debug, Display};

pub(crate) enum DatetimeParts {
    Year,
    Month,
    Day,
    Offset,
    Hour,
    Minute,
    Second,
    FractionalSecond,
}

impl DatetimeParts {
    fn to_blpapi(&self) -> c_uint {
        match self {
            DatetimeParts::Year => blpapi_sys::BLPAPI_DATETIME_YEAR_PART,
            DatetimeParts::Month => blpapi_sys::BLPAPI_DATETIME_MONTH_PART,
            DatetimeParts::Day => blpapi_sys::BLPAPI_DATETIME_DAY_PART,
            DatetimeParts::Offset => blpapi_sys::BLPAPI_DATETIME_OFFSET_PART,
            DatetimeParts::Hour => blpapi_sys::BLPAPI_DATETIME_HOURS_PART,
            DatetimeParts::Minute => blpapi_sys::BLPAPI_DATETIME_MINUTES_PART,
            DatetimeParts::Second => blpapi_sys::BLPAPI_DATETIME_SECONDS_PART,
            DatetimeParts::FractionalSecond => blpapi_sys::BLPAPI_DATETIME_FRACSECONDS_PART,
        }
    }
}

#[derive(Clone)]
pub struct Datetime(pub(crate) blpapi_Datetime_t);

macro_rules! impl_getter {
    ($rust_field:ident: $type:ty, $c_field:ident, $datetime:expr) => {
        fn $rust_field(&self) -> Option<$type> {
            if self.has_part($datetime) {
                Some(self.0.$c_field as $type)
            } else {
                None
            }
        }
    };
}

impl Datetime {
    pub(crate) fn has_part(&self, part: DatetimeParts) -> bool {
        (self.0.parts as c_uint & part.to_blpapi()) != 0
    }

    impl_getter!(hours: u8, hours, DatetimeParts::Hour);
    impl_getter!(minutes: u8, minutes, DatetimeParts::Minute);
    impl_getter!(seconds: u8, seconds, DatetimeParts::Second);
    impl_getter!(milli_seconds: u16, milliSeconds, DatetimeParts::FractionalSecond);
    impl_getter!(month: u8, month, DatetimeParts::Month);
    impl_getter!(day: u8, day, DatetimeParts::Day);
    impl_getter!(year: u16, year, DatetimeParts::Year);
    impl_getter!(offset: i16, offset, DatetimeParts::Offset);

    /// Write the value of this object to the specified output 'stream' in
    /// a human-readable format.
    /// Optionally specify an initial indentation 'level', whose absolute
    /// value is incremented recursively for nested objects.  If 'level' is
    /// specified, optionally specify 'spacesPerLevel', whose absolute
    /// value indicates the number of spaces per indentation level for this
    /// and all of its nested objects.  If 'level' is negative, suppress
    /// indentation of the first line.  If 'spacesPerLevel' is negative,
    /// format the entire output on one line, suppressing all but the
    /// initial indentation (as governed by 'level'). Note that this
    /// human-readable format is not fully specified, and can change
    /// without notice.
    pub fn print(&self, f: &mut Formatter<'_>, indent_level: isize, spaces_per_level: isize) -> Result<(), Error> {
        let res = unsafe {
            let stream = std::mem::transmute(f);
            blpapi_Datetime_print(
                &self.0,
                Some(crate::utils::stream_writer),
                stream,
                indent_level as c_int,
                spaces_per_level as c_int
            )
        };
        Error::check(res)
    }
}

impl Default for Datetime {
    fn default() -> Self {
        Datetime(blpapi_Datetime_t {
            parts: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
            milliSeconds: 0,
            month: 0,
            day: 0,
            year: 0,
            offset: 0,
        })
    }
}

impl Debug for Datetime {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Datetime[year={:?}, month={:?}, day={:?}, hours={:?}, minutes={:?}, seconds={:?}, milliSeconds={:?}, offset={:?}]",
            self.year(),
            self.month(),
            self.day(),
            self.hours(),
            self.minutes(),
            self.seconds(),
            self.milli_seconds(),
            self.offset()
        ))
    }
}

impl Display for Datetime {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.print(f, 0, 4).map_err(|_| std::fmt::Error)
    }
}

unsafe impl Send for Datetime {}
unsafe impl Sync for Datetime {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_datetime() {
        let datetime = Datetime::default();
        assert_eq!(datetime.hours(), None);
        assert_eq!(datetime.minutes(), None);
        assert_eq!(datetime.seconds(), None);
        assert_eq!(datetime.milli_seconds(), None);
        assert_eq!(datetime.month(), None);
        assert_eq!(datetime.day(), None);
        assert_eq!(datetime.year(), None);
        assert_eq!(datetime.offset(), None);
        assert_eq!(format!("{}", datetime), "");
        assert_eq!(
            format!("{:?}", datetime),
            "Datetime[year=None, month=None, day=None, hours=None, minutes=None, seconds=None, milliSeconds=None, offset=None]"
        );
    }

    #[test]
    fn test_date() {
        let datetime = Datetime(blpapi_Datetime_t {
            parts: BLPAPI_DATETIME_DATE_PART as u8,
            hours: 0,
            minutes: 0,
            seconds: 0,
            milliSeconds: 0,
            month: 1,
            day: 1,
            year: 2020,
            offset: 0,
        });
        assert_eq!(datetime.hours(), None);
        assert_eq!(datetime.minutes(), None);
        assert_eq!(datetime.seconds(), None);
        assert_eq!(datetime.milli_seconds(), None);
        assert_eq!(datetime.month(), Some(1));
        assert_eq!(datetime.day(), Some(1));
        assert_eq!(datetime.year(), Some(2020));
        assert_eq!(datetime.offset(), None);
        assert_eq!(format!("{}", datetime), "2020-01-01");
        assert_eq!(
            format!("{:?}", datetime),
            "Datetime[year=Some(2020), month=Some(1), day=Some(1), hours=None, minutes=None, seconds=None, milliSeconds=None, offset=None]"
        );
    }

    #[test]
    fn test_datetime() {
        let datetime = Datetime(blpapi_Datetime_t {
            parts: BLPAPI_DATETIME_DATE_PART as u8 | BLPAPI_DATETIME_TIME_PART as u8,
            hours: 8,
            minutes: 5,
            seconds: 10,
            milliSeconds: 0,
            month: 1,
            day: 1,
            year: 2020,
            offset: 0,
        });
        assert_eq!(datetime.hours(), Some(8));
        assert_eq!(datetime.minutes(), Some(5));
        assert_eq!(datetime.seconds(), Some(10));
        assert_eq!(datetime.milli_seconds(), None);
        assert_eq!(datetime.month(), Some(1));
        assert_eq!(datetime.day(), Some(1));
        assert_eq!(datetime.year(), Some(2020));
        assert_eq!(datetime.offset(), None);
        assert_eq!(format!("{}", datetime), "2020-01-01T08:05:10");
        assert_eq!(
            format!("{:?}", datetime),
            "Datetime[year=Some(2020), month=Some(1), day=Some(1), hours=Some(8), minutes=Some(5), seconds=Some(10), milliSeconds=None, offset=None]"
        );
    }

    #[test]
    fn test_datetime_with_offset() {
        let datetime = Datetime(blpapi_Datetime_t {
            parts: BLPAPI_DATETIME_DATE_PART as u8 | BLPAPI_DATETIME_TIME_PART as u8 | BLPAPI_DATETIME_OFFSET_PART as u8,
            hours: 8,
            minutes: 5,
            seconds: 10,
            milliSeconds: 0,
            month: 1,
            day: 1,
            year: 2020,
            offset: 60,
        });
        assert_eq!(datetime.hours(), Some(8));
        assert_eq!(datetime.minutes(), Some(5));
        assert_eq!(datetime.seconds(), Some(10));
        assert_eq!(datetime.milli_seconds(), None);
        assert_eq!(datetime.month(), Some(1));
        assert_eq!(datetime.day(), Some(1));
        assert_eq!(datetime.year(), Some(2020));
        assert_eq!(datetime.offset(), Some(60));
        assert_eq!(format!("{}", datetime), "2020-01-01T08:05:10+01:00");
        assert_eq!(
            format!("{:?}", datetime),
            "Datetime[year=Some(2020), month=Some(1), day=Some(1), hours=Some(8), minutes=Some(5), seconds=Some(10), milliSeconds=None, offset=Some(60)]"
        );
    }
}

#[cfg(feature = "dates")]
mod chrono {
    use super::{Datetime, DatetimeParts};
    use std::convert::TryInto;
    use chrono::prelude::*;

    #[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub enum ChronoConversionError {
        MissingParts,
        InvalidDateTime,
        InvalidOffset,
    }

    impl TryInto<NaiveDate> for Datetime {
        type Error = ChronoConversionError;

        fn try_into(self) -> Result<NaiveDate, Self::Error> {
            if !self.has_part(DatetimeParts::Year) || !self.has_part(DatetimeParts::Month) || !self.has_part(DatetimeParts::Day) {
                return Err(ChronoConversionError::MissingParts);
            }

            NaiveDate::from_ymd_opt(
                self.0.year as i32,
                self.0.month as u32,
                self.0.day as u32
            ).ok_or(ChronoConversionError::InvalidDateTime)
        }
    }

    impl TryInto<Date<FixedOffset>> for Datetime {
        type Error = ChronoConversionError;

        fn try_into(self) -> Result<Date<FixedOffset>, Self::Error> {
            if !self.has_part(DatetimeParts::Offset) {
                return Err(ChronoConversionError::MissingParts);
            }

            let offset = self.0.offset as i32 * 60;
            Ok(Date::from_utc(
                self.try_into()?,
                FixedOffset::east_opt(offset).ok_or(ChronoConversionError::InvalidOffset)?
            ))
        }
    }

    impl TryInto<NaiveTime> for Datetime {
        type Error = ChronoConversionError;

        fn try_into(self) -> Result<NaiveTime, Self::Error> {
            if !self.has_part(DatetimeParts::Hour) ||
                !self.has_part(DatetimeParts::Minute) ||
                !self.has_part(DatetimeParts::Second) {
                return Err(ChronoConversionError::MissingParts);
            }

            if self.has_part(DatetimeParts::FractionalSecond) {
                NaiveTime::from_hms_milli_opt(
                    self.0.hours as u32,
                    self.0.minutes as u32,
                    self.0.seconds as u32,
                    self.0.milliSeconds as u32
                ).ok_or(ChronoConversionError::InvalidDateTime)
            } else {
                NaiveTime::from_hms_opt(
                    self.0.hours as u32,
                    self.0.minutes as u32,
                    self.0.seconds as u32,
                ).ok_or(ChronoConversionError::InvalidDateTime)
            }
        }
    }

    impl TryInto<NaiveDateTime> for Datetime {
        type Error = ChronoConversionError;

        fn try_into(self) -> Result<NaiveDateTime, Self::Error> {
            Ok(NaiveDateTime::new(self.clone().try_into()?, self.try_into()?))
        }
    }

    impl TryInto<DateTime<FixedOffset>> for Datetime {
        type Error = ChronoConversionError;

        fn try_into(self) -> Result<DateTime<FixedOffset>, Self::Error> {
            if !self.has_part(DatetimeParts::Offset) {
                return Err(ChronoConversionError::MissingParts);
            }

            let offset = self.0.offset as i32 * 60;
            Ok(DateTime::from_utc(
                self.try_into()?,
                FixedOffset::east_opt(offset).ok_or(ChronoConversionError::InvalidOffset)?
            ))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use blpapi_sys::*;

        #[test]
        fn test_empty_datetime() {
            let datetime = Datetime::default();
            assert_eq!(
                TryInto::<NaiveDate>::try_into(datetime.clone()),
                Err(ChronoConversionError::MissingParts)
            );
            assert_eq!(
                TryInto::<NaiveTime>::try_into(datetime.clone()),
                Err(ChronoConversionError::MissingParts)
            );
            assert_eq!(
                TryInto::<NaiveDateTime>::try_into(datetime.clone()),
                Err(ChronoConversionError::MissingParts)
            );
            assert_eq!(
                TryInto::<Date<FixedOffset>>::try_into(datetime.clone()),
                Err(ChronoConversionError::MissingParts)
            );
            assert_eq!(
                TryInto::<DateTime<FixedOffset>>::try_into(datetime.clone()),
                Err(ChronoConversionError::MissingParts)
            );
        }

        #[test]
        fn test_date() {
            let datetime = Datetime(blpapi_Datetime_t {
                parts: BLPAPI_DATETIME_DATE_PART as u8,
                hours: 0,
                minutes: 0,
                seconds: 0,
                milliSeconds: 0,
                month: 1,
                day: 1,
                year: 2020,
                offset: 0,
            });
            assert_eq!(
                datetime.clone().try_into(),
                Ok(NaiveDate::from_ymd(2020, 1, 1))
            );
            assert_eq!(
                TryInto::<NaiveTime>::try_into(datetime.clone()),
                Err(ChronoConversionError::MissingParts)
            );
            assert_eq!(
                TryInto::<NaiveDateTime>::try_into(datetime.clone()),
                Err(ChronoConversionError::MissingParts)
            );
            assert_eq!(
                TryInto::<Date<FixedOffset>>::try_into(datetime.clone()),
                Err(ChronoConversionError::MissingParts)
            );
            assert_eq!(
                TryInto::<DateTime<FixedOffset>>::try_into(datetime.clone()),
                Err(ChronoConversionError::MissingParts)
            );
        }

        #[test]
        fn test_datetime() {
            let datetime = Datetime(blpapi_Datetime_t {
                parts: BLPAPI_DATETIME_DATE_PART as u8 | BLPAPI_DATETIME_TIME_PART as u8,
                hours: 8,
                minutes: 5,
                seconds: 10,
                milliSeconds: 0,
                month: 1,
                day: 1,
                year: 2020,
                offset: 0,
            });
            assert_eq!(
                datetime.clone().try_into(),
                Ok(NaiveDate::from_ymd(2020, 1, 1))
            );
            assert_eq!(
                datetime.clone().try_into(),
                Ok(NaiveTime::from_hms(8, 5, 10))
            );
            assert_eq!(
                datetime.clone().try_into(),
                Ok(NaiveDateTime::new(NaiveDate::from_ymd(2020, 1, 1), NaiveTime::from_hms(8, 5, 10)))
            );
            assert_eq!(
                TryInto::<Date<FixedOffset>>::try_into(datetime.clone()),
                Err(ChronoConversionError::MissingParts)
            );
            assert_eq!(
                TryInto::<DateTime<FixedOffset>>::try_into(datetime.clone()),
                Err(ChronoConversionError::MissingParts)
            );
        }

        #[test]
        fn test_datetime_with_offset() {
            let datetime = Datetime(blpapi_Datetime_t {
                parts: BLPAPI_DATETIME_DATE_PART as u8 | BLPAPI_DATETIME_TIME_PART as u8 | BLPAPI_DATETIME_OFFSET_PART as u8,
                hours: 8,
                minutes: 5,
                seconds: 10,
                milliSeconds: 0,
                month: 1,
                day: 1,
                year: 2020,
                offset: 60,
            });
            assert_eq!(
                datetime.clone().try_into(),
                Ok(NaiveDate::from_ymd(2020, 1, 1))
            );
            assert_eq!(
                datetime.clone().try_into(),
                Ok(NaiveTime::from_hms(8, 5, 10))
            );
            assert_eq!(
                datetime.clone().try_into(),
                Ok(NaiveDateTime::new(NaiveDate::from_ymd(2020, 1, 1), NaiveTime::from_hms(8, 5, 10)))
            );
            assert_eq!(
                datetime.clone().try_into(),
                Ok(Date::from_utc(NaiveDate::from_ymd(2020, 1, 1), FixedOffset::east(60 * 60)))
            );
            assert_eq!(
                datetime.clone().try_into(),
                Ok(DateTime::from_utc(
                    NaiveDateTime::new(NaiveDate::from_ymd(2020, 1, 1), NaiveTime::from_hms(8, 5, 10)),
                    FixedOffset::east(60 * 60)
                ))
            );
        }
    }
}