use super::Buffer;
use super::context::{Container, Context};
use super::generate::generate_default_value;
use super::sanitize::{sanitize_field, sanitize_type_name, snake, upper_camel};
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
        generate_default_message(buf, msg, cx);

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

            let assignment = match cx.cardinality(field) {
                FieldCardinality::Optional => match &field.typ {
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
                },
                FieldCardinality::Required => {
                    if let FieldType::Message(typ) = &field.typ
                        && let Some((_path, c)) = cx.lookup_type(typ)
                        && c.is_enum()
                    {
                        tag = field.number << 3 | 0;
                    }

                    format!(
                        "msg.{} = {}?",
                        snake(&field.name),
                        read_field(&field.typ, cx)
                    )
                }
                FieldCardinality::Repeated => {
                    if cx.packed(field) {
                        tag = field.number << 3 | 2;

                        if field.typ.fixed_size().is_some() {
                            format!("msg.{} = buf.read_packed_fixed()?", snake(&field.name),)
                        } else {
                            format!(
                                "msg.{} = buf.read_packed(|buf| {})?",
                                snake(&field.name),
                                read_field(&field.typ, cx)
                            )
                        }
                    } else {
                        format!(
                            "msg.{}.push({}?)",
                            snake(&field.name),
                            read_field(&field.typ, cx)
                        )
                    }
                }
                FieldCardinality::Map(key, value) => {
                    format!(
                        "buf.read_map(&mut msg.{}, {}, {})?",
                        snake(&field.name),
                        cx.read_func(key),
                        cx.read_func(value)
                    )

                    // buf.push(format!("        {tag} => {{\n"));
                    // buf.push("            let _len = buf.read_varint32()?;\n");
                    // buf.push("            let _typ = buf.read_varint32()? >> 3;\n");
                    // buf.push(format!("            let k = {}?;\n", read_field(&key, cx)));
                    // buf.push("            let _typ = buf.read_varint32()? >> 3;\n");
                    // buf.push(format!(
                    //     "            let v = {}?;\n",
                    //     read_field(&value, cx)
                    // ));
                    // buf.push(format!(
                    //     "        msg.{}.insert(k, v);\n",
                    //     snake(&field.name)
                    // ));
                    // buf.push("        }\n");
                    // continue;
                }
            };

            buf.push(format!("        {tag} => {assignment},\n"));
        }

        for oneof in &msg.oneofs {
            for variant in &oneof.variants {
                let tag = variant.tag();

                // todo: handle type properly
                buf.push(format!(
                    "        {tag} => msg.{} = Some({}::{}({}?)),\n",
                    snake(&oneof.name),
                    format!("{}::{}", snake(&msg.name), upper_camel(&oneof.name)),
                    upper_camel(&variant.name),
                    read_field(&variant.typ, cx),
                ));
            }
        }

        if small_message(msg, cx) {
            buf.push("        tag => if tag < 0x80 {\n");
            buf.push("            buf.read_unknown(tag)?;\n");
            buf.push("        } else {\n");
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

fn generate_default_message(buf: &mut Buffer, msg: &Message, cx: &Context) {
    let defaults = msg
        .fields
        .iter()
        .filter(|f| f.default_value().is_some())
        .count();
    if defaults == 0 || defaults == msg.fields.len() {
        // - defaults == 0:
        //     there is no default field option, use derived Default implement
        // - defaults == msg.fields.len():
        //     all fields has default field option, use manually Default implement
        buf.push("let mut msg = Self::default();\n");
        return;
    }

    buf.push(format!("let mut msg = Self {{\n"));
    let mut default_destruct = false;
    for field in &msg.fields {
        let Some(value) = field.default_value() else {
            default_destruct = true;
            continue;
        };

        let default = generate_default_value(field, value, cx);
        let default = match &field.typ {
            FieldType::String => format!("String::from({default})"),
            FieldType::Bytes => format!("Vec::from({default})"),
            _ => default,
        };
        match cx.cardinality(field) {
            FieldCardinality::Optional => {
                buf.push(format!(
                    "    {}: Some({default}),\n",
                    sanitize_field(&field.name)
                ));
            }
            FieldCardinality::Required => {
                buf.push(format!("    {}: {default},\n", sanitize_field(&field.name)));
            }
            FieldCardinality::Repeated | FieldCardinality::Map(_, _) => {}
        }
    }

    if default_destruct {
        buf.push("    ..Default::default()\n");
    }

    buf.push("};\n");
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
