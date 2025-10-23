use super::Buffer;
use super::context::{Container, Context};
use super::sanitize::{sanitize_type_name, snake, upper_camel};
use crate::ast::{FieldCardinality, FieldType, Message};

pub fn generate_deserialize(buf: &mut Buffer, msg: &Message, cx: &Context) {
    buf.push(format!(
        "impl Deserialize for {} {{\n",
        sanitize_type_name(&msg.name)
    ));
    buf.indent += 1;

    if msg.fields.is_empty() && msg.oneofs.is_empty() {
        buf.push("fn decode(_: &[u8]) -> Result<Self, DecodeError> { Ok(Self) }\n");
        buf.indent -= 1;
        buf.push("}\n");
        return;
    }

    buf.push("fn decode(src: &[u8]) -> Result<Self, DecodeError> {\n");
    buf.indent += 1;

    if msg.fields.is_empty() && msg.oneofs.is_empty() {
        buf.push("Ok(Self::default())\n");
    } else {
        buf.push("let mut buf = Reader::new(src);\n");
        // generate_default_message(buf, msg, cx);
        buf.push("let mut msg: Self = Default::default();\n");

        buf.push("while buf.pos < buf.src.len() {\n");

        // this match is a great design, because we don't need to check field number
        // and wire_type
        if small_message(msg, cx) {
            buf.push("    let tag = buf.src[buf.pos] as u32; buf.pos += 1;\n");
            buf.push("    match tag {\n");
        } else {
            buf.push("    match buf.read_uint32()? {\n");
        }
        for field in &msg.fields {
            let mut tag = cx.tag(field);

            match cx.cardinality(field) {
                FieldCardinality::Optional => {
                    let assignment = match &field.typ {
                        FieldType::Message(typ) => match cx.lookup_type(typ) {
                            Some((_path, Container::Message(_))) => {
                                if &msg.name == typ {
                                    format!(
                                        "msg.{} = Some(Box::new({}?))",
                                        snake(&field.name),
                                        read_field(&field.typ, cx)
                                    )
                                } else {
                                    format!(
                                        "msg.{} = Some({}?)",
                                        snake(&field.name),
                                        read_field(&field.typ, cx)
                                    )
                                }
                            }
                            Some((_path, Container::Enum(_))) => {
                                tag = field.number << 3 | 0;
                                format!("msg.{} = Some(buf.read_enum()?)", snake(&field.name),)
                            }
                            None => {
                                format!(
                                    "msg.{} = Some({}?)",
                                    snake(&field.name),
                                    read_field(&field.typ, cx)
                                )
                            }
                        },
                        _ => {
                            format!(
                                "msg.{} = Some({}?)",
                                snake(&field.name),
                                read_field(&field.typ, cx)
                            )
                        }
                    };

                    buf.push(format!("        {tag} => {assignment},\n"));
                }
                FieldCardinality::Required => {
                    if let FieldType::Message(typ) = &field.typ
                        && let Some((_path, c)) = cx.lookup_type(typ)
                        && c.is_enum()
                    {
                        tag = field.number << 3 | 0;
                    }

                    buf.push(format!(
                        "        {tag} => msg.{} = {}?,\n",
                        snake(&field.name),
                        read_field(&field.typ, cx)
                    ));
                }
                FieldCardinality::Repeated => {
                    let assignment = if cx.packed(field) {
                        tag = field.number << 3 | 2;

                        if cx.maybe_fixed_size(&field.typ).is_some() {
                            format!("msg.{} = buf.read_packed_fixed()?", snake(&field.name),)
                        } else {
                            format!(
                                "msg.{} = buf.read_packed({})?",
                                snake(&field.name),
                                read_func(&field.typ, cx)
                            )
                        }
                    } else {
                        format!(
                            "msg.{}.push({}?)",
                            snake(&field.name),
                            read_field(&field.typ, cx)
                        )
                    };

                    buf.push(format!("        {tag} => {assignment},\n"));
                }
                FieldCardinality::Map(key, value) => {
                    buf.indent += 2;
                    buf.push(format!("{tag} => {{\n"));
                    buf.indent += 1;

                    buf.push(format!(
                        "let (k, v) = buf.read_key_value({}, {})?;\n",
                        read_func(key, cx),
                        read_func(value, cx)
                    ));
                    buf.push(format!("msg.{}.insert(k, v);\n", snake(&field.name)));

                    buf.indent -= 1;
                    buf.push("}\n");
                    buf.indent -= 2;
                }
            }
        }

        for oneof in &msg.oneofs {
            for variant in &oneof.variants {
                let wire_type = match &variant.typ {
                    FieldType::Int32
                    | FieldType::Sint32
                    | FieldType::Int64
                    | FieldType::Sint64
                    | FieldType::Uint32
                    | FieldType::Uint64
                    | FieldType::Bool => 0,
                    FieldType::Fixed64 | FieldType::Sfixed64 | FieldType::Double => 1,
                    FieldType::Message(typ) => match cx.lookup_type(typ) {
                        Some((_, Container::Enum(_))) => 0,
                        _ => 2,
                    },
                    FieldType::String | FieldType::Bytes | FieldType::Map(_, _) => 2,
                    FieldType::Fixed32 | FieldType::Sfixed32 | FieldType::Float => 5,
                };

                // todo: handle type properly
                buf.push(format!(
                    "        {} => msg.{} = Some({}::{}({}?)),\n",
                    variant.number << 3 | wire_type,
                    snake(&oneof.name),
                    format!("{}::{}", snake(&msg.name), upper_camel(&oneof.name)),
                    upper_camel(&variant.name),
                    read_field(&variant.typ, cx),
                ));
            }
        }

        if small_message(msg, cx) {
            buf.push("        _ => {\n");
            buf.push("            buf.pos -= 1;\n");
            buf.push("            let tag = buf.read_uint32()?;\n");
            buf.push("            buf.read_unknown(tag)?;\n");
            buf.push("        }\n");
        } else {
            buf.push("        tag => buf.read_unknown(tag)?,\n");
        }

        buf.push("    }\n");
        buf.push("}\n");
        buf.push("Ok(msg)\n");
    }

    buf.indent -= 1;
    buf.push("}\n");
    buf.indent -= 1;
    buf.push("}\n");
}

fn read_func(typ: &FieldType, cx: &Context) -> &'static str {
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
        FieldType::Message(typ) => match cx.lookup_type(typ) {
            Some((_path, Container::Message(_))) => "Reader::read_msg",
            Some((_path, Container::Enum(_))) => "|buf| { buf.read_int32()?.try_into() }",
            None => unreachable!("{typ} is not found"),
        },
        FieldType::Bytes => "Reader::read_bytes",
        FieldType::Uint32 => "Reader::read_uint32",
        FieldType::Sfixed32 => "Reader::read_sfixed32",
        FieldType::Sfixed64 => "Reader::read_sfixed64",
        FieldType::Sint32 => "Reader::read_sint32",
        FieldType::Sint64 => "Reader::read_sint64",
        FieldType::Map(_, _) => unreachable!(),
    }
}

fn read_field(typ: &FieldType, cx: &Context) -> &'static str {
    match typ {
        FieldType::Double => "buf.read_double()",
        FieldType::Float => "buf.read_float()",
        FieldType::Int64 => "buf.read_int64()",
        FieldType::Uint64 => "buf.read_uint64()",
        FieldType::Int32 => "buf.read_int32()",
        FieldType::Fixed64 => "buf.read_fixed64()",
        FieldType::Fixed32 => "buf.read_fixed32()",
        FieldType::Bool => "buf.read_bool()",
        FieldType::String => "buf.read_string()",
        FieldType::Bytes => "buf.read_bytes()",
        FieldType::Uint32 => "buf.read_uint32()",
        FieldType::Sfixed32 => "buf.read_sfixed32()",
        FieldType::Sfixed64 => "buf.read_sfixed64()",
        FieldType::Sint32 => "buf.read_sint32()",
        FieldType::Sint64 => "buf.read_sint64()",
        FieldType::Message(typ) => match cx.lookup_type(typ) {
            Some((_path, Container::Enum(_))) => "buf.read_enum()",
            _ => "buf.read_msg()",
        },
        FieldType::Map(_, _) => unreachable!("nested maps are not supported by protobuf"),
    }
}

// Is the msg is small, and all tag is small enough to use only 1 byte
//
// This optimization can reduce some function calls, and bound checks,
// in the perf test, this can increase around 10% throughput.
fn small_message(msg: &Message, cx: &Context) -> bool {
    for field in &msg.fields {
        if cx.tag(field) > 0x7F {
            return false;
        }
    }

    for oneof in &msg.oneofs {
        for variant in &oneof.variants {
            if variant.tag() > 0x7F {
                return false;
            }
        }
    }

    true
}
