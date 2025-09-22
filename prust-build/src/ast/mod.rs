#![allow(dead_code)]

mod service;

use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;

pub use service::{Function, Method, Service};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Syntax {
    #[default]
    Proto2,
    Proto3,
    // todo: support edition
    Edition(u32),
}

#[derive(Debug, Default)]
pub struct FileDescriptor {
    pub syntax: Syntax,
    pub package: Option<String>,
    pub options: HashMap<String, String>,
    pub imports: Vec<String>,

    // All top-level definitions in this file
    pub messages: Vec<Message>,
    pub enums: Vec<Enum>,
    pub services: Vec<Service>,
}

#[derive(Debug, PartialEq)]
pub enum FieldType {
    // 0 is reserved for errors
    // Order is weird for historical reasons
    Double,
    Float,
    // Not ZigZag encoded. Negative numbers take 10 bytes.
    // Use SINT64 if negative values are likely.
    Int64,
    Uint64,
    // Not ZigZag encoded. Negative numbers take 10 bytes.
    // Use SINT32 if negative values are likely.
    Int32,
    Fixed64,
    Fixed32,
    Bool,
    String,
    // Tag-delimited aggregate.
    // #[deprecated]
    // Group,

    // Length-delimited aggregate
    Message(String),

    // New in version 2
    Bytes,
    Uint32,
    Sfixed32,
    Sfixed64,
    Sint32, // Uses ZigZag encoding
    Sint64, // Uses ZigZag encoding

    Map(Box<FieldType>, Box<FieldType>),
    // Enum(String),
}

impl From<&str> for FieldType {
    fn from(value: &str) -> Self {
        match value {
            "double" => FieldType::Double,
            "float" => FieldType::Float,
            "int32" => FieldType::Int32,
            "int64" => FieldType::Int64,
            "uint32" => FieldType::Uint32,
            "uint64" => FieldType::Uint64,
            "sint32" => FieldType::Sint32,
            "sint64" => FieldType::Sint64,
            "fixed32" => FieldType::Fixed32,
            "fixed64" => FieldType::Fixed64,
            "sfixed32" => FieldType::Sfixed32,
            "sfixed64" => FieldType::Sfixed64,
            "bool" => FieldType::Bool,
            "string" => FieldType::String,
            "bytes" => FieldType::Bytes,
            _ => FieldType::Message(value.into()),
        }
    }
}

impl Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::Double => f.write_str("double"),
            FieldType::Float => f.write_str("float"),
            FieldType::Int64 => f.write_str("int64"),
            FieldType::Uint64 => f.write_str("uint64"),
            FieldType::Int32 => f.write_str("int32"),
            FieldType::Fixed64 => f.write_str("fixed64"),
            FieldType::Fixed32 => f.write_str("fixed32"),
            FieldType::Bool => f.write_str("bool"),
            FieldType::String => f.write_str("string"),
            FieldType::Message(typ) => f.write_str(typ),
            FieldType::Bytes => f.write_str("bytes"),
            FieldType::Uint32 => f.write_str("uint32"),
            FieldType::Sfixed32 => f.write_str("sfixed32"),
            FieldType::Sfixed64 => f.write_str("sfixed64"),
            FieldType::Sint32 => f.write_str("sint32"),
            FieldType::Sint64 => f.write_str("sint64"),
            FieldType::Map(key, value) => f.write_fmt(format_args!("map<{key}, {value}>")),
        }
    }
}

impl FieldType {
    // 0	Varint	int32, int64, uint32, uint64, sint32, sint64, bool, enum
    // 1	64-bit	fixed64, sfixed64, double
    // 2	Length-delimited	string, bytes, embedded messages, packed repeated fields
    // 3	Start group	groups (deprecated)
    // 4	End group	groups (deprecated)
    // 5	32-bit	fixed32, sfixed32, float
    pub fn wire_type(&self) -> u32 {
        match self {
            FieldType::Int32
            | FieldType::Sint32
            | FieldType::Int64
            | FieldType::Sint64
            | FieldType::Uint32
            | FieldType::Uint64
            | FieldType::Bool => 0,
            FieldType::Fixed64 | FieldType::Sfixed64 | FieldType::Double => 1,
            FieldType::String | FieldType::Bytes | FieldType::Message(_) | FieldType::Map(_, _) => {
                2
            }
            FieldType::Fixed32 | FieldType::Sfixed32 | FieldType::Float => 5,
        }
    }

    pub fn default_value(&self) -> Option<&'static str> {
        match self {
            FieldType::Double | FieldType::Float => Some("0.0"),
            FieldType::Bool => Some("false"),
            FieldType::Uint32
            | FieldType::Sfixed32
            | FieldType::Sfixed64
            | FieldType::Sint32
            | FieldType::Sint64
            | FieldType::Int64
            | FieldType::Uint64
            | FieldType::Int32
            | FieldType::Fixed64
            | FieldType::Fixed32 => Some("0"),
            FieldType::String => Some(""),
            FieldType::Bytes => Some(""),
            FieldType::Message(_) => None,
            FieldType::Map(_, _) => None,
        }
    }

    pub fn rust_type(&self) -> Option<&'static str> {
        match self {
            FieldType::Double => Some("f64"),
            FieldType::Float => Some("f32"),
            FieldType::Int64 => Some("i64"),
            FieldType::Uint64 => Some("u64"),
            FieldType::Int32 => Some("i32"),
            FieldType::Fixed64 => Some("u64"),
            FieldType::Fixed32 => Some("u32"),
            FieldType::Bool => Some("bool"),
            FieldType::String => Some("String"),
            FieldType::Bytes => Some("Vec<u8>"),
            FieldType::Uint32 => Some("u32"),
            FieldType::Sfixed32 => Some("i32"),
            FieldType::Sfixed64 => Some("i64"),
            FieldType::Sint32 => Some("i32"),
            FieldType::Sint64 => Some("i64"),
            _ => None,
        }
    }

    pub fn fixed_size(&self) -> Option<usize> {
        match self {
            FieldType::Fixed64 | FieldType::Sfixed64 | FieldType::Double => Some(8),
            FieldType::Fixed32 | FieldType::Sfixed32 | FieldType::Float => Some(4),
            FieldType::Bool => Some(1),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Label {
    Optional,
    Repeated,
    // The required label is only allowed in google.protobuf. In proto3 and Editions
    Required,
}

impl Label {
    pub fn as_str(&self) -> &'static str {
        match self {
            Label::Optional => "optional",
            Label::Repeated => "repeated",
            Label::Required => "required",
        }
    }
}

pub enum FieldCardinality<'a> {
    // proto3 singular
    // - optional (recommended)
    // - implicit (not recommended)

    // proto2 singular
    // - optional (recommended)
    // - required (Do not use)
    Optional,
    Required,

    Repeated,
    Map(&'a FieldType, &'a FieldType),
}

#[derive(Debug)]
pub struct Field {
    pub label: Label,
    pub typ: FieldType,
    pub name: String,
    pub number: u32,

    pub options: HashMap<String, String>,
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.options.len() > 0 {
            let options = self
                .options
                .iter()
                .map(|(k, v)| format!("{k} = {v}"))
                .collect::<Vec<_>>()
                .join(", ");
            f.write_fmt(format_args!(
                "{} {} {} = {} [ {} ]",
                self.label.as_str(),
                self.typ,
                self.name,
                self.number,
                options
            ))
        } else {
            f.write_fmt(format_args!(
                "{} {} {} = {}",
                self.label.as_str(),
                self.typ,
                self.name,
                self.number,
            ))
        }
    }
}

impl Field {
    pub fn packed(&self) -> bool {
        self.options.get("packed").map(|s| s.as_str()) == Some("true")
    }

    #[inline]
    pub fn deprecated(&self) -> bool {
        self.options.get("deprecated").map(|v| v.as_str()) == Some("true")
    }

    #[inline]
    pub fn default_value(&self) -> Option<&String> {
        self.options.get("default")
    }
}

#[derive(Debug)]
pub enum Reserved {
    Single(u32),
    // (inclusive, exclusive)
    Range(u32, u32),
}

impl Reserved {
    pub fn contains(&self, value: u32) -> bool {
        match self {
            Reserved::Single(v) => *v == value,
            Reserved::Range(start, end) => value >= *start && value < *end,
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub name: String,
    pub fields: Vec<Field>,
    pub reserved: Vec<Reserved>,
    pub options: HashMap<String, String>,

    /// Message level definitions
    pub messages: Vec<Message>,
    pub enums: Vec<Enum>,
    pub oneofs: Vec<OneOf>,
}

impl Message {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty() && self.oneofs.is_empty()
    }
}

#[derive(Debug)]
pub struct Enum {
    pub name: String,

    pub variants: Vec<(String, i32)>,
}

impl Enum {
    pub fn default_value(&self) -> &str {
        &self.variants.first().unwrap().0
    }
}

#[derive(Debug)]
pub struct OneOfVarint {
    pub number: u32,
    pub name: String,
    pub typ: FieldType,
    pub options: HashMap<String, String>,
}

impl OneOfVarint {
    #[inline]
    pub fn tag(&self) -> u32 {
        self.number << 3 | self.typ.wire_type()
    }
}

#[derive(Debug)]
pub struct OneOf {
    pub name: String,

    pub variants: Vec<OneOfVarint>,
}

pub struct Extension {
    pub start: u32,
    pub end: u32,

    pub properties: BTreeMap<String, String>,
}
