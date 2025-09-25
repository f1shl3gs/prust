use std::collections::HashMap;

use super::config::Config;
use super::sanitize::{sanitize_type_name, sanitize_variant, snake, upper_camel};
use crate::ast::{
    Enum, Field, FieldCardinality, FieldType, FileDescriptor, Label, Message, Syntax,
};
use crate::codegen::sizeof::sizeof_varint;

#[derive(Debug)]
pub enum Container<'a> {
    Message(&'a Message),
    Enum(&'a Enum),
}

impl Container<'_> {
    pub fn is_enum(&self) -> bool {
        match self {
            Container::Enum(_) => true,
            Container::Message(_) => false,
        }
    }
}

pub struct Context<'a> {
    pub fd: &'a FileDescriptor,
    pub config: &'a Config,
    pub imports: &'a HashMap<String, FileDescriptor>,

    // we don't need a tree, because Message contains sub Message too,
    // so actually this field is a tree
    pub messages: Vec<&'a Message>,
}

impl Context<'_> {
    pub fn lookup_type(&self, typ: &str) -> Option<(String, Container<'_>)> {
        let mut segments = typ.split('.');
        let first = segments.next().unwrap();

        // 1. lookup in current stack
        for msg in self.messages.iter().rev() {
            if let Some(en) = msg.enums.iter().find(|en| en.name == first) {
                if !typ.contains('.') {
                    return Some((
                        format!("{}::{}", snake(&msg.name), upper_camel(typ)),
                        Container::Enum(en),
                    ));
                }
            }

            for sub in msg.messages.iter() {
                if sub.name != first {
                    continue;
                }

                return lookup(sub, segments).map(|(path, c)| {
                    let path = format!("{}::{}", snake(&msg.name), path);
                    (path, c)
                });
            }
        }

        // 2. lookup in current file descriptor
        for msg in self.fd.messages.iter() {
            if msg.name != first {
                continue;
            }

            return lookup(msg, segments);
        }

        // enum in file descriptor
        for en in self.fd.enums.iter() {
            if en.name == first && segments.next().is_none() {
                return Some((upper_camel(first), Container::Enum(en)));
            }
        }

        // 3. lookup in imports
        for (path, fd) in self.imports {
            let pkg = match fd.package.as_ref() {
                Some(pkg) => pkg,
                None => path.strip_suffix(".proto").unwrap_or(path),
            };

            let stripped = typ.strip_prefix(&format!("{pkg}.")).unwrap_or(typ);

            let mut segments = stripped.split('.');
            let first = segments.next().unwrap();

            for msg in fd.messages.iter() {
                if msg.name != first {
                    continue;
                }

                return match lookup(msg, segments) {
                    Some((path, c)) => {
                        let prefix = pkg.split('.').map(snake).collect::<Vec<_>>().join("::");

                        Some((format!("{}::{}", prefix, path), c))
                    }
                    None => None,
                };
            }

            // lookup enum in imported file descriptor
            for en in fd.enums.iter() {
                if en.name == first && segments.next().is_none() {
                    let prefix = pkg.split('.').map(snake).collect::<Vec<_>>().join("::");
                    return Some((
                        format!("{prefix}::{}", upper_camel(first)),
                        Container::Enum(en),
                    ));
                }
            }
        }

        None
    }

    pub fn tag(&self, field: &Field) -> u32 {
        let wire_type = if self.packed(field) && field.label == Label::Repeated {
            2
        } else {
            match &field.typ {
                FieldType::Int32
                | FieldType::Sint32
                | FieldType::Int64
                | FieldType::Sint64
                | FieldType::Uint32
                | FieldType::Uint64
                | FieldType::Bool => 0,
                FieldType::Fixed64 | FieldType::Sfixed64 | FieldType::Double => 1,
                FieldType::Message(typ) => match self.lookup_type(typ) {
                    Some((_, Container::Enum(_))) => 0,
                    _ => 2,
                },
                FieldType::String | FieldType::Bytes | FieldType::Map(_, _) => 2,
                FieldType::Fixed32 | FieldType::Sfixed32 | FieldType::Float => 5,
            }
        };

        field.number << 3 | wire_type
    }

    pub fn packed(&self, field: &Field) -> bool {
        match &field.typ {
            // scalars
            FieldType::Double
            | FieldType::Float
            | FieldType::Int64
            | FieldType::Uint64
            | FieldType::Int32
            | FieldType::Fixed64
            | FieldType::Fixed32
            | FieldType::Uint32
            | FieldType::Sfixed32
            | FieldType::Sfixed64
            | FieldType::Sint32
            | FieldType::Sint64 => match field.options.get("packed") {
                // In proto3, `repeated` fields of scalar numeric types uses `packed`
                // encoding by default
                None => self.fd.syntax == Syntax::Proto3,
                Some(b) => b == "true"
            },
            FieldType::Message(typ) => match self.lookup_type(typ) {
                Some((_, Container::Enum(_))) => match field.options.get("packed") {
                    None => self.fd.syntax == Syntax::Proto3,
                    Some(b) => b == "true"
                },
                _ => false,
            },
            _ => false,
        }
    }

    pub fn path(&self) -> String {
        self.messages
            .iter()
            .map(|m| m.name.to_string())
            .collect::<Vec<_>>()
            .join(".")
    }

    pub fn message_attributes(&self) -> &[String] {
        if let Some(attrs) = self.config.message_attributes.get("") {
            return attrs.as_slice();
        }

        let path = self.path();
        self.config
            .message_attributes
            .iter()
            .find_map(|(key, attrs)| {
                if key.starts_with(&path) {
                    Some(attrs.as_slice())
                } else {
                    None
                }
            })
            .unwrap_or(&[])
    }

    pub fn enum_attributes(&self) -> &[String] {
        if let Some(attrs) = self.config.enum_attributes.get("") {
            return attrs.as_slice();
        }

        let path = self.path();
        self.config
            .enum_attributes
            .iter()
            .find_map(|(key, attrs)| {
                if key.starts_with(&path) {
                    Some(attrs.as_slice())
                } else {
                    None
                }
            })
            .unwrap_or(&[])
    }

    pub fn oneof_attributes(&self) -> &[String] {
        if let Some(attrs) = self.config.message_attributes.get("") {
            return attrs.as_slice();
        }

        let path = self.path();
        self.config
            .oneof_attributes
            .iter()
            .find_map(|(key, attrs)| {
                if key.starts_with(&path) {
                    Some(attrs.as_slice())
                } else {
                    None
                }
            })
            .unwrap_or(&[])
    }

    pub fn cardinality<'a>(&self, field: &'a Field) -> FieldCardinality<'a> {
        if let FieldType::Map(key, value) = &field.typ {
            return FieldCardinality::Map(key.as_ref(), value.as_ref());
        }

        if field.label == Label::Repeated {
            return FieldCardinality::Repeated;
        }

        match self.fd.syntax {
            Syntax::Proto2 => match field.label {
                Label::Required => FieldCardinality::Required,
                Label::Optional => {
                    if field.default_value().is_some() {
                        FieldCardinality::Required
                    } else {
                        FieldCardinality::Optional
                    }
                }
                _ => unreachable!(),
            },
            Syntax::Proto3 => {
                // implicit: (not recommended) An implicit field has no explicit cardinality label and behaves as follows:
                //     if the field is a message type, it behaves just like an optional field.
                //     if the field is not a message, it has two states:
                //         the field is set to a non-default (non-zero) value that was explicitly set or parsed from the
                //         wire. It will be serialized to the wire.
                //
                //         the field is set to the default (zero) value. It will not be serialized to the wire. In fact,
                //         you cannot determine whether the default (zero) value was set or parsed from the wire or not
                //         provided at all. For more on this subject, see Field Presence.
                if field.label == Label::Optional {
                    return FieldCardinality::Optional;
                }

                match &field.typ {
                    FieldType::Message(typ) => match self.lookup_type(typ) {
                        Some((_, Container::Enum(_))) => FieldCardinality::Required,
                        _ => FieldCardinality::Optional,
                    },
                    FieldType::Double
                    | FieldType::Float
                    | FieldType::Int64
                    | FieldType::Uint64
                    | FieldType::Int32
                    | FieldType::Fixed64
                    | FieldType::Fixed32
                    | FieldType::Bool
                    | FieldType::String
                    | FieldType::Bytes
                    | FieldType::Uint32
                    | FieldType::Sfixed32
                    | FieldType::Sfixed64
                    | FieldType::Sint32
                    | FieldType::Sint64 => FieldCardinality::Required,
                    FieldType::Map(_, _) => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
    }

    pub fn maybe_full_path(&self, typ: &str) -> Option<String> {
        let mut path = typ.to_string();

        for msg in self.messages.iter().rev() {
            if msg.messages.iter().any(|m| m.name == typ)
                || msg.enums.iter().any(|e| &e.name == typ)
            {
                return Some(format!("{}::{path}", snake(&msg.name)));
            }

            path = format!("{}::{path}", snake(&msg.name));
        }

        None
    }

    pub fn default_value(&self, field: &Field) -> Option<String> {
        let value = match (self.fd.syntax, field.default_value()) {
            (Syntax::Proto2, None) => return None,
            (Syntax::Proto2, Some(value)) => value.to_string(),
            // default value is not support in proto3
            (Syntax::Proto3, _) => match &field.typ {
                FieldType::Double | FieldType::Float => "0.0".to_string(),
                FieldType::Int64
                | FieldType::Uint64
                | FieldType::Int32
                | FieldType::Fixed64
                | FieldType::Fixed32
                | FieldType::Uint32
                | FieldType::Sfixed32
                | FieldType::Sfixed64
                | FieldType::Sint32
                | FieldType::Sint64 => "0".to_string(),
                FieldType::Bool => "false".to_string(),
                FieldType::Bytes | FieldType::String => "".to_string(),
                FieldType::Message(typ) => {
                    return match self.lookup_type(typ) {
                        Some((path, Container::Enum(en))) => {
                            let first = &en.variants.first().unwrap().0;

                            Some(format!("{}::{}", path, sanitize_variant(&en.name, first)))
                        }
                        _ => None,
                    };
                }
                FieldType::Map(_, _) => return None,
            },
            // edition 2023 do support default value
            _ => unreachable!(),
        };

        let value = match &field.typ {
            FieldType::Message(typ) => match self.lookup_type(typ) {
                Some((_, Container::Enum(en))) => {
                    format!(
                        "{}::{}",
                        upper_camel(&en.name),
                        sanitize_variant(&en.name, &value)
                    )
                }
                Some((_, Container::Message(_))) => {
                    unreachable!()
                }
                None => value,
            },
            FieldType::Double => match value.as_str() {
                "nan" => "f64::NAN".to_string(),
                "inf" => "f64::INFINITY".to_string(),
                "-inf" => "f64::NEG_INFINITY".to_string(),
                _ => format!("{value}f64"),
            },
            FieldType::Float => match value.as_str() {
                "nan" => "f32::NAN".to_string(),
                "inf" => "f32::INFINITY".to_string(),
                "-inf" => "f32::NEG_INFINITY".to_string(),
                _ => format!("{value}f32"),
            },
            _ => value,
        };

        Some(value)
    }

    pub fn read_func(&self, typ: &FieldType) -> &'static str {
        match typ {
            FieldType::Double => "Reader::read_double",
            FieldType::Float => "Reader::read_float",
            FieldType::Int64 => "Reader::read_int64",
            FieldType::Uint64 => "Reader::read_uint64",
            FieldType::Int32 => "Reader::read_int32",
            FieldType::Fixed64 => "Reader::read_fixed64",
            FieldType::Fixed32 => "Reader::read_fixed32",
            FieldType::Bool => "Reader::read_bool",
            FieldType::String => "Reader::read_string",
            FieldType::Bytes => "Reader::read_bytes",
            FieldType::Uint32 => "Reader::read_uint32",
            FieldType::Sfixed32 => "Reader::read_sfixed32",
            FieldType::Sfixed64 => "Reader::read_sfixed64",
            FieldType::Sint32 => "Reader::read_sint32",
            FieldType::Sint64 => "Reader::read_sint64",
            FieldType::Message(typ) => match self.lookup_type(typ) {
                Some((_, Container::Enum(_en))) => "Reader::read_enum",
                _ => "Reader::read_msg",
            },
            FieldType::Map(_, _) => "Reader::read_map",
        }
    }

    pub fn maybe_fixed_size(&self, typ: &FieldType) -> Option<usize> {
        match typ {
            FieldType::Bool => Some(1),
            FieldType::Fixed64 | FieldType::Sfixed64 | FieldType::Double => Some(8),
            FieldType::Fixed32 | FieldType::Sfixed32 | FieldType::Float => Some(4),
            FieldType::Message(typ) => match self.lookup_type(typ) {
                Some((_, Container::Enum(en))) => maybe_fixed_size_enum(en),
                _ => None,
            },
            _ => None,
        }
    }
}

fn lookup<'a, 'b>(
    mut msg: &'a Message,
    mut segments: impl Iterator<Item = &'b str>,
) -> Option<(String, Container<'a>)> {
    let mut stack = vec![msg.name.clone()];

    loop {
        let Some(segment) = segments.next() else {
            break;
        };

        match msg.messages.iter().find(|m| m.name == segment) {
            Some(m) => {
                msg = m;
            }
            None => {
                if segments.next().is_none() {
                    // maybe enum
                    msg.enums.iter().find(|e| e.name == segment)?;
                } else {
                    return None;
                }
            }
        }

        stack.push(segment.to_string());
    }

    if let [previous @ .., last] = stack.as_slice() {
        let path = if previous.is_empty() {
            sanitize_type_name(last)
        } else {
            format!(
                "{}::{}",
                previous
                    .iter()
                    .map(|s| snake(s))
                    .collect::<Vec<_>>()
                    .join("::"),
                sanitize_type_name(last)
            )
        };

        if let Some(en) = msg.enums.iter().find(|e| &e.name == last) {
            Some((path, Container::Enum(en)))
        } else {
            Some((path, Container::Message(msg)))
        }
    } else {
        None
    }
}

// a little optimize for enums which don't have dynamic size
pub fn maybe_fixed_size_enum(en: &Enum) -> Option<usize> {
    let mut values = en.variants.iter().map(|(_variant, value)| value);
    let size = sizeof_varint(*values.next()? as u64);

    for other in values {
        if size != sizeof_varint(*other as u64) {
            return None;
        }
    }

    Some(size)
}
