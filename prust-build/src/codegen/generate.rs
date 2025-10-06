use super::context::{Container, Context};
use super::deserialize::generate_deserialize;
use super::sanitize::{
    sanitize_field, sanitize_filepath, sanitize_type, sanitize_type_name, sanitize_variant,
};
use super::sanitize::{snake, upper_camel};
use super::serialize::generate_serialize;
use crate::Error;
use crate::ast::{Enum, Field, FieldCardinality, FieldType, Message, OneOf};
use crate::codegen::Buffer;
use crate::codegen::config::MapType;
use crate::codegen::service::generate_service;

pub fn generate_proto<'a>(buf: &mut Buffer, cx: &mut Context<'a>) -> Result<(), Error> {
    for path in &cx.fd.imports {
        let Some(import) = cx.imports.get(path) else {
            return Err(Error::ImportNotFound(path.to_string()));
        };

        let path = match &import.package {
            Some(pkg) => pkg.to_string(),
            None => match path.strip_suffix(".proto") {
                Some(pkg) => sanitize_filepath(pkg),
                None => sanitize_filepath(path),
            },
        };

        let base_indent = buf.indent;
        for name in path.split('.') {
            buf.push(format!("pub mod {} {{\n", snake(name)));
            buf.push("    use super::*;\n");
            buf.indent += 1;
        }

        let mut cx = Context {
            fd: import,
            config: cx.config,
            imports: cx.imports,
            messages: vec![],
        };

        generate_proto(buf, &mut cx)?;

        while buf.indent != base_indent {
            buf.indent -= 1;
            buf.push("}\n");
        }
    }

    for msg in &cx.fd.messages {
        cx.messages.push(msg);
        generate_struct(buf, msg, cx);
        cx.messages.pop();
    }

    for en in &cx.fd.enums {
        generate_enum(buf, en, cx);
    }

    for svc in &cx.fd.services {
        generate_service(buf, svc, cx);
    }

    Ok(())
}

fn generate_struct<'a>(buf: &mut Buffer, msg: &'a Message, cx: &mut Context<'a>) {
    generate_simple_struct(buf, msg, cx);

    let path = cx.path();
    if !cx.config.skip_deserialize.contains(&path) {
        generate_deserialize(buf, msg, cx);
    }
    if !cx.config.skip_serialize.contains(&path) {
        generate_serialize(buf, msg, cx);
    }

    if msg.messages.is_empty() && msg.enums.is_empty() && msg.oneofs.is_empty() {
        return;
    }

    buf.push(format!("pub mod {} {{\n", snake(&msg.name)));
    buf.push("    use super::*;\n");

    {
        buf.indent += 1;

        for msg in &msg.messages {
            cx.messages.push(msg);
            generate_struct(buf, msg, cx);
            cx.messages.pop();
        }
        for en in &msg.enums {
            generate_enum(buf, en, cx);
        }
        for oneof in &msg.oneofs {
            generate_oneof(buf, oneof, cx);
        }

        buf.indent -= 1;
    }

    buf.push("}\n");
}

fn generate_simple_struct(buf: &mut Buffer, msg: &Message, cx: &Context) {
    for attr in cx.message_attributes() {
        buf.push(format!("{attr}\n"))
    }

    if msg.fields.iter().any(|f| f.default_value().is_some()) {
        buf.push("#[derive(Debug)]\n");
    } else {
        buf.push("#[derive(Debug, Default)]\n");
    }

    if msg.is_empty() {
        buf.push(format!("pub struct {};\n", sanitize_type_name(&msg.name)));
        return;
    }

    buf.push(format!("pub struct {} {{\n", sanitize_type_name(&msg.name)));

    for field in &msg.fields {
        if field.deprecated() {
            continue;
        }

        #[cfg(feature = "debug")]
        buf.push(format!(
            "    // {} {} {} = {}\n",
            field.label.as_str(),
            field.typ,
            field.name,
            field.number
        ));

        let typ = generate_field_type(&field.typ, cx);
        let typ = match cx.cardinality(field) {
            FieldCardinality::Optional => {
                if typ == msg.name {
                    format!("Option<Box<{typ}>>")
                } else {
                    format!("Option<{typ}>")
                }
            }
            FieldCardinality::Required => {
                if typ == msg.name {
                    format!("Box<{typ}>")
                } else {
                    typ
                }
            }
            FieldCardinality::Repeated => {
                format!("Vec<{typ}>")
            }
            FieldCardinality::Map(key, value) => {
                let path = match &cx.fd.package {
                    Some(pkg) => format!("{}.{}.{}", pkg, cx.path(), field.name),
                    None => format!("{}.{}", cx.path(), field.name),
                };
                let map_type = match cx.config.tree_map.get(&path) {
                    Some(MapType::BTreeMap) => "BTreeMap",
                    _ => "HashMap",
                };

                format!(
                    "std::collections::{}<{}, {}>",
                    map_type,
                    generate_field_type(key, cx),
                    generate_field_type(value, cx)
                )
            }
        };

        let path = format!("{}.{}", cx.path(), field.name);
        if let Some(attrs) = cx.config.field_attributes.get(&path) {
            for attr in attrs {
                buf.push(format!("    {attr}\n"));
            }
        }

        buf.push(format!(
            "    pub {}: {},\n",
            sanitize_field(&field.name),
            typ
        ));
    }

    for oneof in &msg.oneofs {
        buf.push(format!(
            "    pub {}: Option<{}::{}>,\n",
            sanitize_field(&oneof.name),
            snake(&msg.name),
            upper_camel(&oneof.name)
        ));
    }

    buf.push("}\n");

    if msg.fields.iter().any(|f| f.default_value().is_some()) {
        generate_struct_default(buf, msg, cx);
    }
}

fn generate_struct_default(buf: &mut Buffer, msg: &Message, cx: &Context) {
    buf.push(format!(
        "impl Default for {} {{\n",
        sanitize_type_name(&msg.name)
    ));
    buf.indent += 1;
    buf.push("fn default() -> Self {\n");
    buf.push("    Self {\n");

    for field in &msg.fields {
        let field_name = snake(&field.name);

        match cx.cardinality(field) {
            FieldCardinality::Optional => {
                let Some(default) = field.default_value() else {
                    buf.push(format!("        {field_name}: None,\n"));
                    continue;
                };

                let default = generate_default_value(field, default, cx);
                let default = match &field.typ {
                    FieldType::Bytes => format!("Vec::from({default})"),
                    FieldType::String => format!("String::from({default})"),
                    _ => default,
                };
                buf.push(format!(
                    "        {}: Some({default}),\n",
                    snake(&field.name),
                ));
            }
            FieldCardinality::Required => {
                if let Some(default) = cx.default_value(field) {
                    let default = match &field.typ {
                        FieldType::Bytes => format!("Vec::from(\"{default}\")"),
                        FieldType::String => format!("String::from(\"{default}\")"),
                        _ => default,
                    };

                    buf.push(format!("        {field_name}: {default},\n"));

                    continue;
                }

                if let FieldType::Message(typ) = &field.typ
                    && let Some((path, Container::Enum(en))) = cx.lookup_type(typ)
                {
                    buf.push(format!(
                        "        {field_name}: {path}::{},\n",
                        sanitize_variant(&en.name, en.default_value())
                    ));
                } else {
                    buf.push(format!("        {field_name}: Default::default(),\n"));
                }
            }
            FieldCardinality::Repeated | FieldCardinality::Map(_, _) => {
                buf.push(format!("        {field_name}: Default::default(),\n"))
            }
        }
    }

    for oneof in &msg.oneofs {
        buf.push(format!("        {}: None,\n", snake(&oneof.name)));
    }

    buf.push("    }\n");
    buf.push("}\n");
    buf.indent -= 1;
    buf.push("}\n");
}

pub fn generate_default_value(field: &Field, value: &str, cx: &Context) -> String {
    match &field.typ {
        FieldType::String => {
            format!("\"{}\"", value)
        }
        FieldType::Bytes => {
            format!("b\"{}\"", value)
        }
        FieldType::Float => match value {
            "inf" => "f32::INFINITY".to_string(),
            "nan" => "f32::NAN".to_string(),
            "-inf" => "f32::NEG_INFINITY".to_string(),
            _ => format!("{value}f32"),
        },
        FieldType::Double => match value {
            "inf" => "f64::INFINITY".to_string(),
            "nan" => "f64::NAN".to_string(),
            "-inf" => "f64::NEG_INFINITY".to_string(),
            _ => format!("{value}f64"),
        },
        FieldType::Int32 | FieldType::Sint32 | FieldType::Sfixed32 => format!("{value}i32"),
        FieldType::Int64 | FieldType::Sint64 | FieldType::Sfixed64 => format!("{value}i64"),
        FieldType::Uint32 | FieldType::Fixed32 => format!("{value}u32"),
        FieldType::Uint64 | FieldType::Fixed64 => format!("{value}u64"),
        FieldType::Bool => value.to_string(),
        FieldType::Message(typ) => match cx.lookup_type(typ) {
            Some((path, Container::Enum(en))) => {
                if en.variants.iter().any(|(variant, _value)| variant == value) {
                    format!("{}::{}", path, sanitize_variant(&en.name, value))
                } else {
                    format!(
                        "{}::{}",
                        sanitize_type(typ),
                        sanitize_variant(&en.name, value)
                    )
                }
            }
            _ => {
                panic!("unknown default value {value}")
            }
        },
        FieldType::Map(_, _) => {
            panic!("default value for map is not supported")
        }
    }
}

fn generate_enum(buf: &mut Buffer, en: &Enum, cx: &Context) {
    for attr in cx.enum_attributes() {
        buf.push(format!("{attr}\n"))
    }

    buf.push("#[derive(Clone, Copy, Debug, Default, PartialEq)]\n");
    // buf.push("#[repr(i32)]\n");
    buf.push(format!("pub enum {} {{\n", upper_camel(&en.name)));

    buf.indent += 1;
    let mut first = true;
    for (variant, value) in &en.variants {
        if first {
            first = false;
            buf.push("#[default]\n");
        }

        buf.push(format!(
            "{} = {},\n",
            sanitize_variant(&en.name, variant),
            value
        ))
    }
    buf.indent -= 1;

    buf.push("}\n");

    // try from
    {
        buf.push(format!(
            "impl TryFrom<i32> for {} {{\n",
            upper_camel(&en.name)
        ));
        buf.push("    type Error = DecodeError;\n");
        buf.push("    fn try_from(value: i32) -> Result<Self, DecodeError> {\n");
        buf.push("        match value {\n");

        for (variant, value) in &en.variants {
            buf.push(format!(
                "            {} => Ok({}::{}),\n",
                value,
                upper_camel(&en.name),
                sanitize_variant(&en.name, variant)
            ));
        }
        buf.push(format!(
            "            _ => Err(DecodeError::UnknownVariant(\"{}\", value)),\n",
            upper_camel(&en.name)
        ));
        buf.push("        }\n");
        buf.push("    }\n");
        buf.push("}\n");
    }
}

fn generate_oneof<'a>(buf: &mut Buffer, oneof: &OneOf, cx: &Context<'a>) {
    for attr in cx.oneof_attributes() {
        buf.push(format!("{attr}\n"))
    }

    buf.push("#[derive(Debug)]\n");
    buf.push(format!("pub enum {} {{\n", upper_camel(&oneof.name)));
    for variant in &oneof.variants {
        let typ = match variant.typ.rust_type() {
            Some(typ) => typ.to_string(),
            None => match &variant.typ {
                FieldType::Message(typ) => match typ.rsplit_once(".") {
                    Some((path, typ)) => {
                        format!("{}::{}", path.replace('.', "::"), upper_camel(typ))
                    }
                    None => typ.to_string(),
                },
                _ => unreachable!(),
            },
        };

        buf.push(format!("    {}({}),\n", upper_camel(&variant.name), typ));
    }
    buf.push("}\n");
}

fn generate_field_type(typ: &FieldType, cx: &Context) -> String {
    match typ {
        FieldType::Double => "f64".to_string(),
        FieldType::Float => "f32".to_string(),
        FieldType::Int64 => "i64".to_string(),
        FieldType::Uint64 => "u64".to_string(),
        FieldType::Int32 => "i32".to_string(),
        FieldType::Fixed64 => "u64".to_string(),
        FieldType::Fixed32 => "u32".to_string(),
        FieldType::Bool => "bool".to_string(),
        FieldType::String => "String".to_string(),
        FieldType::Bytes => "Vec<u8>".to_string(),
        FieldType::Uint32 => "u32".to_string(),
        FieldType::Sfixed32 => "i32".to_string(),
        FieldType::Sfixed64 => "i64".to_string(),
        FieldType::Sint32 => "i32".to_string(),
        FieldType::Sint64 => "i64".to_string(),
        FieldType::Message(typ) => {
            match cx.lookup_type(typ) {
                Some((typ_with_path, _)) => typ_with_path,
                // this default typ, might not what we want
                None => typ.to_string(),
            }
        }
        FieldType::Map(key, value) => {
            let key = generate_field_type(key, cx);
            let value = generate_field_type(value, cx);
            format!("BTreeMap<{key}, {value}>")
        }
    }
}
