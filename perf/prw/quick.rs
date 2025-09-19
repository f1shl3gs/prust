// Automatically generated rust module for 'remote.proto' file

#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::borrow::Cow;
use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

mod prometheus {
    // Automatically generated rust module for 'types.proto' file

    #![allow(non_snake_case)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(unused_imports)]
    #![allow(unknown_lints)]
    #![allow(clippy::all)]
    #![cfg_attr(rustfmt, rustfmt_skip)]


    use std::borrow::Cow;
    use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
    use quick_protobuf::sizeofs::*;
    use super::*;

    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct MetricMetadata<'a> {
        pub type_pb: prometheus::mod_MetricMetadata::MetricType,
        pub metric_family_name: Cow<'a, str>,
        pub help: Cow<'a, str>,
        pub unit: Cow<'a, str>,
    }

    impl<'a> MessageRead<'a> for MetricMetadata<'a> {
        fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(8) => msg.type_pb = r.read_enum(bytes)?,
                    Ok(18) => msg.metric_family_name = r.read_string(bytes).map(Cow::Borrowed)?,
                    Ok(34) => msg.help = r.read_string(bytes).map(Cow::Borrowed)?,
                    Ok(42) => msg.unit = r.read_string(bytes).map(Cow::Borrowed)?,
                    Ok(t) => { r.read_unknown(bytes, t)?; }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl<'a> MessageWrite for MetricMetadata<'a> {
        fn get_size(&self) -> usize {
            0
                + if self.type_pb == prometheus::mod_MetricMetadata::MetricType::UNKNOWN { 0 } else { 1 + sizeof_varint(*(&self.type_pb) as u64) }
                + if self.metric_family_name == "" { 0 } else { 1 + sizeof_len((&self.metric_family_name).len()) }
                + if self.help == "" { 0 } else { 1 + sizeof_len((&self.help).len()) }
                + if self.unit == "" { 0 } else { 1 + sizeof_len((&self.unit).len()) }
        }

        fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
            if self.type_pb != prometheus::mod_MetricMetadata::MetricType::UNKNOWN { w.write_with_tag(8, |w| w.write_enum(*&self.type_pb as i32))?; }
            if self.metric_family_name != "" { w.write_with_tag(18, |w| w.write_string(&**&self.metric_family_name))?; }
            if self.help != "" { w.write_with_tag(34, |w| w.write_string(&**&self.help))?; }
            if self.unit != "" { w.write_with_tag(42, |w| w.write_string(&**&self.unit))?; }
            Ok(())
        }
    }

    pub mod mod_MetricMetadata {


        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub enum MetricType {
            UNKNOWN = 0,
            COUNTER = 1,
            GAUGE = 2,
            HISTOGRAM = 3,
            GAUGEHISTOGRAM = 4,
            SUMMARY = 5,
            INFO = 6,
            STATESET = 7,
        }

        impl Default for MetricType {
            fn default() -> Self {
                MetricType::UNKNOWN
            }
        }

        impl From<i32> for MetricType {
            fn from(i: i32) -> Self {
                match i {
                    0 => MetricType::UNKNOWN,
                    1 => MetricType::COUNTER,
                    2 => MetricType::GAUGE,
                    3 => MetricType::HISTOGRAM,
                    4 => MetricType::GAUGEHISTOGRAM,
                    5 => MetricType::SUMMARY,
                    6 => MetricType::INFO,
                    7 => MetricType::STATESET,
                    _ => Self::default(),
                }
            }
        }

        impl<'a> From<&'a str> for MetricType {
            fn from(s: &'a str) -> Self {
                match s {
                    "UNKNOWN" => MetricType::UNKNOWN,
                    "COUNTER" => MetricType::COUNTER,
                    "GAUGE" => MetricType::GAUGE,
                    "HISTOGRAM" => MetricType::HISTOGRAM,
                    "GAUGEHISTOGRAM" => MetricType::GAUGEHISTOGRAM,
                    "SUMMARY" => MetricType::SUMMARY,
                    "INFO" => MetricType::INFO,
                    "STATESET" => MetricType::STATESET,
                    _ => Self::default(),
                }
            }
        }

    }

    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct Sample {
        pub value: f64,
        pub timestamp: i64,
    }

    impl<'a> MessageRead<'a> for Sample {
        fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(9) => msg.value = r.read_double(bytes)?,
                    Ok(16) => msg.timestamp = r.read_int64(bytes)?,
                    Ok(t) => { r.read_unknown(bytes, t)?; }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl MessageWrite for Sample {
        fn get_size(&self) -> usize {
            0
                + if self.value == 0f64 { 0 } else { 1 + 8 }
                + if self.timestamp == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.timestamp) as u64) }
        }

        fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
            if self.value != 0f64 { w.write_with_tag(9, |w| w.write_double(*&self.value))?; }
            if self.timestamp != 0i64 { w.write_with_tag(16, |w| w.write_int64(*&self.timestamp))?; }
            Ok(())
        }
    }

    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct Exemplar<'a> {
        pub labels: Vec<prometheus::Label<'a>>,
        pub value: f64,
        pub timestamp: i64,
    }

    impl<'a> MessageRead<'a> for Exemplar<'a> {
        fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(10) => msg.labels.push(r.read_message::<prometheus::Label>(bytes)?),
                    Ok(17) => msg.value = r.read_double(bytes)?,
                    Ok(24) => msg.timestamp = r.read_int64(bytes)?,
                    Ok(t) => { r.read_unknown(bytes, t)?; }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl<'a> MessageWrite for Exemplar<'a> {
        fn get_size(&self) -> usize {
            0
                + self.labels.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
                + if self.value == 0f64 { 0 } else { 1 + 8 }
                + if self.timestamp == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.timestamp) as u64) }
        }

        fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
            for s in &self.labels { w.write_with_tag(10, |w| w.write_message(s))?; }
            if self.value != 0f64 { w.write_with_tag(17, |w| w.write_double(*&self.value))?; }
            if self.timestamp != 0i64 { w.write_with_tag(24, |w| w.write_int64(*&self.timestamp))?; }
            Ok(())
        }
    }

    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct Histogram<'a> {
        pub sum: f64,
        pub schema: i32,
        pub zero_threshold: f64,
        pub negative_spans: Vec<prometheus::BucketSpan>,
        pub negative_deltas: Vec<i64>,
        pub negative_counts: Cow<'a, [f64]>,
        pub positive_spans: Vec<prometheus::BucketSpan>,
        pub positive_deltas: Vec<i64>,
        pub positive_counts: Cow<'a, [f64]>,
        pub reset_hint: prometheus::mod_Histogram::ResetHint,
        pub timestamp: i64,
        pub custom_values: Cow<'a, [f64]>,
        pub count: prometheus::mod_Histogram::OneOfcount,
        pub zero_count: prometheus::mod_Histogram::OneOfzero_count,
    }

    impl<'a> MessageRead<'a> for Histogram<'a> {
        fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(25) => msg.sum = r.read_double(bytes)?,
                    Ok(32) => msg.schema = r.read_sint32(bytes)?,
                    Ok(41) => msg.zero_threshold = r.read_double(bytes)?,
                    Ok(66) => msg.negative_spans.push(r.read_message::<prometheus::BucketSpan>(bytes)?),
                    Ok(74) => msg.negative_deltas = r.read_packed(bytes, |r, bytes| Ok(r.read_sint64(bytes)?))?,
                    Ok(82) => msg.negative_counts = r.read_packed_fixed(bytes)?.into(),
                    Ok(90) => msg.positive_spans.push(r.read_message::<prometheus::BucketSpan>(bytes)?),
                    Ok(98) => msg.positive_deltas = r.read_packed(bytes, |r, bytes| Ok(r.read_sint64(bytes)?))?,
                    Ok(106) => msg.positive_counts = r.read_packed_fixed(bytes)?.into(),
                    Ok(112) => msg.reset_hint = r.read_enum(bytes)?,
                    Ok(120) => msg.timestamp = r.read_int64(bytes)?,
                    Ok(130) => msg.custom_values = r.read_packed_fixed(bytes)?.into(),
                    Ok(8) => msg.count = prometheus::mod_Histogram::OneOfcount::count_int(r.read_uint64(bytes)?),
                    Ok(17) => msg.count = prometheus::mod_Histogram::OneOfcount::count_float(r.read_double(bytes)?),
                    Ok(48) => msg.zero_count = prometheus::mod_Histogram::OneOfzero_count::zero_count_int(r.read_uint64(bytes)?),
                    Ok(57) => msg.zero_count = prometheus::mod_Histogram::OneOfzero_count::zero_count_float(r.read_double(bytes)?),
                    Ok(t) => { r.read_unknown(bytes, t)?; }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl<'a> MessageWrite for Histogram<'a> {
        fn get_size(&self) -> usize {
            0
                + if self.sum == 0f64 { 0 } else { 1 + 8 }
                + if self.schema == 0i32 { 0 } else { 1 + sizeof_sint32(*(&self.schema)) }
                + if self.zero_threshold == 0f64 { 0 } else { 1 + 8 }
                + self.negative_spans.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
                + if self.negative_deltas.is_empty() { 0 } else { 1 + sizeof_len(self.negative_deltas.iter().map(|s| sizeof_sint64(*(s))).sum::<usize>()) }
                + if self.negative_counts.is_empty() { 0 } else { 1 + sizeof_len(self.negative_counts.len() * 8) }
                + self.positive_spans.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
                + if self.positive_deltas.is_empty() { 0 } else { 1 + sizeof_len(self.positive_deltas.iter().map(|s| sizeof_sint64(*(s))).sum::<usize>()) }
                + if self.positive_counts.is_empty() { 0 } else { 1 + sizeof_len(self.positive_counts.len() * 8) }
                + if self.reset_hint == prometheus::mod_Histogram::ResetHint::UNKNOWN { 0 } else { 1 + sizeof_varint(*(&self.reset_hint) as u64) }
                + if self.timestamp == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.timestamp) as u64) }
                + if self.custom_values.is_empty() { 0 } else { 2 + sizeof_len(self.custom_values.len() * 8) }
                + match self.count {
                prometheus::mod_Histogram::OneOfcount::count_int(ref m) => 1 + sizeof_varint(*(m) as u64),
                prometheus::mod_Histogram::OneOfcount::count_float(_) => 1 + 8,
                prometheus::mod_Histogram::OneOfcount::None => 0,
            }        + match self.zero_count {
                prometheus::mod_Histogram::OneOfzero_count::zero_count_int(ref m) => 1 + sizeof_varint(*(m) as u64),
                prometheus::mod_Histogram::OneOfzero_count::zero_count_float(_) => 1 + 8,
                prometheus::mod_Histogram::OneOfzero_count::None => 0,
            }    }

        fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
            if self.sum != 0f64 { w.write_with_tag(25, |w| w.write_double(*&self.sum))?; }
            if self.schema != 0i32 { w.write_with_tag(32, |w| w.write_sint32(*&self.schema))?; }
            if self.zero_threshold != 0f64 { w.write_with_tag(41, |w| w.write_double(*&self.zero_threshold))?; }
            for s in &self.negative_spans { w.write_with_tag(66, |w| w.write_message(s))?; }
            w.write_packed_with_tag(74, &self.negative_deltas, |w, m| w.write_sint64(*m), &|m| sizeof_sint64(*(m)))?;
            w.write_packed_fixed_with_tag(82, &self.negative_counts)?;
            for s in &self.positive_spans { w.write_with_tag(90, |w| w.write_message(s))?; }
            w.write_packed_with_tag(98, &self.positive_deltas, |w, m| w.write_sint64(*m), &|m| sizeof_sint64(*(m)))?;
            w.write_packed_fixed_with_tag(106, &self.positive_counts)?;
            if self.reset_hint != prometheus::mod_Histogram::ResetHint::UNKNOWN { w.write_with_tag(112, |w| w.write_enum(*&self.reset_hint as i32))?; }
            if self.timestamp != 0i64 { w.write_with_tag(120, |w| w.write_int64(*&self.timestamp))?; }
            w.write_packed_fixed_with_tag(130, &self.custom_values)?;
            match self.count {            prometheus::mod_Histogram::OneOfcount::count_int(ref m) => { w.write_with_tag(8, |w| w.write_uint64(*m))? },
                prometheus::mod_Histogram::OneOfcount::count_float(ref m) => { w.write_with_tag(17, |w| w.write_double(*m))? },
                prometheus::mod_Histogram::OneOfcount::None => {},
            }        match self.zero_count {            prometheus::mod_Histogram::OneOfzero_count::zero_count_int(ref m) => { w.write_with_tag(48, |w| w.write_uint64(*m))? },
                prometheus::mod_Histogram::OneOfzero_count::zero_count_float(ref m) => { w.write_with_tag(57, |w| w.write_double(*m))? },
                prometheus::mod_Histogram::OneOfzero_count::None => {},
            }        Ok(())
        }
    }

    pub mod mod_Histogram {

        use super::*;

        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub enum ResetHint {
            UNKNOWN = 0,
            YES = 1,
            NO = 2,
            GAUGE = 3,
        }

        impl Default for ResetHint {
            fn default() -> Self {
                ResetHint::UNKNOWN
            }
        }

        impl From<i32> for ResetHint {
            fn from(i: i32) -> Self {
                match i {
                    0 => ResetHint::UNKNOWN,
                    1 => ResetHint::YES,
                    2 => ResetHint::NO,
                    3 => ResetHint::GAUGE,
                    _ => Self::default(),
                }
            }
        }

        impl<'a> From<&'a str> for ResetHint {
            fn from(s: &'a str) -> Self {
                match s {
                    "UNKNOWN" => ResetHint::UNKNOWN,
                    "YES" => ResetHint::YES,
                    "NO" => ResetHint::NO,
                    "GAUGE" => ResetHint::GAUGE,
                    _ => Self::default(),
                }
            }
        }

        #[derive(Debug, PartialEq, Clone)]
        pub enum OneOfcount {
            count_int(u64),
            count_float(f64),
            None,
        }

        impl Default for OneOfcount {
            fn default() -> Self {
                OneOfcount::None
            }
        }

        #[derive(Debug, PartialEq, Clone)]
        pub enum OneOfzero_count {
            zero_count_int(u64),
            zero_count_float(f64),
            None,
        }

        impl Default for OneOfzero_count {
            fn default() -> Self {
                OneOfzero_count::None
            }
        }

    }

    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct BucketSpan {
        pub offset: i32,
        pub length: u32,
    }

    impl<'a> MessageRead<'a> for BucketSpan {
        fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(8) => msg.offset = r.read_sint32(bytes)?,
                    Ok(16) => msg.length = r.read_uint32(bytes)?,
                    Ok(t) => { r.read_unknown(bytes, t)?; }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl MessageWrite for BucketSpan {
        fn get_size(&self) -> usize {
            0
                + if self.offset == 0i32 { 0 } else { 1 + sizeof_sint32(*(&self.offset)) }
                + if self.length == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.length) as u64) }
        }

        fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
            if self.offset != 0i32 { w.write_with_tag(8, |w| w.write_sint32(*&self.offset))?; }
            if self.length != 0u32 { w.write_with_tag(16, |w| w.write_uint32(*&self.length))?; }
            Ok(())
        }
    }

    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct TimeSeries<'a> {
        pub labels: Vec<prometheus::Label<'a>>,
        pub samples: Vec<prometheus::Sample>,
        pub exemplars: Vec<prometheus::Exemplar<'a>>,
        pub histograms: Vec<prometheus::Histogram<'a>>,
    }

    impl<'a> MessageRead<'a> for TimeSeries<'a> {
        fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(10) => msg.labels.push(r.read_message::<prometheus::Label>(bytes)?),
                    Ok(18) => msg.samples.push(r.read_message::<prometheus::Sample>(bytes)?),
                    Ok(26) => msg.exemplars.push(r.read_message::<prometheus::Exemplar>(bytes)?),
                    Ok(34) => msg.histograms.push(r.read_message::<prometheus::Histogram>(bytes)?),
                    Ok(t) => { r.read_unknown(bytes, t)?; }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl<'a> MessageWrite for TimeSeries<'a> {
        fn get_size(&self) -> usize {
            0
                + self.labels.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
                + self.samples.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
                + self.exemplars.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
                + self.histograms.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        }

        fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
            for s in &self.labels { w.write_with_tag(10, |w| w.write_message(s))?; }
            for s in &self.samples { w.write_with_tag(18, |w| w.write_message(s))?; }
            for s in &self.exemplars { w.write_with_tag(26, |w| w.write_message(s))?; }
            for s in &self.histograms { w.write_with_tag(34, |w| w.write_message(s))?; }
            Ok(())
        }
    }

    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct Label<'a> {
        pub name: Cow<'a, str>,
        pub value: Cow<'a, str>,
    }

    impl<'a> MessageRead<'a> for Label<'a> {
        fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(10) => msg.name = r.read_string(bytes).map(Cow::Borrowed)?,
                    Ok(18) => msg.value = r.read_string(bytes).map(Cow::Borrowed)?,
                    Ok(t) => { r.read_unknown(bytes, t)?; }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl<'a> MessageWrite for Label<'a> {
        fn get_size(&self) -> usize {
            0
                + if self.name == "" { 0 } else { 1 + sizeof_len((&self.name).len()) }
                + if self.value == "" { 0 } else { 1 + sizeof_len((&self.value).len()) }
        }

        fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
            if self.name != "" { w.write_with_tag(10, |w| w.write_string(&**&self.name))?; }
            if self.value != "" { w.write_with_tag(18, |w| w.write_string(&**&self.value))?; }
            Ok(())
        }
    }

    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct Labels<'a> {
        pub labels: Vec<prometheus::Label<'a>>,
    }

    impl<'a> MessageRead<'a> for Labels<'a> {
        fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(10) => msg.labels.push(r.read_message::<prometheus::Label>(bytes)?),
                    Ok(t) => { r.read_unknown(bytes, t)?; }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl<'a> MessageWrite for Labels<'a> {
        fn get_size(&self) -> usize {
            0
                + self.labels.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        }

        fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
            for s in &self.labels { w.write_with_tag(10, |w| w.write_message(s))?; }
            Ok(())
        }
    }

    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct LabelMatcher<'a> {
        pub type_pb: prometheus::mod_LabelMatcher::Type,
        pub name: Cow<'a, str>,
        pub value: Cow<'a, str>,
    }

    impl<'a> MessageRead<'a> for LabelMatcher<'a> {
        fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(8) => msg.type_pb = r.read_enum(bytes)?,
                    Ok(18) => msg.name = r.read_string(bytes).map(Cow::Borrowed)?,
                    Ok(26) => msg.value = r.read_string(bytes).map(Cow::Borrowed)?,
                    Ok(t) => { r.read_unknown(bytes, t)?; }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl<'a> MessageWrite for LabelMatcher<'a> {
        fn get_size(&self) -> usize {
            0
                + if self.type_pb == prometheus::mod_LabelMatcher::Type::EQ { 0 } else { 1 + sizeof_varint(*(&self.type_pb) as u64) }
                + if self.name == "" { 0 } else { 1 + sizeof_len((&self.name).len()) }
                + if self.value == "" { 0 } else { 1 + sizeof_len((&self.value).len()) }
        }

        fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
            if self.type_pb != prometheus::mod_LabelMatcher::Type::EQ { w.write_with_tag(8, |w| w.write_enum(*&self.type_pb as i32))?; }
            if self.name != "" { w.write_with_tag(18, |w| w.write_string(&**&self.name))?; }
            if self.value != "" { w.write_with_tag(26, |w| w.write_string(&**&self.value))?; }
            Ok(())
        }
    }

    pub mod mod_LabelMatcher {


        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub enum Type {
            EQ = 0,
            NEQ = 1,
            RE = 2,
            NRE = 3,
        }

        impl Default for Type {
            fn default() -> Self {
                Type::EQ
            }
        }

        impl From<i32> for Type {
            fn from(i: i32) -> Self {
                match i {
                    0 => Type::EQ,
                    1 => Type::NEQ,
                    2 => Type::RE,
                    3 => Type::NRE,
                    _ => Self::default(),
                }
            }
        }

        impl<'a> From<&'a str> for Type {
            fn from(s: &'a str) -> Self {
                match s {
                    "EQ" => Type::EQ,
                    "NEQ" => Type::NEQ,
                    "RE" => Type::RE,
                    "NRE" => Type::NRE,
                    _ => Self::default(),
                }
            }
        }

    }

    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct ReadHints<'a> {
        pub step_ms: i64,
        pub func: Cow<'a, str>,
        pub start_ms: i64,
        pub end_ms: i64,
        pub grouping: Vec<Cow<'a, str>>,
        pub by: bool,
        pub range_ms: i64,
    }

    impl<'a> MessageRead<'a> for ReadHints<'a> {
        fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(8) => msg.step_ms = r.read_int64(bytes)?,
                    Ok(18) => msg.func = r.read_string(bytes).map(Cow::Borrowed)?,
                    Ok(24) => msg.start_ms = r.read_int64(bytes)?,
                    Ok(32) => msg.end_ms = r.read_int64(bytes)?,
                    Ok(42) => msg.grouping.push(r.read_string(bytes).map(Cow::Borrowed)?),
                    Ok(48) => msg.by = r.read_bool(bytes)?,
                    Ok(56) => msg.range_ms = r.read_int64(bytes)?,
                    Ok(t) => { r.read_unknown(bytes, t)?; }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl<'a> MessageWrite for ReadHints<'a> {
        fn get_size(&self) -> usize {
            0
                + if self.step_ms == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.step_ms) as u64) }
                + if self.func == "" { 0 } else { 1 + sizeof_len((&self.func).len()) }
                + if self.start_ms == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.start_ms) as u64) }
                + if self.end_ms == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.end_ms) as u64) }
                + self.grouping.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
                + if self.by == false { 0 } else { 1 + sizeof_varint(*(&self.by) as u64) }
                + if self.range_ms == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.range_ms) as u64) }
        }

        fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
            if self.step_ms != 0i64 { w.write_with_tag(8, |w| w.write_int64(*&self.step_ms))?; }
            if self.func != "" { w.write_with_tag(18, |w| w.write_string(&**&self.func))?; }
            if self.start_ms != 0i64 { w.write_with_tag(24, |w| w.write_int64(*&self.start_ms))?; }
            if self.end_ms != 0i64 { w.write_with_tag(32, |w| w.write_int64(*&self.end_ms))?; }
            for s in &self.grouping { w.write_with_tag(42, |w| w.write_string(&**s))?; }
            if self.by != false { w.write_with_tag(48, |w| w.write_bool(*&self.by))?; }
            if self.range_ms != 0i64 { w.write_with_tag(56, |w| w.write_int64(*&self.range_ms))?; }
            Ok(())
        }
    }

    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct Chunk<'a> {
        pub min_time_ms: i64,
        pub max_time_ms: i64,
        pub type_pb: prometheus::mod_Chunk::Encoding,
        pub data: Cow<'a, [u8]>,
    }

    impl<'a> MessageRead<'a> for Chunk<'a> {
        fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(8) => msg.min_time_ms = r.read_int64(bytes)?,
                    Ok(16) => msg.max_time_ms = r.read_int64(bytes)?,
                    Ok(24) => msg.type_pb = r.read_enum(bytes)?,
                    Ok(34) => msg.data = r.read_bytes(bytes).map(Cow::Borrowed)?,
                    Ok(t) => { r.read_unknown(bytes, t)?; }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl<'a> MessageWrite for Chunk<'a> {
        fn get_size(&self) -> usize {
            0
                + if self.min_time_ms == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.min_time_ms) as u64) }
                + if self.max_time_ms == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.max_time_ms) as u64) }
                + if self.type_pb == prometheus::mod_Chunk::Encoding::UNKNOWN { 0 } else { 1 + sizeof_varint(*(&self.type_pb) as u64) }
                + if self.data == Cow::Borrowed(b"") { 0 } else { 1 + sizeof_len((&self.data).len()) }
        }

        fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
            if self.min_time_ms != 0i64 { w.write_with_tag(8, |w| w.write_int64(*&self.min_time_ms))?; }
            if self.max_time_ms != 0i64 { w.write_with_tag(16, |w| w.write_int64(*&self.max_time_ms))?; }
            if self.type_pb != prometheus::mod_Chunk::Encoding::UNKNOWN { w.write_with_tag(24, |w| w.write_enum(*&self.type_pb as i32))?; }
            if self.data != Cow::Borrowed(b"") { w.write_with_tag(34, |w| w.write_bytes(&**&self.data))?; }
            Ok(())
        }
    }

    pub mod mod_Chunk {


        #[derive(Debug, PartialEq, Eq, Clone, Copy)]
        pub enum Encoding {
            UNKNOWN = 0,
            XOR = 1,
            HISTOGRAM = 2,
            FLOAT_HISTOGRAM = 3,
        }

        impl Default for Encoding {
            fn default() -> Self {
                Encoding::UNKNOWN
            }
        }

        impl From<i32> for Encoding {
            fn from(i: i32) -> Self {
                match i {
                    0 => Encoding::UNKNOWN,
                    1 => Encoding::XOR,
                    2 => Encoding::HISTOGRAM,
                    3 => Encoding::FLOAT_HISTOGRAM,
                    _ => Self::default(),
                }
            }
        }

        impl<'a> From<&'a str> for Encoding {
            fn from(s: &'a str) -> Self {
                match s {
                    "UNKNOWN" => Encoding::UNKNOWN,
                    "XOR" => Encoding::XOR,
                    "HISTOGRAM" => Encoding::HISTOGRAM,
                    "FLOAT_HISTOGRAM" => Encoding::FLOAT_HISTOGRAM,
                    _ => Self::default(),
                }
            }
        }

    }

    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Debug, Default, PartialEq, Clone)]
    pub struct ChunkedSeries<'a> {
        pub labels: Vec<prometheus::Label<'a>>,
        pub chunks: Vec<prometheus::Chunk<'a>>,
    }

    impl<'a> MessageRead<'a> for ChunkedSeries<'a> {
        fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
            let mut msg = Self::default();
            while !r.is_eof() {
                match r.next_tag(bytes) {
                    Ok(10) => msg.labels.push(r.read_message::<prometheus::Label>(bytes)?),
                    Ok(18) => msg.chunks.push(r.read_message::<prometheus::Chunk>(bytes)?),
                    Ok(t) => { r.read_unknown(bytes, t)?; }
                    Err(e) => return Err(e),
                }
            }
            Ok(msg)
        }
    }

    impl<'a> MessageWrite for ChunkedSeries<'a> {
        fn get_size(&self) -> usize {
            0
                + self.labels.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
                + self.chunks.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        }

        fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
            for s in &self.labels { w.write_with_tag(10, |w| w.write_message(s))?; }
            for s in &self.chunks { w.write_with_tag(18, |w| w.write_message(s))?; }
            Ok(())
        }
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct WriteRequest<'a> {
    pub timeseries: Vec<prometheus::TimeSeries<'a>>,
    pub metadata: Vec<prometheus::MetricMetadata<'a>>,
}

impl<'a> MessageRead<'a> for WriteRequest<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.timeseries.push(r.read_message::<prometheus::TimeSeries>(bytes)?),
                Ok(26) => msg.metadata.push(r.read_message::<prometheus::MetricMetadata>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for WriteRequest<'a> {
    fn get_size(&self) -> usize {
        0
            + self.timeseries.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
            + self.metadata.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.timeseries { w.write_with_tag(10, |w| w.write_message(s))?; }
        for s in &self.metadata { w.write_with_tag(26, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ReadRequest<'a> {
    pub queries: Vec<Query<'a>>,
    pub accepted_response_types: Vec<mod_ReadRequest::ResponseType>,
}

impl<'a> MessageRead<'a> for ReadRequest<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.queries.push(r.read_message::<Query>(bytes)?),
                Ok(18) => msg.accepted_response_types = r.read_packed(bytes, |r, bytes| Ok(r.read_enum(bytes)?))?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ReadRequest<'a> {
    fn get_size(&self) -> usize {
        0
            + self.queries.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
            + if self.accepted_response_types.is_empty() { 0 } else { 1 + sizeof_len(self.accepted_response_types.iter().map(|s| sizeof_varint(*(s) as u64)).sum::<usize>()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.queries { w.write_with_tag(10, |w| w.write_message(s))?; }
        w.write_packed_with_tag(18, &self.accepted_response_types, |w, m| w.write_enum(*m as i32), &|m| sizeof_varint(*(m) as u64))?;
        Ok(())
    }
}

pub mod mod_ReadRequest {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum ResponseType {
        SAMPLES = 0,
        STREAMED_XOR_CHUNKS = 1,
    }

    impl Default for ResponseType {
        fn default() -> Self {
            ResponseType::SAMPLES
        }
    }

    impl From<i32> for ResponseType {
        fn from(i: i32) -> Self {
            match i {
                0 => ResponseType::SAMPLES,
                1 => ResponseType::STREAMED_XOR_CHUNKS,
                _ => Self::default(),
            }
        }
    }

    impl<'a> From<&'a str> for ResponseType {
        fn from(s: &'a str) -> Self {
            match s {
                "SAMPLES" => ResponseType::SAMPLES,
                "STREAMED_XOR_CHUNKS" => ResponseType::STREAMED_XOR_CHUNKS,
                _ => Self::default(),
            }
        }
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ReadResponse<'a> {
    pub results: Vec<QueryResult<'a>>,
}

impl<'a> MessageRead<'a> for ReadResponse<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.results.push(r.read_message::<QueryResult>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ReadResponse<'a> {
    fn get_size(&self) -> usize {
        0
            + self.results.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.results { w.write_with_tag(10, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Query<'a> {
    pub start_timestamp_ms: i64,
    pub end_timestamp_ms: i64,
    pub matchers: Vec<prometheus::LabelMatcher<'a>>,
    pub hints: Option<prometheus::ReadHints<'a>>,
}

impl<'a> MessageRead<'a> for Query<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.start_timestamp_ms = r.read_int64(bytes)?,
                Ok(16) => msg.end_timestamp_ms = r.read_int64(bytes)?,
                Ok(26) => msg.matchers.push(r.read_message::<prometheus::LabelMatcher>(bytes)?),
                Ok(34) => msg.hints = Some(r.read_message::<prometheus::ReadHints>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Query<'a> {
    fn get_size(&self) -> usize {
        0
            + if self.start_timestamp_ms == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.start_timestamp_ms) as u64) }
            + if self.end_timestamp_ms == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.end_timestamp_ms) as u64) }
            + self.matchers.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
            + self.hints.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.start_timestamp_ms != 0i64 { w.write_with_tag(8, |w| w.write_int64(*&self.start_timestamp_ms))?; }
        if self.end_timestamp_ms != 0i64 { w.write_with_tag(16, |w| w.write_int64(*&self.end_timestamp_ms))?; }
        for s in &self.matchers { w.write_with_tag(26, |w| w.write_message(s))?; }
        if let Some(ref s) = self.hints { w.write_with_tag(34, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct QueryResult<'a> {
    pub timeseries: Vec<prometheus::TimeSeries<'a>>,
}

impl<'a> MessageRead<'a> for QueryResult<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.timeseries.push(r.read_message::<prometheus::TimeSeries>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for QueryResult<'a> {
    fn get_size(&self) -> usize {
        0
            + self.timeseries.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.timeseries { w.write_with_tag(10, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ChunkedReadResponse<'a> {
    pub chunked_series: Vec<prometheus::ChunkedSeries<'a>>,
    pub query_index: i64,
}

impl<'a> MessageRead<'a> for ChunkedReadResponse<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.chunked_series.push(r.read_message::<prometheus::ChunkedSeries>(bytes)?),
                Ok(16) => msg.query_index = r.read_int64(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ChunkedReadResponse<'a> {
    fn get_size(&self) -> usize {
        0
            + self.chunked_series.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
            + if self.query_index == 0i64 { 0 } else { 1 + sizeof_varint(*(&self.query_index) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.chunked_series { w.write_with_tag(10, |w| w.write_message(s))?; }
        if self.query_index != 0i64 { w.write_with_tag(16, |w| w.write_int64(*&self.query_index))?; }
        Ok(())
    }
}
