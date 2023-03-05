// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Zero-sized types used to parameterize generic array implementations

use crate::array::ArrowPrimitiveType;
use crate::delta::shift_months;
use crate::{ArrowNativeTypeOp, OffsetSizeTrait};
use arrow_buffer::{i256, ArrowNativeType};
use arrow_data::decimal::{validate_decimal256_precision, validate_decimal_precision};
use arrow_schema::{
    ArrowError, DataType, IntervalUnit, TimeUnit, DECIMAL128_MAX_PRECISION,
    DECIMAL128_MAX_SCALE, DECIMAL256_MAX_PRECISION, DECIMAL256_MAX_SCALE,
    DECIMAL_DEFAULT_SCALE,
};
use chrono::{Duration, NaiveDate};
use half::f16;
use lazy_static::lazy_static;
use regex::Regex;
use std::marker::PhantomData;
use std::ops::{Add, Sub};

// BooleanType is special: its bit-width is not the size of the primitive type, and its `index`
// operation assumes bit-packing.
/// A boolean datatype
#[derive(Debug)]
pub struct BooleanType {}

impl BooleanType {
    /// Type represetings is arrow [`DataType`]
    pub const DATA_TYPE: DataType = DataType::Boolean;
}

macro_rules! make_type {
    ($name:ident, $native_ty:ty, $data_ty:expr, $doc_string: literal) => {
        #[derive(Debug)]
        #[doc = $doc_string]
        pub struct $name {}

        impl ArrowPrimitiveType for $name {
            type Native = $native_ty;
            const DATA_TYPE: DataType = $data_ty;
        }
    };
}

make_type!(Int8Type, i8, DataType::Int8, "A signed 8-bit integer type.");
make_type!(
    Int16Type,
    i16,
    DataType::Int16,
    "A signed 16-bit integer type."
);
make_type!(
    Int32Type,
    i32,
    DataType::Int32,
    "A signed 32-bit integer type."
);
make_type!(
    Int64Type,
    i64,
    DataType::Int64,
    "A signed 64-bit integer type."
);
make_type!(
    UInt8Type,
    u8,
    DataType::UInt8,
    "An unsigned 8-bit integer type."
);
make_type!(
    UInt16Type,
    u16,
    DataType::UInt16,
    "An unsigned 16-bit integer type."
);
make_type!(
    UInt32Type,
    u32,
    DataType::UInt32,
    "An unsigned 32-bit integer type."
);
make_type!(
    UInt64Type,
    u64,
    DataType::UInt64,
    "An unsigned 64-bit integer type."
);
make_type!(
    Float16Type,
    f16,
    DataType::Float16,
    "A 16-bit floating point number type."
);
make_type!(
    Float32Type,
    f32,
    DataType::Float32,
    "A 32-bit floating point number type."
);
make_type!(
    Float64Type,
    f64,
    DataType::Float64,
    "A 64-bit floating point number type."
);
make_type!(
    TimestampSecondType,
    i64,
    DataType::Timestamp(TimeUnit::Second, None),
    "A timestamp second type with an optional timezone."
);
make_type!(
    TimestampMillisecondType,
    i64,
    DataType::Timestamp(TimeUnit::Millisecond, None),
    "A timestamp millisecond type with an optional timezone."
);
make_type!(
    TimestampMicrosecondType,
    i64,
    DataType::Timestamp(TimeUnit::Microsecond, None),
    "A timestamp microsecond type with an optional timezone."
);
make_type!(
    TimestampNanosecondType,
    i64,
    DataType::Timestamp(TimeUnit::Nanosecond, None),
    "A timestamp nanosecond type with an optional timezone."
);
make_type!(
    Date32Type,
    i32,
    DataType::Date32,
    "A 32-bit date type representing the elapsed time since UNIX epoch in days(32 bits)."
);
make_type!(
    Date64Type,
    i64,
    DataType::Date64,
    "A 64-bit date type representing the elapsed time since UNIX epoch in milliseconds(64 bits)."
);
make_type!(
    Time32SecondType,
    i32,
    DataType::Time32(TimeUnit::Second),
    "A 32-bit time type representing the elapsed time since midnight in seconds."
);
make_type!(
    Time32MillisecondType,
    i32,
    DataType::Time32(TimeUnit::Millisecond),
    "A 32-bit time type representing the elapsed time since midnight in milliseconds."
);
make_type!(
    Time64MicrosecondType,
    i64,
    DataType::Time64(TimeUnit::Microsecond),
    "A 64-bit time type representing the elapsed time since midnight in microseconds."
);
make_type!(
    Time64NanosecondType,
    i64,
    DataType::Time64(TimeUnit::Nanosecond),
    "A 64-bit time type representing the elapsed time since midnight in nanoseconds."
);
make_type!(
    IntervalYearMonthType,
    i32,
    DataType::Interval(IntervalUnit::YearMonth),
    "A “calendar” interval type in months."
);
make_type!(
    IntervalDayTimeType,
    i64,
    DataType::Interval(IntervalUnit::DayTime),
    "A “calendar” interval type in days and milliseconds."
);
make_type!(
    IntervalMonthDayNanoType,
    i128,
    DataType::Interval(IntervalUnit::MonthDayNano),
    "A “calendar” interval type in months, days, and nanoseconds."
);
make_type!(
    DurationSecondType,
    i64,
    DataType::Duration(TimeUnit::Second),
    "An elapsed time type in seconds."
);
make_type!(
    DurationMillisecondType,
    i64,
    DataType::Duration(TimeUnit::Millisecond),
    "An elapsed time type in milliseconds."
);
make_type!(
    DurationMicrosecondType,
    i64,
    DataType::Duration(TimeUnit::Microsecond),
    "An elapsed time type in microseconds."
);
make_type!(
    DurationNanosecondType,
    i64,
    DataType::Duration(TimeUnit::Nanosecond),
    "An elapsed time type in nanoseconds."
);

/// A subtype of primitive type that represents legal dictionary keys.
/// See <https://arrow.apache.org/docs/format/Columnar.html>
pub trait ArrowDictionaryKeyType: ArrowPrimitiveType {}

impl ArrowDictionaryKeyType for Int8Type {}

impl ArrowDictionaryKeyType for Int16Type {}

impl ArrowDictionaryKeyType for Int32Type {}

impl ArrowDictionaryKeyType for Int64Type {}

impl ArrowDictionaryKeyType for UInt8Type {}

impl ArrowDictionaryKeyType for UInt16Type {}

impl ArrowDictionaryKeyType for UInt32Type {}

impl ArrowDictionaryKeyType for UInt64Type {}

mod run {
    use super::*;

    pub trait RunEndTypeSealed {}

    impl RunEndTypeSealed for Int16Type {}

    impl RunEndTypeSealed for Int32Type {}

    impl RunEndTypeSealed for Int64Type {}
}

/// A subtype of primitive type that is used as run-ends index
/// in `RunArray`.
/// See <https://arrow.apache.org/docs/format/Columnar.html>
///
/// Note: The implementation of this trait is sealed to avoid accidental misuse.
pub trait RunEndIndexType: ArrowPrimitiveType + run::RunEndTypeSealed {}

impl RunEndIndexType for Int16Type {}

impl RunEndIndexType for Int32Type {}

impl RunEndIndexType for Int64Type {}

/// A subtype of primitive type that represents temporal values.
pub trait ArrowTemporalType: ArrowPrimitiveType {}

impl ArrowTemporalType for TimestampSecondType {}
impl ArrowTemporalType for TimestampMillisecondType {}
impl ArrowTemporalType for TimestampMicrosecondType {}
impl ArrowTemporalType for TimestampNanosecondType {}
impl ArrowTemporalType for Date32Type {}
impl ArrowTemporalType for Date64Type {}
impl ArrowTemporalType for Time32SecondType {}
impl ArrowTemporalType for Time32MillisecondType {}
impl ArrowTemporalType for Time64MicrosecondType {}
impl ArrowTemporalType for Time64NanosecondType {}
// impl ArrowTemporalType for IntervalYearMonthType {}
// impl ArrowTemporalType for IntervalDayTimeType {}
// impl ArrowTemporalType for IntervalMonthDayNanoType {}
impl ArrowTemporalType for DurationSecondType {}
impl ArrowTemporalType for DurationMillisecondType {}
impl ArrowTemporalType for DurationMicrosecondType {}
impl ArrowTemporalType for DurationNanosecondType {}

/// A timestamp type allows us to create array builders that take a timestamp.
pub trait ArrowTimestampType: ArrowTemporalType {
    /// Returns the `TimeUnit` of this timestamp.
    fn get_time_unit() -> TimeUnit;
}

impl ArrowTimestampType for TimestampSecondType {
    fn get_time_unit() -> TimeUnit {
        TimeUnit::Second
    }
}
impl ArrowTimestampType for TimestampMillisecondType {
    fn get_time_unit() -> TimeUnit {
        TimeUnit::Millisecond
    }
}
impl ArrowTimestampType for TimestampMicrosecondType {
    fn get_time_unit() -> TimeUnit {
        TimeUnit::Microsecond
    }
}
impl ArrowTimestampType for TimestampNanosecondType {
    fn get_time_unit() -> TimeUnit {
        TimeUnit::Nanosecond
    }
}

impl IntervalYearMonthType {
    /// Creates a IntervalYearMonthType::Native
    ///
    /// # Arguments
    ///
    /// * `years` - The number of years (+/-) represented in this interval
    /// * `months` - The number of months (+/-) represented in this interval
    pub fn make_value(
        years: i32,
        months: i32,
    ) -> <IntervalYearMonthType as ArrowPrimitiveType>::Native {
        years * 12 + months
    }

    /// Turns a IntervalYearMonthType type into an i32 of months.
    ///
    /// This operation is technically a no-op, it is included for comprehensiveness.
    ///
    /// # Arguments
    ///
    /// * `i` - The IntervalYearMonthType::Native to convert
    pub fn to_months(i: <IntervalYearMonthType as ArrowPrimitiveType>::Native) -> i32 {
        i
    }
}

impl IntervalDayTimeType {
    /// Creates a IntervalDayTimeType::Native
    ///
    /// # Arguments
    ///
    /// * `days` - The number of days (+/-) represented in this interval
    /// * `millis` - The number of milliseconds (+/-) represented in this interval
    pub fn make_value(
        days: i32,
        millis: i32,
    ) -> <IntervalDayTimeType as ArrowPrimitiveType>::Native {
        /*
        https://github.com/apache/arrow/blob/02c8598d264c839a5b5cf3109bfd406f3b8a6ba5/cpp/src/arrow/type.h#L1433
        struct DayMilliseconds {
            int32_t days = 0;
            int32_t milliseconds = 0;
            ...
        }
        64      56      48      40      32      24      16      8       0
        +-------+-------+-------+-------+-------+-------+-------+-------+
        |             days              |         milliseconds          |
        +-------+-------+-------+-------+-------+-------+-------+-------+
        */
        let m = millis as u64 & u32::MAX as u64;
        let d = (days as u64 & u32::MAX as u64) << 32;
        (m | d) as <IntervalDayTimeType as ArrowPrimitiveType>::Native
    }

    /// Turns a IntervalDayTimeType into a tuple of (days, milliseconds)
    ///
    /// # Arguments
    ///
    /// * `i` - The IntervalDayTimeType to convert
    pub fn to_parts(
        i: <IntervalDayTimeType as ArrowPrimitiveType>::Native,
    ) -> (i32, i32) {
        let days = (i >> 32) as i32;
        let ms = i as i32;
        (days, ms)
    }
}

impl IntervalMonthDayNanoType {
    /// Creates a IntervalMonthDayNanoType::Native
    ///
    /// # Arguments
    ///
    /// * `months` - The number of months (+/-) represented in this interval
    /// * `days` - The number of days (+/-) represented in this interval
    /// * `nanos` - The number of nanoseconds (+/-) represented in this interval
    pub fn make_value(
        months: i32,
        days: i32,
        nanos: i64,
    ) -> <IntervalMonthDayNanoType as ArrowPrimitiveType>::Native {
        /*
        https://github.com/apache/arrow/blob/02c8598d264c839a5b5cf3109bfd406f3b8a6ba5/cpp/src/arrow/type.h#L1475
        struct MonthDayNanos {
            int32_t months;
            int32_t days;
            int64_t nanoseconds;
        }
        128     112     96      80      64      48      32      16      0
        +-------+-------+-------+-------+-------+-------+-------+-------+
        |     months    |      days     |             nanos             |
        +-------+-------+-------+-------+-------+-------+-------+-------+
        */
        let m = (months as u128 & u32::MAX as u128) << 96;
        let d = (days as u128 & u32::MAX as u128) << 64;
        let n = nanos as u128 & u64::MAX as u128;
        (m | d | n) as <IntervalMonthDayNanoType as ArrowPrimitiveType>::Native
    }

    /// Turns a IntervalMonthDayNanoType into a tuple of (months, days, nanos)
    ///
    /// # Arguments
    ///
    /// * `i` - The IntervalMonthDayNanoType to convert
    pub fn to_parts(
        i: <IntervalMonthDayNanoType as ArrowPrimitiveType>::Native,
    ) -> (i32, i32, i64) {
        let months = (i >> 96) as i32;
        let days = (i >> 64) as i32;
        let nanos = i as i64;
        (months, days, nanos)
    }
}

impl Date32Type {
    /// Converts an arrow Date32Type into a chrono::NaiveDate
    ///
    /// # Arguments
    ///
    /// * `i` - The Date32Type to convert
    pub fn to_naive_date(i: <Date32Type as ArrowPrimitiveType>::Native) -> NaiveDate {
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        epoch.add(Duration::days(i as i64))
    }

    /// Converts a chrono::NaiveDate into an arrow Date32Type
    ///
    /// # Arguments
    ///
    /// * `d` - The NaiveDate to convert
    pub fn from_naive_date(d: NaiveDate) -> <Date32Type as ArrowPrimitiveType>::Native {
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        d.sub(epoch).num_days() as <Date32Type as ArrowPrimitiveType>::Native
    }

    /// Adds the given IntervalYearMonthType to an arrow Date32Type
    ///
    /// # Arguments
    ///
    /// * `date` - The date on which to perform the operation
    /// * `delta` - The interval to add
    pub fn add_year_months(
        date: <Date32Type as ArrowPrimitiveType>::Native,
        delta: <IntervalYearMonthType as ArrowPrimitiveType>::Native,
    ) -> <Date32Type as ArrowPrimitiveType>::Native {
        let prior = Date32Type::to_naive_date(date);
        let months = IntervalYearMonthType::to_months(delta);
        let posterior = shift_months(prior, months);
        Date32Type::from_naive_date(posterior)
    }

    /// Adds the given IntervalDayTimeType to an arrow Date32Type
    ///
    /// # Arguments
    ///
    /// * `date` - The date on which to perform the operation
    /// * `delta` - The interval to add
    pub fn add_day_time(
        date: <Date32Type as ArrowPrimitiveType>::Native,
        delta: <IntervalDayTimeType as ArrowPrimitiveType>::Native,
    ) -> <Date32Type as ArrowPrimitiveType>::Native {
        let (days, ms) = IntervalDayTimeType::to_parts(delta);
        let res = Date32Type::to_naive_date(date);
        let res = res.add(Duration::days(days as i64));
        let res = res.add(Duration::milliseconds(ms as i64));
        Date32Type::from_naive_date(res)
    }

    /// Adds the given IntervalMonthDayNanoType to an arrow Date32Type
    ///
    /// # Arguments
    ///
    /// * `date` - The date on which to perform the operation
    /// * `delta` - The interval to add
    pub fn add_month_day_nano(
        date: <Date32Type as ArrowPrimitiveType>::Native,
        delta: <IntervalMonthDayNanoType as ArrowPrimitiveType>::Native,
    ) -> <Date32Type as ArrowPrimitiveType>::Native {
        let (months, days, nanos) = IntervalMonthDayNanoType::to_parts(delta);
        let res = Date32Type::to_naive_date(date);
        let res = shift_months(res, months);
        let res = res.add(Duration::days(days as i64));
        let res = res.add(Duration::nanoseconds(nanos));
        Date32Type::from_naive_date(res)
    }
}

impl Date64Type {
    /// Converts an arrow Date64Type into a chrono::NaiveDate
    ///
    /// # Arguments
    ///
    /// * `i` - The Date64Type to convert
    pub fn to_naive_date(i: <Date64Type as ArrowPrimitiveType>::Native) -> NaiveDate {
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        epoch.add(Duration::milliseconds(i))
    }

    /// Converts a chrono::NaiveDate into an arrow Date64Type
    ///
    /// # Arguments
    ///
    /// * `d` - The NaiveDate to convert
    pub fn from_naive_date(d: NaiveDate) -> <Date64Type as ArrowPrimitiveType>::Native {
        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        d.sub(epoch).num_milliseconds() as <Date64Type as ArrowPrimitiveType>::Native
    }

    /// Adds the given IntervalYearMonthType to an arrow Date64Type
    ///
    /// # Arguments
    ///
    /// * `date` - The date on which to perform the operation
    /// * `delta` - The interval to add
    pub fn add_year_months(
        date: <Date64Type as ArrowPrimitiveType>::Native,
        delta: <IntervalYearMonthType as ArrowPrimitiveType>::Native,
    ) -> <Date64Type as ArrowPrimitiveType>::Native {
        let prior = Date64Type::to_naive_date(date);
        let months = IntervalYearMonthType::to_months(delta);
        let posterior = shift_months(prior, months);
        Date64Type::from_naive_date(posterior)
    }

    /// Adds the given IntervalDayTimeType to an arrow Date64Type
    ///
    /// # Arguments
    ///
    /// * `date` - The date on which to perform the operation
    /// * `delta` - The interval to add
    pub fn add_day_time(
        date: <Date64Type as ArrowPrimitiveType>::Native,
        delta: <IntervalDayTimeType as ArrowPrimitiveType>::Native,
    ) -> <Date64Type as ArrowPrimitiveType>::Native {
        let (days, ms) = IntervalDayTimeType::to_parts(delta);
        let res = Date64Type::to_naive_date(date);
        let res = res.add(Duration::days(days as i64));
        let res = res.add(Duration::milliseconds(ms as i64));
        Date64Type::from_naive_date(res)
    }

    /// Adds the given IntervalMonthDayNanoType to an arrow Date64Type
    ///
    /// # Arguments
    ///
    /// * `date` - The date on which to perform the operation
    /// * `delta` - The interval to add
    pub fn add_month_day_nano(
        date: <Date64Type as ArrowPrimitiveType>::Native,
        delta: <IntervalMonthDayNanoType as ArrowPrimitiveType>::Native,
    ) -> <Date64Type as ArrowPrimitiveType>::Native {
        let (months, days, nanos) = IntervalMonthDayNanoType::to_parts(delta);
        let res = Date64Type::to_naive_date(date);
        let res = shift_months(res, months);
        let res = res.add(Duration::days(days as i64));
        let res = res.add(Duration::nanoseconds(nanos));
        Date64Type::from_naive_date(res)
    }
}

/// Crate private types for Decimal Arrays
///
/// Not intended to be used outside this crate
mod decimal {
    use super::*;

    pub trait DecimalTypeSealed {}
    impl DecimalTypeSealed for Decimal128Type {}
    impl DecimalTypeSealed for Decimal256Type {}
}

/// A trait over the decimal types, used by [`PrimitiveArray`] to provide a generic
/// implementation across the various decimal types
///
/// Implemented by [`Decimal128Type`] and [`Decimal256Type`] for [`Decimal128Array`]
/// and [`Decimal256Array`] respectively
///
/// [`PrimitiveArray`]: crate::array::PrimitiveArray
/// [`Decimal128Array`]: crate::array::Decimal128Array
/// [`Decimal256Array`]: crate::array::Decimal256Array
pub trait DecimalType:
    'static + Send + Sync + ArrowPrimitiveType + decimal::DecimalTypeSealed
{
    /// Width of the type
    const BYTE_LENGTH: usize;
    /// Maximum number of significant digits
    const MAX_PRECISION: u8;
    /// Maximum no of digits after the decimal point (note the scale can be negative)
    const MAX_SCALE: i8;
    /// fn to create its [`DataType`]
    const TYPE_CONSTRUCTOR: fn(u8, i8) -> DataType;
    /// Default values for [`DataType`]
    const DEFAULT_TYPE: DataType;

    /// "Decimal128" or "Decimal256", for use in error messages
    const PREFIX: &'static str;

    /// Formats the decimal value with the provided precision and scale
    fn format_decimal(value: Self::Native, precision: u8, scale: i8) -> String;

    /// Validates that `value` contains no more than `precision` decimal digits
    fn validate_decimal_precision(
        value: Self::Native,
        precision: u8,
    ) -> Result<(), ArrowError>;
}

/// The decimal type for a Decimal128Array
#[derive(Debug)]
pub struct Decimal128Type {}

impl DecimalType for Decimal128Type {
    const BYTE_LENGTH: usize = 16;
    const MAX_PRECISION: u8 = DECIMAL128_MAX_PRECISION;
    const MAX_SCALE: i8 = DECIMAL128_MAX_SCALE;
    const TYPE_CONSTRUCTOR: fn(u8, i8) -> DataType = DataType::Decimal128;
    const DEFAULT_TYPE: DataType =
        DataType::Decimal128(DECIMAL128_MAX_PRECISION, DECIMAL_DEFAULT_SCALE);
    const PREFIX: &'static str = "Decimal128";

    fn format_decimal(value: Self::Native, precision: u8, scale: i8) -> String {
        format_decimal_str(&value.to_string(), precision as usize, scale)
    }

    fn validate_decimal_precision(num: i128, precision: u8) -> Result<(), ArrowError> {
        validate_decimal_precision(num, precision)
    }
}

impl ArrowPrimitiveType for Decimal128Type {
    type Native = i128;

    const DATA_TYPE: DataType = <Self as DecimalType>::DEFAULT_TYPE;
}

/// The decimal type for a Decimal256Array
#[derive(Debug)]
pub struct Decimal256Type {}

impl DecimalType for Decimal256Type {
    const BYTE_LENGTH: usize = 32;
    const MAX_PRECISION: u8 = DECIMAL256_MAX_PRECISION;
    const MAX_SCALE: i8 = DECIMAL256_MAX_SCALE;
    const TYPE_CONSTRUCTOR: fn(u8, i8) -> DataType = DataType::Decimal256;
    const DEFAULT_TYPE: DataType =
        DataType::Decimal256(DECIMAL256_MAX_PRECISION, DECIMAL_DEFAULT_SCALE);
    const PREFIX: &'static str = "Decimal256";

    fn format_decimal(value: Self::Native, precision: u8, scale: i8) -> String {
        format_decimal_str(&value.to_string(), precision as usize, scale)
    }

    fn validate_decimal_precision(num: i256, precision: u8) -> Result<(), ArrowError> {
        validate_decimal256_precision(num, precision)
    }
}

impl ArrowPrimitiveType for Decimal256Type {
    type Native = i256;

    const DATA_TYPE: DataType = <Self as DecimalType>::DEFAULT_TYPE;
}

fn format_decimal_str(value_str: &str, precision: usize, scale: i8) -> String {
    let (sign, rest) = match value_str.strip_prefix('-') {
        Some(stripped) => ("-", stripped),
        None => ("", value_str),
    };
    let bound = precision.min(rest.len()) + sign.len();
    let value_str = &value_str[0..bound];

    if scale == 0 {
        value_str.to_string()
    } else if scale < 0 {
        let padding = value_str.len() + scale.unsigned_abs() as usize;
        format!("{value_str:0<padding$}")
    } else if rest.len() > scale as usize {
        // Decimal separator is in the middle of the string
        let (whole, decimal) = value_str.split_at(value_str.len() - scale as usize);
        format!("{whole}.{decimal}")
    } else {
        // String has to be padded
        format!("{}0.{:0>width$}", sign, rest, width = scale as usize)
    }
}

lazy_static! {
    static ref PARSE_DECIMAL_RE: Regex =
        Regex::new(r"^-?(\d+\.?\d*|\d*\.?\d+)$").unwrap();
}

/// Parse the string format decimal value to i128/i256 format and checking the precision and scale.
/// The result value can't be out of bounds.
pub fn parse_decimal_with_parameter<T: DecimalType>(
    s: &str,
    precision: u8,
    scale: i8,
) -> Result<T::Native, ArrowError> {
    if PARSE_DECIMAL_RE.is_match(s) {
        let mut offset = s.len();
        let len = s.len();
        let mut base = T::Native::usize_as(1);
        let scale_usize = usize::from(scale as u8);

        // handle the value after the '.' and meet the scale
        let delimiter_position = s.find('.');
        match delimiter_position {
            None => {
                // there is no '.'
                base = T::Native::usize_as(10).pow_checked(scale as u32)?;
            }
            Some(mid) => {
                // there is the '.'
                if len - mid >= scale_usize + 1 {
                    // If the string value is "123.12345" and the scale is 2, we should just remain '.12' and drop the '345' value.
                    offset -= len - mid - 1 - scale_usize;
                } else {
                    // If the string value is "123.12" and the scale is 4, we should append '00' to the tail.
                    base = T::Native::usize_as(10)
                        .pow_checked((scale_usize + 1 + mid - len) as u32)?;
                }
            }
        };

        // each byte is digit、'-' or '.'
        let bytes = s.as_bytes();
        let mut negative = false;
        let mut result = T::Native::usize_as(0);

        bytes[0..offset]
            .iter()
            .rev()
            .try_for_each::<_, Result<(), ArrowError>>(|&byte| {
                match byte {
                    b'-' => {
                        negative = true;
                    }
                    b'0'..=b'9' => {
                        let add = T::Native::usize_as((byte - b'0') as usize)
                            .mul_checked(base)?;
                        result = result.add_checked(add)?;
                        base = base.mul_checked(T::Native::usize_as(10))?;
                    }
                    // because of the PARSE_DECIMAL_RE, bytes just contains digit、'-' and '.'.
                    _ => (),
                }
                Ok(())
            })?;

        if negative {
            result = result.neg_checked()?;
        }

        match T::validate_decimal_precision(result, precision) {
            Ok(_) => Ok(result),
            Err(e) => Err(ArrowError::ParseError(format!(
                "parse decimal overflow: {e}"
            ))),
        }
    } else {
        Err(ArrowError::ParseError(format!(
            "can't parse the string value {s} to decimal"
        )))
    }
}

// Parse the string format decimal value to i128 format without checking the precision and scale.
// Like "125.12" to 12512_i128.
#[cfg(test)]
fn parse_decimal(s: &str) -> Result<i128, ArrowError> {
    use std::ops::Neg;

    if PARSE_DECIMAL_RE.is_match(s) {
        let mut offset = s.len();
        // each byte is digit、'-' or '.'
        let bytes = s.as_bytes();
        let mut negative = false;
        let mut result: i128 = 0;
        let mut base = 1;
        while offset > 0 {
            match bytes[offset - 1] {
                b'-' => {
                    negative = true;
                }
                b'.' => {
                    // do nothing
                }
                b'0'..=b'9' => {
                    result += i128::from(bytes[offset - 1] - b'0') * base;
                    base *= 10;
                }
                _ => {
                    return Err(ArrowError::ParseError(format!(
                        "can't match byte {}",
                        bytes[offset - 1]
                    )));
                }
            }
            offset -= 1;
        }
        if negative {
            Ok(result.neg())
        } else {
            Ok(result)
        }
    } else {
        Err(ArrowError::ParseError(format!(
            "can't parse the string value {s} to decimal"
        )))
    }
}

/// Crate private types for Byte Arrays
///
/// Not intended to be used outside this crate
pub(crate) mod bytes {
    use super::*;

    pub trait ByteArrayTypeSealed {}
    impl<O: OffsetSizeTrait> ByteArrayTypeSealed for GenericStringType<O> {}
    impl<O: OffsetSizeTrait> ByteArrayTypeSealed for GenericBinaryType<O> {}

    pub trait ByteArrayNativeType: std::fmt::Debug + Send + Sync {
        /// # Safety
        ///
        /// `b` must be a valid byte sequence for `Self`
        unsafe fn from_bytes_unchecked(b: &[u8]) -> &Self;
    }

    impl ByteArrayNativeType for [u8] {
        unsafe fn from_bytes_unchecked(b: &[u8]) -> &Self {
            b
        }
    }

    impl ByteArrayNativeType for str {
        unsafe fn from_bytes_unchecked(b: &[u8]) -> &Self {
            std::str::from_utf8_unchecked(b)
        }
    }
}

/// A trait over the variable-size byte array types
///
/// See [Variable Size Binary Layout](https://arrow.apache.org/docs/format/Columnar.html#variable-size-binary-layout)
pub trait ByteArrayType: 'static + Send + Sync + bytes::ByteArrayTypeSealed {
    /// Type of offset i.e i32/i64
    type Offset: OffsetSizeTrait;
    /// Type for representing its equivalent rust type i.e
    /// Utf8Array will have native type has &str
    /// BinaryArray will have type as [u8]
    type Native: bytes::ByteArrayNativeType + AsRef<Self::Native> + AsRef<[u8]> + ?Sized;
    /// "Binary" or "String", for use in error messages
    const PREFIX: &'static str;
    /// Datatype of array elements
    const DATA_TYPE: DataType;
}

/// [`ByteArrayType`] for string arrays
pub struct GenericStringType<O: OffsetSizeTrait> {
    phantom: PhantomData<O>,
}

impl<O: OffsetSizeTrait> ByteArrayType for GenericStringType<O> {
    type Offset = O;
    type Native = str;
    const PREFIX: &'static str = "String";

    const DATA_TYPE: DataType = if O::IS_LARGE {
        DataType::LargeUtf8
    } else {
        DataType::Utf8
    };
}

/// An arrow utf8 array with i32 offsets
pub type Utf8Type = GenericStringType<i32>;
/// An arrow utf8 array with i64 offsets
pub type LargeUtf8Type = GenericStringType<i64>;

/// [`ByteArrayType`] for binary arrays
pub struct GenericBinaryType<O: OffsetSizeTrait> {
    phantom: PhantomData<O>,
}

impl<O: OffsetSizeTrait> ByteArrayType for GenericBinaryType<O> {
    type Offset = O;
    type Native = [u8];
    const PREFIX: &'static str = "Binary";

    const DATA_TYPE: DataType = if O::IS_LARGE {
        DataType::LargeBinary
    } else {
        DataType::Binary
    };
}

/// An arrow binary array with i32 offsets
pub type BinaryType = GenericBinaryType<i32>;
/// An arrow binary array with i64 offsets
pub type LargeBinaryType = GenericBinaryType<i64>;

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_data::{layout, BufferSpec};
    use std::mem::size_of;

    #[test]
    fn month_day_nano_should_roundtrip() {
        let value = IntervalMonthDayNanoType::make_value(1, 2, 3);
        assert_eq!(IntervalMonthDayNanoType::to_parts(value), (1, 2, 3));
    }

    #[test]
    fn month_day_nano_should_roundtrip_neg() {
        let value = IntervalMonthDayNanoType::make_value(-1, -2, -3);
        assert_eq!(IntervalMonthDayNanoType::to_parts(value), (-1, -2, -3));
    }

    #[test]
    fn day_time_should_roundtrip() {
        let value = IntervalDayTimeType::make_value(1, 2);
        assert_eq!(IntervalDayTimeType::to_parts(value), (1, 2));
    }

    #[test]
    fn day_time_should_roundtrip_neg() {
        let value = IntervalDayTimeType::make_value(-1, -2);
        assert_eq!(IntervalDayTimeType::to_parts(value), (-1, -2));
    }

    #[test]
    fn year_month_should_roundtrip() {
        let value = IntervalYearMonthType::make_value(1, 2);
        assert_eq!(IntervalYearMonthType::to_months(value), 14);
    }

    #[test]
    fn year_month_should_roundtrip_neg() {
        let value = IntervalYearMonthType::make_value(-1, -2);
        assert_eq!(IntervalYearMonthType::to_months(value), -14);
    }

    fn test_layout<T: ArrowPrimitiveType>() {
        let layout = layout(&T::DATA_TYPE);

        assert_eq!(layout.buffers.len(), 1);

        let spec = &layout.buffers[0];
        assert_eq!(
            spec,
            &BufferSpec::FixedWidth {
                byte_width: size_of::<T::Native>()
            }
        );
    }

    #[test]
    fn test_layouts() {
        test_layout::<Int8Type>();
        test_layout::<Int16Type>();
        test_layout::<Int32Type>();
        test_layout::<Int64Type>();
        test_layout::<UInt8Type>();
        test_layout::<UInt16Type>();
        test_layout::<UInt32Type>();
        test_layout::<UInt64Type>();
        test_layout::<Float16Type>();
        test_layout::<Float32Type>();
        test_layout::<Float64Type>();
        test_layout::<TimestampSecondType>();
        test_layout::<Date32Type>();
        test_layout::<Date64Type>();
        test_layout::<Time32SecondType>();
        test_layout::<Time32MillisecondType>();
        test_layout::<Time64MicrosecondType>();
        test_layout::<Time64NanosecondType>();
        test_layout::<IntervalMonthDayNanoType>();
        test_layout::<IntervalDayTimeType>();
        test_layout::<IntervalYearMonthType>();
        test_layout::<DurationNanosecondType>();
        test_layout::<DurationMicrosecondType>();
        test_layout::<DurationMillisecondType>();
    }

    #[test]
    fn test_parse_decimal() {
        let tests = [
            ("123.00", 12300i128),
            ("123.123", 123123i128),
            ("0.0123", 123i128),
            ("0.12300", 12300i128),
            ("-5.123", -5123i128),
            ("-45.432432", -45432432i128),
        ];
        for (s, i) in tests {
            let result = parse_decimal(s);
            assert_eq!(i, result.unwrap());
        }
    }

    #[test]
    fn test_parse_decimal_with_parameter() {
        let tests = [
            ("123.123", 123123i128),
            ("123.1234", 123123i128),
            ("123.1", 123100i128),
            ("123", 123000i128),
            ("-123.123", -123123i128),
            ("-123.1234", -123123i128),
            ("-123.1", -123100i128),
            ("-123", -123000i128),
            ("0.0000123", 0i128),
            ("12.", 12000i128),
            ("-12.", -12000i128),
            ("00.1", 100i128),
            ("-00.1", -100i128),
            ("12345678912345678.1234", 12345678912345678123i128),
            ("-12345678912345678.1234", -12345678912345678123i128),
            ("99999999999999999.999", 99999999999999999999i128),
            ("-99999999999999999.999", -99999999999999999999i128),
            (".123", 123i128),
            ("-.123", -123i128),
            ("123.", 123000i128),
            ("-123.", -123000i128),
        ];
        for (s, i) in tests {
            let result_128 = parse_decimal_with_parameter::<Decimal128Type>(s, 20, 3);
            assert_eq!(i, result_128.unwrap());
            let result_256 = parse_decimal_with_parameter::<Decimal256Type>(s, 20, 3);
            assert_eq!(i256::from_i128(i), result_256.unwrap());
        }
        let can_not_parse_tests = ["123,123", ".", "123.123.123"];
        for s in can_not_parse_tests {
            let result_128 = parse_decimal_with_parameter::<Decimal128Type>(s, 20, 3);
            assert_eq!(
                format!("Parser error: can't parse the string value {s} to decimal"),
                result_128.unwrap_err().to_string()
            );
            let result_256 = parse_decimal_with_parameter::<Decimal256Type>(s, 20, 3);
            assert_eq!(
                format!("Parser error: can't parse the string value {s} to decimal"),
                result_256.unwrap_err().to_string()
            );
        }
        let overflow_parse_tests = ["12345678", "12345678.9", "99999999.99"];
        for s in overflow_parse_tests {
            let result_128 = parse_decimal_with_parameter::<Decimal128Type>(s, 10, 3);
            let expected_128 = "Parser error: parse decimal overflow";
            let actual_128 = result_128.unwrap_err().to_string();

            assert!(
                actual_128.contains(expected_128),
                "actual: '{actual_128}', expected: '{expected_128}'"
            );

            let result_256 = parse_decimal_with_parameter::<Decimal256Type>(s, 10, 3);
            let expected_256 = "Parser error: parse decimal overflow";
            let actual_256 = result_256.unwrap_err().to_string();

            assert!(
                actual_256.contains(expected_256),
                "actual: '{actual_256}', expected: '{expected_256}'"
            );
        }
    }
}
