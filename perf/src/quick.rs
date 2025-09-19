// Automatically generated rust module for 'perf.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::borrow::Cow;
use std::collections::HashMap;
type KVMap<K, V> = HashMap<K, V>;
use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    Off = 0,
    On = 1,
}

impl Default for State {
    fn default() -> Self {
        State::Off
    }
}

impl From<i32> for State {
    fn from(i: i32) -> Self {
        match i {
            0 => State::Off,
            1 => State::On,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for State {
    fn from(s: &'a str) -> Self {
        match s {
            "Off" => State::Off,
            "On" => State::On,
            _ => Self::default(),
        }
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Simple<'a> {
    pub key: Option<Cow<'a, str>>,
    pub value: Option<i32>,
}

impl<'a> MessageRead<'a> for Simple<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.key = Some(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(16) => msg.value = Some(r.read_int32(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Simple<'a> {
    fn get_size(&self) -> usize {
        0
            + self.key.as_ref().map_or(0, |m| 1 + sizeof_len((m).len()))
            + self.value.as_ref().map_or(0, |m| 1 + sizeof_varint(*(m) as u64))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if let Some(ref s) = self.key { w.write_with_tag(10, |w| w.write_string(&**s))?; }
        if let Some(ref s) = self.value { w.write_with_tag(16, |w| w.write_int32(*s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Repeats<'a> {
    pub double: Vec<f64>,
    pub float: Vec<f32>,
    pub int32: Vec<i32>,
    pub int64: Vec<i64>,
    pub uint32: Vec<u32>,
    pub uint64: Vec<u64>,
    pub sint32: Vec<i32>,
    pub sint64: Vec<i64>,
    pub fixed32: Vec<u32>,
    pub fixed64: Vec<u64>,
    pub sfixed32: Vec<i32>,
    pub sfixed64: Vec<i64>,
    pub bool_pb: Vec<bool>,
    pub string: Vec<Cow<'a, str>>,
    pub bytes: Vec<Cow<'a, [u8]>>,
}

impl<'a> MessageRead<'a> for Repeats<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(9) => msg.double.push(r.read_double(bytes)?),
                Ok(21) => msg.float.push(r.read_float(bytes)?),
                Ok(24) => msg.int32.push(r.read_int32(bytes)?),
                Ok(32) => msg.int64.push(r.read_int64(bytes)?),
                Ok(40) => msg.uint32.push(r.read_uint32(bytes)?),
                Ok(48) => msg.uint64.push(r.read_uint64(bytes)?),
                Ok(56) => msg.sint32.push(r.read_sint32(bytes)?),
                Ok(64) => msg.sint64.push(r.read_sint64(bytes)?),
                Ok(77) => msg.fixed32.push(r.read_fixed32(bytes)?),
                Ok(81) => msg.fixed64.push(r.read_fixed64(bytes)?),
                Ok(93) => msg.sfixed32.push(r.read_sfixed32(bytes)?),
                Ok(97) => msg.sfixed64.push(r.read_sfixed64(bytes)?),
                Ok(104) => msg.bool_pb.push(r.read_bool(bytes)?),
                Ok(114) => msg.string.push(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(122) => msg.bytes.push(r.read_bytes(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Repeats<'a> {
    fn get_size(&self) -> usize {
        0
            + (1 + 8) * self.double.len()
            + (1 + 4) * self.float.len()
            + self.int32.iter().map(|s| 1 + sizeof_varint(*(s) as u64)).sum::<usize>()
            + self.int64.iter().map(|s| 1 + sizeof_varint(*(s) as u64)).sum::<usize>()
            + self.uint32.iter().map(|s| 1 + sizeof_varint(*(s) as u64)).sum::<usize>()
            + self.uint64.iter().map(|s| 1 + sizeof_varint(*(s) as u64)).sum::<usize>()
            + self.sint32.iter().map(|s| 1 + sizeof_sint32(*(s))).sum::<usize>()
            + self.sint64.iter().map(|s| 1 + sizeof_sint64(*(s))).sum::<usize>()
            + (1 + 4) * self.fixed32.len()
            + (1 + 8) * self.fixed64.len()
            + (1 + 4) * self.sfixed32.len()
            + (1 + 8) * self.sfixed64.len()
            + self.bool_pb.iter().map(|s| 1 + sizeof_varint(*(s) as u64)).sum::<usize>()
            + self.string.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
            + self.bytes.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.double { w.write_with_tag(9, |w| w.write_double(*s))?; }
        for s in &self.float { w.write_with_tag(21, |w| w.write_float(*s))?; }
        for s in &self.int32 { w.write_with_tag(24, |w| w.write_int32(*s))?; }
        for s in &self.int64 { w.write_with_tag(32, |w| w.write_int64(*s))?; }
        for s in &self.uint32 { w.write_with_tag(40, |w| w.write_uint32(*s))?; }
        for s in &self.uint64 { w.write_with_tag(48, |w| w.write_uint64(*s))?; }
        for s in &self.sint32 { w.write_with_tag(56, |w| w.write_sint32(*s))?; }
        for s in &self.sint64 { w.write_with_tag(64, |w| w.write_sint64(*s))?; }
        for s in &self.fixed32 { w.write_with_tag(77, |w| w.write_fixed32(*s))?; }
        for s in &self.fixed64 { w.write_with_tag(81, |w| w.write_fixed64(*s))?; }
        for s in &self.sfixed32 { w.write_with_tag(93, |w| w.write_sfixed32(*s))?; }
        for s in &self.sfixed64 { w.write_with_tag(97, |w| w.write_sfixed64(*s))?; }
        for s in &self.bool_pb { w.write_with_tag(104, |w| w.write_bool(*s))?; }
        for s in &self.string { w.write_with_tag(114, |w| w.write_string(&**s))?; }
        for s in &self.bytes { w.write_with_tag(122, |w| w.write_bytes(&**s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct PackedRepeats<'a> {
    pub double: Cow<'a, [f64]>,
    pub float: Cow<'a, [f32]>,
    pub int32: Vec<i32>,
    pub int64: Vec<i64>,
    pub uint32: Vec<u32>,
    pub uint64: Vec<u64>,
    pub sint32: Vec<i32>,
    pub sint64: Vec<i64>,
    pub fixed32: Cow<'a, [u32]>,
    pub fixed64: Cow<'a, [u64]>,
    pub sfixed32: Cow<'a, [i32]>,
    pub sfixed64: Cow<'a, [i64]>,
    pub bool_pb: Vec<bool>,
}

impl<'a> MessageRead<'a> for PackedRepeats<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.double = r.read_packed_fixed(bytes)?.into(),
                Ok(18) => msg.float = r.read_packed_fixed(bytes)?.into(),
                Ok(26) => msg.int32 = r.read_packed(bytes, |r, bytes| Ok(r.read_int32(bytes)?))?,
                Ok(34) => msg.int64 = r.read_packed(bytes, |r, bytes| Ok(r.read_int64(bytes)?))?,
                Ok(42) => msg.uint32 = r.read_packed(bytes, |r, bytes| Ok(r.read_uint32(bytes)?))?,
                Ok(50) => msg.uint64 = r.read_packed(bytes, |r, bytes| Ok(r.read_uint64(bytes)?))?,
                Ok(58) => msg.sint32 = r.read_packed(bytes, |r, bytes| Ok(r.read_sint32(bytes)?))?,
                Ok(66) => msg.sint64 = r.read_packed(bytes, |r, bytes| Ok(r.read_sint64(bytes)?))?,
                Ok(74) => msg.fixed32 = r.read_packed_fixed(bytes)?.into(),
                Ok(82) => msg.fixed64 = r.read_packed_fixed(bytes)?.into(),
                Ok(90) => msg.sfixed32 = r.read_packed_fixed(bytes)?.into(),
                Ok(98) => msg.sfixed64 = r.read_packed_fixed(bytes)?.into(),
                Ok(106) => msg.bool_pb = r.read_packed(bytes, |r, bytes| Ok(r.read_bool(bytes)?))?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for PackedRepeats<'a> {
    fn get_size(&self) -> usize {
        0
            + if self.double.is_empty() { 0 } else { 1 + sizeof_len(self.double.len() * 8) }
            + if self.float.is_empty() { 0 } else { 1 + sizeof_len(self.float.len() * 4) }
            + if self.int32.is_empty() { 0 } else { 1 + sizeof_len(self.int32.iter().map(|s| sizeof_varint(*(s) as u64)).sum::<usize>()) }
            + if self.int64.is_empty() { 0 } else { 1 + sizeof_len(self.int64.iter().map(|s| sizeof_varint(*(s) as u64)).sum::<usize>()) }
            + if self.uint32.is_empty() { 0 } else { 1 + sizeof_len(self.uint32.iter().map(|s| sizeof_varint(*(s) as u64)).sum::<usize>()) }
            + if self.uint64.is_empty() { 0 } else { 1 + sizeof_len(self.uint64.iter().map(|s| sizeof_varint(*(s) as u64)).sum::<usize>()) }
            + if self.sint32.is_empty() { 0 } else { 1 + sizeof_len(self.sint32.iter().map(|s| sizeof_sint32(*(s))).sum::<usize>()) }
            + if self.sint64.is_empty() { 0 } else { 1 + sizeof_len(self.sint64.iter().map(|s| sizeof_sint64(*(s))).sum::<usize>()) }
            + if self.fixed32.is_empty() { 0 } else { 1 + sizeof_len(self.fixed32.len() * 4) }
            + if self.fixed64.is_empty() { 0 } else { 1 + sizeof_len(self.fixed64.len() * 8) }
            + if self.sfixed32.is_empty() { 0 } else { 1 + sizeof_len(self.sfixed32.len() * 4) }
            + if self.sfixed64.is_empty() { 0 } else { 1 + sizeof_len(self.sfixed64.len() * 8) }
            + if self.bool_pb.is_empty() { 0 } else { 1 + sizeof_len(self.bool_pb.iter().map(|s| sizeof_varint(*(s) as u64)).sum::<usize>()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_packed_fixed_with_tag(10, &self.double)?;
        w.write_packed_fixed_with_tag(18, &self.float)?;
        w.write_packed_with_tag(26, &self.int32, |w, m| w.write_int32(*m), &|m| sizeof_varint(*(m) as u64))?;
        w.write_packed_with_tag(34, &self.int64, |w, m| w.write_int64(*m), &|m| sizeof_varint(*(m) as u64))?;
        w.write_packed_with_tag(42, &self.uint32, |w, m| w.write_uint32(*m), &|m| sizeof_varint(*(m) as u64))?;
        w.write_packed_with_tag(50, &self.uint64, |w, m| w.write_uint64(*m), &|m| sizeof_varint(*(m) as u64))?;
        w.write_packed_with_tag(58, &self.sint32, |w, m| w.write_sint32(*m), &|m| sizeof_sint32(*(m)))?;
        w.write_packed_with_tag(66, &self.sint64, |w, m| w.write_sint64(*m), &|m| sizeof_sint64(*(m)))?;
        w.write_packed_fixed_with_tag(74, &self.fixed32)?;
        w.write_packed_fixed_with_tag(82, &self.fixed64)?;
        w.write_packed_fixed_with_tag(90, &self.sfixed32)?;
        w.write_packed_fixed_with_tag(98, &self.sfixed64)?;
        w.write_packed_with_tag(106, &self.bool_pb, |w, m| w.write_bool(*m), &|m| sizeof_varint(*(m) as u64))?;
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Complex<'a> {
    pub double: f64,
    pub float: f32,
    pub int32: i32,
    pub int64: i64,
    pub uint32: u32,
    pub uint64: u64,
    pub sint32: i32,
    pub sint64: i64,
    pub fixed32: u32,
    pub fixed64: u64,
    pub sfixed32: i32,
    pub sfixed64: i64,
    pub bool_pb: bool,
    pub small_string: Cow<'a, str>,
    pub large_string: Cow<'a, str>,
    pub small_bytes: Cow<'a, [u8]>,
    pub large_bytes: Cow<'a, [u8]>,
    pub state: State,
    pub string_int32: KVMap<Cow<'a, str>, i32>,
    pub string_simple: KVMap<Cow<'a, str>, Simple<'a>>,
    pub string_state: KVMap<Cow<'a, str>, State>,
}

impl<'a> MessageRead<'a> for Complex<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(9) => msg.double = r.read_double(bytes)?,
                Ok(21) => msg.float = r.read_float(bytes)?,
                Ok(24) => msg.int32 = r.read_int32(bytes)?,
                Ok(32) => msg.int64 = r.read_int64(bytes)?,
                Ok(40) => msg.uint32 = r.read_uint32(bytes)?,
                Ok(48) => msg.uint64 = r.read_uint64(bytes)?,
                Ok(56) => msg.sint32 = r.read_sint32(bytes)?,
                Ok(64) => msg.sint64 = r.read_sint64(bytes)?,
                Ok(77) => msg.fixed32 = r.read_fixed32(bytes)?,
                Ok(81) => msg.fixed64 = r.read_fixed64(bytes)?,
                Ok(93) => msg.sfixed32 = r.read_sfixed32(bytes)?,
                Ok(97) => msg.sfixed64 = r.read_sfixed64(bytes)?,
                Ok(104) => msg.bool_pb = r.read_bool(bytes)?,
                Ok(114) => msg.small_string = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(122) => msg.large_string = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(130) => msg.small_bytes = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(138) => msg.large_bytes = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(144) => msg.state = r.read_enum(bytes)?,
                Ok(154) => {
                    let (key, value) = r.read_map(bytes, |r, bytes| Ok(r.read_string(bytes).map(Cow::Borrowed)?), |r, bytes| Ok(r.read_int32(bytes)?))?;
                    msg.string_int32.insert(key, value);
                }
                Ok(162) => {
                    let (key, value) = r.read_map(bytes, |r, bytes| Ok(r.read_string(bytes).map(Cow::Borrowed)?), |r, bytes| Ok(r.read_message::<Simple>(bytes)?))?;
                    msg.string_simple.insert(key, value);
                }
                Ok(170) => {
                    let (key, value) = r.read_map(bytes, |r, bytes| Ok(r.read_string(bytes).map(Cow::Borrowed)?), |r, bytes| Ok(r.read_enum(bytes)?))?;
                    msg.string_state.insert(key, value);
                }
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Complex<'a> {
    fn get_size(&self) -> usize {
        0
            + 1 + 8
            + 1 + 4
            + 1 + sizeof_varint(*(&self.int32) as u64)
            + 1 + sizeof_varint(*(&self.int64) as u64)
            + 1 + sizeof_varint(*(&self.uint32) as u64)
            + 1 + sizeof_varint(*(&self.uint64) as u64)
            + 1 + sizeof_sint32(*(&self.sint32))
            + 1 + sizeof_sint64(*(&self.sint64))
            + 1 + 4
            + 1 + 8
            + 1 + 4
            + 1 + 8
            + 1 + sizeof_varint(*(&self.bool_pb) as u64)
            + 1 + sizeof_len((&self.small_string).len())
            + 1 + sizeof_len((&self.large_string).len())
            + 2 + sizeof_len((&self.small_bytes).len())
            + 2 + sizeof_len((&self.large_bytes).len())
            + 2 + sizeof_varint(*(&self.state) as u64)
            + self.string_int32.iter().map(|(k, v)| 2 + sizeof_len(2 + sizeof_len((k).len()) + sizeof_varint(*(v) as u64))).sum::<usize>()
            + self.string_simple.iter().map(|(k, v)| 2 + sizeof_len(2 + sizeof_len((k).len()) + sizeof_len((v).get_size()))).sum::<usize>()
            + self.string_state.iter().map(|(k, v)| 2 + sizeof_len(2 + sizeof_len((k).len()) + sizeof_varint(*(v) as u64))).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(9, |w| w.write_double(*&self.double))?;
        w.write_with_tag(21, |w| w.write_float(*&self.float))?;
        w.write_with_tag(24, |w| w.write_int32(*&self.int32))?;
        w.write_with_tag(32, |w| w.write_int64(*&self.int64))?;
        w.write_with_tag(40, |w| w.write_uint32(*&self.uint32))?;
        w.write_with_tag(48, |w| w.write_uint64(*&self.uint64))?;
        w.write_with_tag(56, |w| w.write_sint32(*&self.sint32))?;
        w.write_with_tag(64, |w| w.write_sint64(*&self.sint64))?;
        w.write_with_tag(77, |w| w.write_fixed32(*&self.fixed32))?;
        w.write_with_tag(81, |w| w.write_fixed64(*&self.fixed64))?;
        w.write_with_tag(93, |w| w.write_sfixed32(*&self.sfixed32))?;
        w.write_with_tag(97, |w| w.write_sfixed64(*&self.sfixed64))?;
        w.write_with_tag(104, |w| w.write_bool(*&self.bool_pb))?;
        w.write_with_tag(114, |w| w.write_string(&**&self.small_string))?;
        w.write_with_tag(122, |w| w.write_string(&**&self.large_string))?;
        w.write_with_tag(130, |w| w.write_bytes(&**&self.small_bytes))?;
        w.write_with_tag(138, |w| w.write_bytes(&**&self.large_bytes))?;
        w.write_with_tag(144, |w| w.write_enum(*&self.state as i32))?;
        for (k, v) in self.string_int32.iter() { w.write_with_tag(154, |w| w.write_map(2 + sizeof_len((k).len()) + sizeof_varint(*(v) as u64), 10, |w| w.write_string(&**k), 16, |w| w.write_int32(*v)))?; }
        for (k, v) in self.string_simple.iter() { w.write_with_tag(162, |w| w.write_map(2 + sizeof_len((k).len()) + sizeof_len((v).get_size()), 10, |w| w.write_string(&**k), 18, |w| w.write_message(v)))?; }
        for (k, v) in self.string_state.iter() { w.write_with_tag(170, |w| w.write_map(2 + sizeof_len((k).len()) + sizeof_varint(*(v) as u64), 10, |w| w.write_string(&**k), 16, |w| w.write_enum(*v as i32)))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct SelfReference<'a> {
    pub name: Cow<'a, str>,
    pub value: i32,
    pub reference: Option<Box<SelfReference<'a>>>,
}

impl<'a> MessageRead<'a> for SelfReference<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.value = r.read_int32(bytes)?,
                Ok(26) => msg.reference = Some(Box::new(r.read_message::<SelfReference>(bytes)?)),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for SelfReference<'a> {
    fn get_size(&self) -> usize {
        0
            + 1 + sizeof_len((&self.name).len())
            + 1 + sizeof_varint(*(&self.value) as u64)
            + self.reference.as_ref().map_or(0, |m| 1 + sizeof_len((m).get_size()))
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_string(&**&self.name))?;
        w.write_with_tag(16, |w| w.write_int32(*&self.value))?;
        if let Some(ref s) = self.reference { w.write_with_tag(26, |w| w.write_message(&**s))?; }
        Ok(())
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Data<'a> {
    pub state: State,
    pub states: Vec<State>,
    pub simple: Simple<'a>,
    pub simples: Vec<Simple<'a>>,
    pub complex: Complex<'a>,
    pub complexes: Vec<Complex<'a>>,
    pub self_reference: SelfReference<'a>,
    pub self_references: Vec<SelfReference<'a>>,
    pub repeat: Repeats<'a>,
    pub repeats: Vec<Repeats<'a>>,
    pub packed_repeat: PackedRepeats<'a>,
    pub packed_repeats: Vec<PackedRepeats<'a>>,
}

impl<'a> MessageRead<'a> for Data<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.state = r.read_enum(bytes)?,
                Ok(16) => msg.states.push(r.read_enum(bytes)?),
                Ok(26) => msg.simple = r.read_message::<Simple>(bytes)?,
                Ok(34) => msg.simples.push(r.read_message::<Simple>(bytes)?),
                Ok(42) => msg.complex = r.read_message::<Complex>(bytes)?,
                Ok(50) => msg.complexes.push(r.read_message::<Complex>(bytes)?),
                Ok(58) => msg.self_reference = r.read_message::<SelfReference>(bytes)?,
                Ok(66) => msg.self_references.push(r.read_message::<SelfReference>(bytes)?),
                Ok(74) => msg.repeat = r.read_message::<Repeats>(bytes)?,
                Ok(82) => msg.repeats.push(r.read_message::<Repeats>(bytes)?),
                Ok(90) => msg.packed_repeat = r.read_message::<PackedRepeats>(bytes)?,
                Ok(98) => msg.packed_repeats.push(r.read_message::<PackedRepeats>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Data<'a> {
    fn get_size(&self) -> usize {
        0
            + 1 + sizeof_varint(*(&self.state) as u64)
            + self.states.iter().map(|s| 1 + sizeof_varint(*(s) as u64)).sum::<usize>()
            + 1 + sizeof_len((&self.simple).get_size())
            + self.simples.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
            + 1 + sizeof_len((&self.complex).get_size())
            + self.complexes.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
            + 1 + sizeof_len((&self.self_reference).get_size())
            + self.self_references.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
            + 1 + sizeof_len((&self.repeat).get_size())
            + self.repeats.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
            + 1 + sizeof_len((&self.packed_repeat).get_size())
            + self.packed_repeats.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_enum(*&self.state as i32))?;
        for s in &self.states { w.write_with_tag(16, |w| w.write_enum(*s as i32))?; }
        w.write_with_tag(26, |w| w.write_message(&self.simple))?;
        for s in &self.simples { w.write_with_tag(34, |w| w.write_message(s))?; }
        w.write_with_tag(42, |w| w.write_message(&self.complex))?;
        for s in &self.complexes { w.write_with_tag(50, |w| w.write_message(s))?; }
        w.write_with_tag(58, |w| w.write_message(&self.self_reference))?;
        for s in &self.self_references { w.write_with_tag(66, |w| w.write_message(s))?; }
        w.write_with_tag(74, |w| w.write_message(&self.repeat))?;
        for s in &self.repeats { w.write_with_tag(82, |w| w.write_message(s))?; }
        w.write_with_tag(90, |w| w.write_message(&self.packed_repeat))?;
        for s in &self.packed_repeats { w.write_with_tag(98, |w| w.write_message(s))?; }
        Ok(())
    }
}

