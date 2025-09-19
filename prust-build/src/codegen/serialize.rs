use super::Buffer;
use super::context::{Container, Context};
use super::sanitize::{sanitize_type_name, snake, upper_camel};
use super::sizeof::sizeof_varint;
use crate::ast::{Enum, FieldCardinality, FieldType, Message};

fn generate_encoded_len(buf: &mut Buffer, msg: &Message, cx: &Context) {
    if msg.fields.is_empty() && msg.oneofs.is_empty() {
        buf.push("0\n");
        return;
    }

    let mut adding = false;
    for field in &msg.fields {
        let prefix = if adding {
            "    + "
        } else {
            adding = true;
            ""
        };

        match cx.cardinality(field) {
            FieldCardinality::Optional => {
                let tag = field.number << 3 | field.typ.wire_type();
                let mut tag_size = sizeof_varint(tag as u64);

                let field_name = match &field.typ {
                    FieldType::Map(_, _) | FieldType::String | FieldType::Bytes => "v",
                    FieldType::Message(typ) => {
                        match cx.lookup_type(typ) {
                            Some((_path, Container::Enum(_))) => {
                                // since we treat enum as i32(wire_type = 0) so recalculate tag_size
                                tag_size = sizeof_varint((field.number << 3 | 0) as u64);

                                "*v"
                            }
                            _ => "v",
                        }
                    }
                    _ => "*v",
                };

                let arg = match &field.typ {
                    FieldType::Bool
                    | FieldType::Fixed32
                    | FieldType::Sfixed32
                    | FieldType::Float
                    | FieldType::Fixed64
                    | FieldType::Sfixed64
                    | FieldType::Double => "_",
                    FieldType::Message(typ) => match cx.lookup_type(typ) {
                        Some((_path, Container::Enum(en)))
                            if maybe_fixed_size_enum(en).is_some() =>
                        {
                            "_"
                        }
                        _ => "v",
                    },
                    _ => "v",
                };

                buf.push(format!(
                    "{prefix}self.{}.as_ref().map_or(0, |{arg}| {tag_size} + {})\n",
                    snake(&field.name),
                    type_size(&field.typ, field_name, cx)
                ));
            }
            FieldCardinality::Required => {
                let tag = field.number << 3 | field.typ.wire_type();
                let tag_size = sizeof_varint(tag as u64);
                let size_of = type_size(&field.typ, &format!("self.{}", snake(&field.name)), cx);

                match cx.default_value(field).as_ref() {
                    Some(default) => {
                        let comparison = match (&field.typ, default.as_str()) {
                            (FieldType::Bool, value) => {
                                if value == "true" {
                                    format!("!self.{}", snake(&field.name))
                                } else if value == "false" {
                                    format!("self.{}", snake(&field.name))
                                } else {
                                    unreachable!()
                                }
                            }
                            (FieldType::Double | FieldType::Float, value) => {
                                if value == "f32::NAN" || value == "f64::NAN" {
                                    format!("!self.{}.is_nan()", snake(&field.name))
                                } else {
                                    format!("self.{} != {}", snake(&field.name), value)
                                }
                            }
                            (FieldType::Bytes, value) => {
                                if value == "" {
                                    format!("!self.{}.is_empty()", snake(&field.name))
                                } else {
                                    format!("self.{} != b\"{}\"", snake(&field.name), value)
                                }
                            }
                            (FieldType::String, value) => {
                                if value == "" {
                                    format!("!self.{}.is_empty()", snake(&field.name))
                                } else {
                                    format!("self.{} != \"{}\"", snake(&field.name), default)
                                }
                            }
                            _ => format!("self.{} != {}", snake(&field.name), default),
                        };

                        buf.push(format!(
                            "{prefix}{}if {comparison} {{ {tag_size} + {size_of} }} else {{ 0 }}{}\n",
                            if prefix.is_empty() && msg.fields.len() + msg.oneofs.len() > 1 {
                                "("
                            } else {
                                ""
                            },
                            if prefix.is_empty() && msg.fields.len() + msg.oneofs.len() > 1 {
                                ")"
                            } else {
                                ""
                            }
                        ))
                    }
                    // no default value
                    _ => buf.push(format!("{prefix}{tag_size} + {size_of}\n")),
                }
            }
            FieldCardinality::Repeated => {
                let tag = if cx.packed(field) {
                    field.number << 3 | 2
                } else {
                    field.number << 3 | field.typ.wire_type()
                };
                let tag_size = sizeof_varint(tag as u64);

                let field_name = match &field.typ {
                    FieldType::Bool => "_",
                    FieldType::String | FieldType::Bytes => "v",
                    FieldType::Message(typ) => match cx.lookup_type(typ) {
                        Some((_path, Container::Enum(_))) => "*v",
                        _ => "v",
                    },
                    _ => "*v",
                };
                let sizeof = match &field.typ {
                    FieldType::Bool => "1".to_string(),
                    FieldType::Fixed32 | FieldType::Sfixed32 | FieldType::Float => "4".to_string(),
                    FieldType::Fixed64 | FieldType::Sfixed64 | FieldType::Double => "8".to_string(),
                    FieldType::Int32 => {
                        format!("sizeof_int32({field_name})")
                    }
                    FieldType::Int64 | FieldType::Uint32 => {
                        format!("sizeof_varint({field_name} as u64)")
                    }
                    FieldType::Uint64 => format!("sizeof_varint({field_name})"),
                    FieldType::Sint32 => format!("sizeof_sint32({field_name})"),
                    FieldType::Sint64 => format!("sizeof_sint64({field_name})"),
                    FieldType::Bytes | FieldType::String => {
                        format!("sizeof_len({field_name}.len())")
                    }
                    FieldType::Message(typ) => match cx.lookup_type(typ) {
                        Some((_path, Container::Enum(en))) => match maybe_fixed_size_enum(en) {
                            Some(size) => size.to_string(),
                            None => format!("sizeof_varint({field_name} as u64)"),
                        },
                        _ => {
                            format!("sizeof_len({field_name}.encoded_len())")
                        }
                    },
                    FieldType::Map(key, value) => {
                        let vf = match value.as_ref() {
                            FieldType::String | FieldType::Bytes => "v",
                            FieldType::Message(_) => "v",
                            FieldType::Map(_, _) => unreachable!("nested map is not allowed"),
                            _ => "*v",
                        };

                        format!(
                            "sizeof_len(2 + {} + {})",
                            type_size(key.as_ref(), "k", cx),
                            type_size(value.as_ref(), vf, cx)
                        )
                    }
                };

                if cx.packed(field) {
                    let sizeof = match field.typ.fixed_size() {
                        Some(size) => format!("self.{}.len() * {}", snake(&field.name), size),
                        None => format!(
                            "self.{}.iter().map(|v| {}).sum::<usize>()",
                            snake(&field.name),
                            sizeof
                        ),
                    };

                    buf.push(format!(
                        "{prefix}{}if self.{}.is_empty() {{ 0 }} else {{ {tag_size} + sizeof_len({}) }}{}\n",
                        if prefix.is_empty() && msg.fields.len() + msg.oneofs.len() > 1 { "(" } else { "" },
                        snake(&field.name),
                        sizeof,
                        if prefix.is_empty() && msg.fields.len() + msg.oneofs.len() > 1 { ")" } else { "" },
                    ));
                } else {
                    let sizeof = match cx.maybe_fixed_size(&field.typ) {
                        Some(size) => {
                            format!(
                                "self.{}.len() * ({tag_size} + {})",
                                snake(&field.name),
                                size
                            )
                        }
                        None => {
                            format!(
                                "self.{}.iter().map(|v| {tag_size} + {}).sum::<usize>()",
                                snake(&field.name),
                                sizeof
                            )
                        }
                    };

                    buf.push(format!("{prefix}{sizeof}\n"));
                }
            }
            FieldCardinality::Map(key, value) => {
                let key_field = match &key {
                    FieldType::String | FieldType::Bytes => "k",
                    _ => "*k",
                };

                let (val_arg, val_field) = match &value {
                    FieldType::Bool
                    | FieldType::Float
                    | FieldType::Fixed32
                    | FieldType::Fixed64
                    | FieldType::Double
                    | FieldType::Sfixed32
                    | FieldType::Sfixed64 => ("_", ""),
                    FieldType::String | FieldType::Bytes => ("v", "v"),
                    FieldType::Message(typ) => match cx.lookup_type(typ) {
                        Some((_path, Container::Enum(_en))) => ("_", ""),
                        Some((_path, Container::Message(_msg))) => ("v", "v"),
                        None => ("v", "v"),
                    },
                    _ => ("v", "*v"),
                };

                let tag = field.number << 3 | field.typ.wire_type();
                let tag_size = sizeof_varint(tag as u64);

                buf.push(format!(
                    // the size should be `1 + key_size + 1 + value_size`, we optimized it like so
                    "{prefix}self.{}.iter().map(|(k, {val_arg})| {tag_size} + sizeof_len(2 + {} + {})).sum::<usize>()\n",
                    snake(&field.name),
                    type_size(key, key_field, cx),
                    type_size(value, val_field, cx),
                ))
            }
        }
    }

    for oneof in &msg.oneofs {
        let prefix = if adding {
            "    + "
        } else {
            adding = true;
            ""
        };

        buf.push(format!("{prefix}match &self.{} {{\n", snake(&oneof.name)));

        buf.indent += 1;
        for variant in &oneof.variants {
            let tag = variant.tag();
            let tag_size = sizeof_varint(tag as u64);

            let field_name = match variant.typ {
                FieldType::Message(_)
                | FieldType::Map(_, _)
                | FieldType::String
                | FieldType::Bytes => "v",
                _ => "*v",
            };

            let sizeof = type_size(&variant.typ, field_name, cx);
            let arg = if sizeof.parse::<usize>().is_ok() {
                "_"
            } else {
                "v"
            };
            buf.push(format!(
                "    Some({}::{}::{}({arg})) => {tag_size} + {sizeof},\n",
                snake(&msg.name),
                upper_camel(&oneof.name),
                upper_camel(&variant.name)
            ))
        }

        buf.push("    None => 0,\n");
        buf.push("}\n");
        buf.indent -= 1;
    }
}

fn generate_encode(buf: &mut Buffer, msg: &Message, cx: &Context) {
    if msg.fields.is_empty() && msg.oneofs.is_empty() {
        buf.push("Ok(0)\n");
        return;
    }

    buf.push("let mut buf = Writer::new(buf);\n");

    for field in &msg.fields {
        let tag = cx.tag(field);
        match cx.cardinality(field) {
            FieldCardinality::Optional => {
                let field_name = match &field.typ {
                    FieldType::Message(typ) => {
                        if &msg.name == typ {
                            "v.as_ref()"
                        } else if let Some((_path, c)) = cx.lookup_type(typ)
                            && c.is_enum()
                        {
                            "*v"
                        } else {
                            "v"
                        }
                    }
                    FieldType::String | FieldType::Bytes => "v",
                    _ => "*v",
                };

                buf.push(format!(
                    "if let Some(v) = &self.{} {{ {}? }}\n",
                    snake(&field.name),
                    encode_type(&field.typ, field_name, tag, cx)
                ));
            }
            FieldCardinality::Required => {
                let field_name = match &field.typ {
                    FieldType::Message(typ) => match cx.lookup_type(typ) {
                        Some((_path, Container::Enum(_en))) => {
                            format!("self.{}", snake(&field.name))
                        }
                        _ => format!("&self.{}", snake(&field.name)),
                    },
                    FieldType::String => format!("self.{}", snake(&field.name)),
                    _ => format!("self.{}", snake(&field.name)),
                };

                let write = encode_type(&field.typ, &field_name, tag, cx);
                match cx.default_value(field) {
                    Some(default) => {
                        let comparison = match (&field.typ, &default) {
                            (FieldType::Bool, value) => {
                                if value == "true" {
                                    format!("!self.{}", snake(&field.name))
                                } else if value == "false" {
                                    format!("self.{}", snake(&field.name))
                                } else {
                                    unreachable!()
                                }
                            }
                            (FieldType::Double | FieldType::Float, value) => {
                                if value == "f32::NAN" || value == "f64::NAN" {
                                    format!("!self.{}.is_nan()", snake(&field.name))
                                } else {
                                    format!("self.{} != {}", snake(&field.name), value)
                                }
                            }
                            (FieldType::Bytes, value) => {
                                if value == "" {
                                    format!("!self.{}.is_empty()", snake(&field.name))
                                } else {
                                    format!("self.{} != b\"{}\"", snake(&field.name), value)
                                }
                            }
                            (FieldType::String, value) => {
                                if value == "" {
                                    format!("!self.{}.is_empty()", snake(&field.name))
                                } else {
                                    format!("self.{} != \"{}\"", snake(&field.name), value)
                                }
                            }
                            _ => format!("self.{} != {}", snake(&field.name), default),
                        };

                        buf.push(format!("if {comparison} {{ {write}?; }}\n"));
                    }
                    None => buf.push(format!("{write}?;\n")),
                }
            }
            FieldCardinality::Repeated => {
                let field_name = match &field.typ {
                    FieldType::Bytes | FieldType::String => "v",
                    FieldType::Message(typ) => match cx.lookup_type(typ) {
                        Some((_, Container::Enum(_))) => "*v",
                        _ => "v",
                    },
                    _ => "*v",
                };

                if cx.packed(field) {
                    if field.typ.fixed_size().is_some() {
                        buf.push(format!(
                            "buf.write_packed_fixed({tag}, &self.{})?;\n",
                            snake(&field.name),
                        ));

                        continue;
                    }

                    let write = match &field.typ {
                        FieldType::Double => format!("buf.write_double({field_name})"),
                        FieldType::Float => format!("buf.write_float({field_name})"),
                        FieldType::Int64 => format!("buf.write_int64({field_name})"),
                        FieldType::Uint64 => format!("buf.write_uint64({field_name})"),
                        FieldType::Int32 => format!("buf.write_int32({field_name})"),
                        FieldType::Fixed64 => format!("buf.write_fixed64({field_name})"),
                        FieldType::Fixed32 => format!("buf.write_fixed32({field_name})"),
                        FieldType::Bool => format!("buf.write_bool({field_name})"),
                        FieldType::String => format!("buf.write_string({field_name})"),
                        FieldType::Message(typ) => match cx.lookup_type(typ) {
                            Some((_path, Container::Message(_msg))) => {
                                if typ == &msg.name {
                                    format!("buf.write_msg({field_name}.as_ref())")
                                } else {
                                    format!("buf.write_msg({field_name})")
                                }
                            }
                            Some((_, Container::Enum(_))) => {
                                format!("buf.write_int32({field_name} as i32)")
                            }
                            None => unreachable!(),
                        },
                        FieldType::Bytes => format!("buf.write_bytes({field_name})"),
                        FieldType::Uint32 => format!("buf.write_uint32({field_name})"),
                        FieldType::Sfixed32 => format!("buf.write_sfixed32({field_name})"),
                        FieldType::Sfixed64 => format!("buf.write_sfixed64({field_name})"),
                        FieldType::Sint32 => format!("buf.write_sint32({field_name})"),
                        FieldType::Sint64 => format!("buf.write_sint64({field_name})"),
                        FieldType::Map(_, _) => unreachable!(),
                    };

                    buf.push(format!(
                        "buf.write_packed({tag}, &self.{}, |v| {}, |buf, v| {})?;\n",
                        snake(&field.name),
                        type_size(&field.typ, field_name, cx),
                        write
                    ));
                } else {
                    buf.push(format!(
                        "for v in &self.{} {{ {}? }}\n",
                        snake(&field.name),
                        encode_type(&field.typ, &field_name, tag, cx)
                    ));
                }
            }
            FieldCardinality::Map(key, value) => {
                buf.push(format!("for (k, v) in &self.{} {{\n", snake(&field.name)));
                // write tag
                buf.push(format!("    buf.write_varint({tag})?;\n"));
                // length delimited
                let key_field_name = match key {
                    FieldType::Bytes | FieldType::String => "k",
                    _ => "*k",
                };
                let ks = type_size(key, key_field_name, cx);
                let value_field_name = match value {
                    FieldType::Bytes | FieldType::String => "v",
                    FieldType::Message(typ) => match cx.lookup_type(typ) {
                        Some((_path, Container::Enum(_en))) => "*v",
                        _ => "v",
                    },
                    _ => "*v",
                };
                let vs = type_size(value, value_field_name, cx);
                buf.push(format!("    buf.write_length(2 + {ks} + {vs})?;\n"));
                buf.push(format!(
                    "    {}?;\n",
                    encode_type(key, key_field_name, 1 << 3 | key.wire_type(), cx)
                ));
                let value_wire_type = match value {
                    FieldType::Int32
                    | FieldType::Sint32
                    | FieldType::Int64
                    | FieldType::Sint64
                    | FieldType::Uint32
                    | FieldType::Uint64
                    | FieldType::Bool => 0,
                    FieldType::Message(typ) => match cx.lookup_type(typ) {
                        Some((_path, Container::Enum(_en))) => 0,
                        _ => 2,
                    },
                    FieldType::Fixed64 | FieldType::Sfixed64 | FieldType::Double => 1,
                    FieldType::String | FieldType::Bytes | FieldType::Map(_, _) => 2,
                    FieldType::Fixed32 | FieldType::Sfixed32 | FieldType::Float => 5,
                };
                buf.push(format!(
                    "    {}?;\n",
                    encode_type(value, value_field_name, 2 << 3 | value_wire_type, cx)
                ));
                buf.push("}\n");
            }
        }
    }

    for oneof in &msg.oneofs {
        buf.push(format!("match &self.{} {{\n", snake(&oneof.name)));

        for variant in &oneof.variants {
            let tag = variant.tag();
            let field_name = match &variant.typ {
                FieldType::String | FieldType::Bytes => "v",
                FieldType::Message(typ) => match cx.lookup_type(typ) {
                    Some((_path, Container::Enum(_))) => "*v",
                    _ => "v",
                },
                _ => "*v",
            };

            buf.push(format!(
                "    Some({}::{}::{}(v)) => {}?,\n",
                snake(&msg.name),
                upper_camel(&oneof.name),
                upper_camel(&variant.name),
                encode_type(&variant.typ, field_name, tag, cx)
            ));
        }

        buf.push("    None => {}\n");
        buf.push("}\n");
    }

    buf.push("Ok(buf.pos)\n");
}

pub fn generate_serialize(buf: &mut Buffer, msg: &Message, cx: &Context) {
    buf.push(format!(
        "impl Serialize for {} {{\n",
        sanitize_type_name(&msg.name)
    ));
    buf.indent += 1;

    // empty msg
    if msg.fields.is_empty() && msg.oneofs.is_empty() {
        buf.push("fn encoded_len(&self) -> usize { 0 }\n");
        buf.push("fn encode(&self, _: &mut [u8]) -> Result<usize, EncodeError> { Ok(0) }\n");

        buf.indent -= 1;
        buf.push("}\n");
        return;
    }

    {
        buf.push("fn encoded_len(&self) -> usize {\n");
        buf.indent += 1;

        generate_encoded_len(buf, &msg, cx);

        buf.indent -= 1;
        buf.push("}\n");
    }

    {
        buf.push("fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {\n");
        buf.indent += 1;

        generate_encode(buf, &msg, cx);

        buf.indent -= 1;
        buf.push("}\n");
    }

    // end of impl
    buf.indent -= 1;
    buf.push("}\n");
}

fn type_size(typ: &FieldType, field_name: &str, cx: &Context) -> String {
    match typ {
        // no mater true or false the sizeof_varint is always 1
        FieldType::Bool => "1".to_string(),
        FieldType::Fixed32 | FieldType::Sfixed32 | FieldType::Float => "4".to_string(),
        FieldType::Fixed64 | FieldType::Sfixed64 | FieldType::Double => "8".to_string(),
        FieldType::Int32 => {
            format!("sizeof_int32({field_name})")
        }
        FieldType::Int64 | FieldType::Uint32 => {
            format!("sizeof_varint({field_name} as u64)")
        }
        FieldType::Uint64 => format!("sizeof_varint({field_name})"),
        FieldType::Sint32 => {
            format!("sizeof_sint32({field_name})")
        }
        FieldType::Sint64 => {
            format!("sizeof_sint64({field_name})")
        }
        FieldType::Bytes | FieldType::String => {
            format!("sizeof_len({field_name}.len())")
        }
        FieldType::Message(typ) => match cx.lookup_type(typ) {
            Some((_path, Container::Enum(en))) => match maybe_fixed_size_enum(en) {
                Some(size) => size.to_string(),
                None => {
                    format!("sizeof_varint({field_name} as u64)")
                }
            },
            _ => {
                format!("sizeof_len({field_name}.encoded_len())")
            }
        },
        _ => unreachable!(),
    }
}

// a little optimize for enums which don't have dynamic size
fn maybe_fixed_size_enum(en: &Enum) -> Option<usize> {
    let mut values = en.variants.iter().map(|(_variant, value)| value);
    let size = sizeof_varint(*values.next()? as u64);

    for other in values {
        if size != sizeof_varint(*other as u64) {
            return None;
        }
    }

    Some(size)
}

fn encode_type(typ: &FieldType, field_name: &str, tag: u32, cx: &Context) -> String {
    let method = match typ {
        FieldType::Double => "write_double",
        FieldType::Float => "write_float",
        FieldType::Int64 => "write_int64",
        FieldType::Uint64 => "write_uint64",
        FieldType::Int32 => "write_int32",
        FieldType::Fixed64 => "write_fixed64",
        FieldType::Fixed32 => "write_fixed32",
        FieldType::Bool => "write_bool",
        FieldType::Message(typ) => match cx.lookup_type(typ) {
            Some((_path, Container::Enum(_))) => {
                return format!("buf.write({tag}, {field_name} as i32, Writer::write_int32)");
            }
            _ => "write_msg",
        },
        FieldType::Bytes => {
            return format!("buf.write({tag}, {field_name}.as_slice(), Writer::write_bytes)");
        }
        FieldType::String => {
            return format!("buf.write({tag}, {field_name}.as_str(), Writer::write_string)");
        }
        FieldType::Uint32 => "write_uint32",
        FieldType::Sfixed32 => "write_sfixed32",
        FieldType::Sfixed64 => "write_sfixed64",
        FieldType::Sint32 => "write_sint32",
        FieldType::Sint64 => "write_sint64",
        FieldType::Map(_, _) => unreachable!("map should be handled outside"),
    };

    format!("buf.write({tag}, {field_name}, Writer::{method})")
}
