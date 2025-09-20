use std::collections::{BTreeMap, HashMap};

use super::lex::{Error as LexerError, Lexer, Span, Token};
use crate::ast::{
    Enum, Extension, Field, FieldType, FileDescriptor, Function, Label, Message, Method, OneOf,
    OneOfVarint, Reserved, Service, Syntax,
};
use crate::parse::unescape::unescape_c_escape_string;

const PROTOBUF_RESERVED: Reserved = Reserved::Range(19000, 19999);

/*
1 to 536,870,911
19,000 to 19,999
*/
#[derive(Debug)]
pub enum Error {
    Lex(LexerError),

    Unknown(String),

    Unexpected {
        token: String,
        span: Span,
        expected: String,
    },

    Eof,

    InvalidFieldNumber {
        number: i64,
        reason: String,
        span: Span,
    },

    InvalidMapKey(String),

    Duplicate(String),

    Unsupported(String),

    NotAllowed(String),
}

impl<'a> From<LexerError> for Error {
    fn from(err: LexerError) -> Self {
        match err {
            LexerError::Eof => Error::Eof,
            err => Error::Lex(err),
        }
    }
}

struct Context {
    syntax: Syntax,
}

pub fn parse(input: &[u8]) -> Result<FileDescriptor, Error> {
    let mut lexer = Lexer::new(input);

    let mut package = None;
    let mut imports = Vec::<String>::new();
    let mut options = HashMap::new();
    let mut messages = vec![];
    let mut enums = vec![];
    let mut services = vec![];

    let mut cx = Context {
        syntax: Default::default(),
    };

    while let Some(result) = lexer.next() {
        let (token, span) = result?;

        match token {
            Token::Ident(ident) => match ident {
                "syntax" => {
                    let syntax = parse_syntax(&mut lexer)?;
                    cx.syntax = syntax;
                }
                "package" => {
                    let ident = take_ident(&mut lexer)?;
                    package = Some(ident.to_string());
                    assert_next(&mut lexer, Token::Semicolon)?;
                }
                "import" => {
                    let (token, span) = take_next(&mut lexer)?;
                    match token {
                        Token::String(s) => {
                            if imports.iter().any(|item| item == s) {
                                return Err(Error::Duplicate(format!("duplicate import `{}`", s)));
                            }

                            imports.push(s.to_string());

                            assert_next(&mut lexer, Token::Semicolon)?;

                            continue;
                        }
                        _ => {
                            return Err(Error::Unexpected {
                                token: token.to_string(),
                                expected: "string".to_string(),
                                span,
                            });
                        }
                    }
                }
                "option" => {
                    let key = take_ident(&mut lexer)?;
                    assert_next(&mut lexer, Token::Equals)?;
                    let value = match lexer.next() {
                        Some(Ok((token, _span))) => match token {
                            Token::Ident(ident) => ident.to_string(),
                            Token::String(s) => s.to_string(),
                            Token::Integer(i) => i.to_string(),
                            _ => {
                                return Err(Error::Unexpected {
                                    token: token.to_string(),
                                    expected: "identity, integer, bool or string".to_string(),
                                    span,
                                });
                            }
                        },
                        Some(Err(err)) => return Err(err.into()),
                        None => return Err(Error::Eof),
                    };

                    assert_next(&mut lexer, Token::Semicolon)?;

                    options.insert(key.to_string(), value);
                }
                "message" => {
                    let msg = parse_message(&mut lexer, &mut cx)?;
                    // todo: check msg name, return err is it is duplicate

                    messages.push(msg);
                    continue;
                }
                "enum" => {
                    let enumerator = parse_enum(&mut lexer, &mut cx)?;
                    enums.push(enumerator);
                    continue;
                }
                "service" => {
                    let service = parse_service(&mut lexer)?;
                    services.push(service);
                    continue;
                }
                _ => {
                    panic!("{}", lexer.diagnostic(span, "unexpected token"));
                }
            },
            Token::Semicolon => {}
            token => {
                panic!(
                    "{}",
                    lexer.diagnostic(span, format!("unexpected token `{token:?}`"))
                );
            }
        }
    }

    Ok(FileDescriptor {
        syntax: cx.syntax,
        package,
        options,
        imports,
        messages,
        enums,
        services,
    })
}

fn parse_syntax(lexer: &mut Lexer) -> Result<Syntax, Error> {
    assert_next(lexer, Token::Equals)?;

    match lexer.next() {
        Some(Ok((token, _span))) => {
            let syntax = if token == Token::String("proto2") {
                Syntax::Proto2
            } else if token == Token::String("proto3") {
                Syntax::Proto3
            } else {
                return Err(Error::Unknown(token.to_string()));
            };

            assert_next(lexer, Token::Semicolon)?;

            Ok(syntax)
        }
        Some(Err(err)) => Err(err.into()),
        None => Err(Error::Eof),
    }
}

fn take_next<'a>(lexer: &mut Lexer<'a>) -> Result<(Token<'a>, Span), Error> {
    match lexer.next() {
        Some(Ok((token, span))) => Ok((token, span)),
        Some(Err(err)) => Err(err.into()),
        None => Err(Error::Eof),
    }
}

fn assert_next(lexer: &mut Lexer, expected: Token) -> Result<(), Error> {
    let (token, span) = take_next(lexer)?;
    if token != expected {
        panic!(
            "{}",
            lexer.diagnostic(span, format!("expected token {:?}", expected))
        );
    }

    Ok(())
}

fn take_ident<'a>(lexer: &mut Lexer<'a>) -> Result<&'a str, Error> {
    match lexer.next() {
        Some(Ok((token, span))) => match token {
            Token::Ident(ident) => Ok(ident),
            _ => panic!(
                "{}",
                lexer.diagnostic(span, format!("unexpected token {:?}", token))
            ),
        },
        Some(Err(err)) => Err(err.into()),
        None => Err(Error::Eof),
    }
}

fn parse_field_type(lexer: &mut Lexer) -> Result<FieldType, Error> {
    let (token, span) = take_next(lexer)?;

    match token {
        // map
        Token::LeftAngleBracket => {
            // key_typ can be any scaler type but float types
            let key_typ = take_ident(lexer)?;
            let key = FieldType::from(key_typ);
            match key {
                FieldType::Int32
                | FieldType::Int64
                | FieldType::Uint32
                | FieldType::Uint64
                | FieldType::Sint32
                | FieldType::Sint64
                | FieldType::Fixed32
                | FieldType::Fixed64
                | FieldType::Sfixed32
                | FieldType::Sfixed64
                | FieldType::Bool
                | FieldType::String => {}
                _ => return Err(Error::InvalidMapKey(key_typ.to_string())),
            }

            assert_next(lexer, Token::Comma)?;

            // value_typ can be any type but another map, the next
            // `assert_token` can handle this
            let value_typ = take_ident(lexer)?;
            assert_next(lexer, Token::RightAngleBracket)?;
            let value = FieldType::from(value_typ);

            Ok(FieldType::Map(Box::new(key), Box::new(value)))
        }
        Token::Ident(ident) => Ok(FieldType::from(ident)),
        _ => Err(Error::Unexpected {
            expected: "a valid field type".to_string(),
            token: token.to_string(),
            span,
        }),
    }
}

fn parse_field_and_next(
    lexer: &mut Lexer,
    cx: &Context,
) -> Result<(String, u32, HashMap<String, String>), Error> {
    let name = take_ident(lexer)?.to_string();

    assert_next(lexer, Token::Equals)?;

    let number = match lexer.next() {
        Some(Ok((token, span))) => match token {
            Token::Integer(value) => {
                u32::try_from(value).map_err(|err| Error::InvalidFieldNumber {
                    number: value,
                    reason: err.to_string(),
                    span,
                })?
            }
            _ => {
                return Err(Error::Unexpected {
                    token: token.to_string(),
                    expected: "unsigned integer".to_string(),
                    span,
                });
            }
        },
        Some(Err(err)) => return Err(err.into()),
        None => return Err(Error::Eof),
    };

    let extensions = match lexer.next() {
        Some(Ok((token, span))) => match token {
            Token::Semicolon => {
                // all done
                Default::default()
            }
            Token::LeftBracket => parse_field_options(lexer, cx)?,
            _ => {
                return Err(Error::Unexpected {
                    token: token.to_string(),
                    expected: "semicolon or left bracket".to_string(),
                    span,
                });
            }
        },
        Some(Err(err)) => return Err(err.into()),
        None => return Err(Error::Eof),
    };

    Ok((name, number, extensions))
}

fn parse_field(lexer: &mut Lexer, cx: &Context) -> Result<Field, Error> {
    let typ = FieldType::from(take_ident(lexer)?);
    let name = take_ident(lexer)?.to_string();

    assert_next(lexer, Token::Equals)?;

    let number = match lexer.next() {
        Some(Ok((token, span))) => match token {
            Token::Integer(value) => {
                if value < 1 || value > (1 << 29) - 1 {
                    return Err(Error::InvalidFieldNumber {
                        number: value,
                        reason: "field number must in [1, 536,870,911]".to_string(),
                        span,
                    });
                }

                value as u32
            }
            _ => {
                return Err(Error::Unexpected {
                    token: token.to_string(),
                    expected: "unsigned integer".to_string(),
                    span,
                });
            }
        },
        Some(Err(err)) => return Err(err.into()),
        None => return Err(Error::Eof),
    };

    let options = match lexer.next() {
        Some(Ok((token, span))) => match token {
            Token::Semicolon => {
                // all done
                Default::default()
            }
            Token::LeftBracket => parse_field_options(lexer, cx)?,
            _ => {
                panic!(
                    "{}",
                    lexer.diagnostic(span, "expected token semicolon or left bracket")
                );
            }
        },
        Some(Err(err)) => return Err(err.into()),
        None => return Err(Error::Eof),
    };

    Ok(Field {
        label: Label::Optional,
        typ,
        name,
        number,
        options,
    })
}

fn parse_field_options<'a>(
    lexer: &mut Lexer,
    cx: &Context,
) -> Result<HashMap<String, String>, Error> {
    let mut extensions = HashMap::new();

    loop {
        let (token, span) = take_next(lexer)?;
        let key = match token {
            Token::Ident(ident) => ident,
            Token::LeftParentheses => {
                // something like `[(gogoproto.nullable) = false];`
                let key = take_ident(lexer)?;
                assert_next(lexer, Token::RightParentheses)?;
                key
            }
            _ => {
                panic!("{}", lexer.diagnostic(span, "unknown token"));
            }
        };

        if cx.syntax == Syntax::Proto3 && key == "default" {
            return Err(Error::NotAllowed(
                "explicit default values are not allowed in proto3".to_string(),
            ));
        }

        if extensions.contains_key(key) {
            return Err(Error::Duplicate(key.to_string()));
        }

        assert_next(lexer, Token::Equals)?;

        // value part
        let (token, span) = take_next(lexer)?;
        let value = match token {
            Token::Ident(ident) => ident.to_string(),
            Token::Integer(i) => i.to_string(),
            Token::Float(f) => f.to_string(),
            Token::String(s) => {
                let mut buf = String::new();
                for b in unescape_c_escape_string(s) {
                    buf.extend((b as char).escape_default());
                }

                buf
            }
            _ => {
                panic!(
                    "{}",
                    lexer.diagnostic(span, "identity, integer, float, bool or string")
                );
            }
        };

        extensions.insert(key.to_string(), value);

        let (token, span) = take_next(lexer)?;
        match token {
            Token::Comma => continue,
            Token::RightBracket => break,
            _ => {
                return Err(Error::Unexpected {
                    token: token.to_string(),
                    expected: "comma or bracket".to_string(),
                    span,
                });
            }
        }
    }

    assert_next(lexer, Token::Semicolon)?;

    Ok(extensions)
}

fn parse_enum(lexer: &mut Lexer, cx: &mut Context) -> Result<Enum, Error> {
    let name = take_ident(lexer)?;

    assert_next(lexer, Token::LeftBrace)?;

    let mut variants = Vec::new();

    loop {
        let (token, span) = take_next(lexer)?;
        let variant = match token {
            Token::RightBrace => break,
            Token::Ident(ident) => {
                if variants.iter().any(|(v, _)| v == ident) {
                    return Err(Error::Duplicate(format!("duplicate variant {ident}")));
                }

                ident
            }
            _ => {
                return Err(Error::Unexpected {
                    token: token.to_string(),
                    expected: "enum variant or right brace".to_string(),
                    span,
                });
            }
        };

        assert_next(lexer, Token::Equals)?;

        let (token, span) = take_next(lexer)?;
        let value = match token {
            // enum variant value could be negative
            Token::Integer(value) => match i32::try_from(value) {
                Ok(value) => value,
                Err(err) => panic!("{}", lexer.diagnostic(span, &err.to_string())),
            },
            _ => {
                panic!("{}", lexer.diagnostic(span, "expect an int32"));
            }
        };

        if cx.syntax == Syntax::Proto3 && variants.is_empty() && value != 0 {
            panic!(
                "{}",
                lexer.diagnostic(span, "first variant of enum must be zero in proto3")
            );
        }

        if variants.iter().any(|(n, v)| variant == n || value == *v) {
            return Err(Error::Duplicate(format!("duplicate tag {value}")));
        }

        assert_next(lexer, Token::Semicolon)?;

        variants.push((variant.to_string(), value));
    }

    Ok(Enum {
        name: name.to_string(),
        variants,
    })
}

fn parse_oneof(lexer: &mut Lexer, cx: &Context) -> Result<OneOf, Error> {
    let name = take_ident(lexer)?;

    assert_next(lexer, Token::LeftBrace)?;

    let mut variants = vec![];

    loop {
        let (token, span) = take_next(lexer)?;
        let typ = match token {
            Token::RightBrace => break,
            Token::Ident(typ) => FieldType::from(typ),
            _ => {
                return Err(Error::Unexpected {
                    token: token.to_string(),
                    expected: "enum variant or right brace".to_string(),
                    span,
                });
            }
        };

        let name = take_ident(lexer)?;

        assert_next(lexer, Token::Equals)?;

        let (token, span) = take_next(lexer)?;
        let number = match token {
            // enum variant value could be negative
            Token::Integer(value) => match u32::try_from(value) {
                Ok(value) => value,
                Err(err) => panic!("{}", lexer.diagnostic(span, &err.to_string())),
            },
            _ => {
                panic!("{}", lexer.diagnostic(span, "expect an uint32"));
            }
        };

        let (token, span) = take_next(lexer)?;
        let options = match token {
            Token::Semicolon => HashMap::new(),
            Token::LeftBracket => parse_field_options(lexer, cx)?,
            _ => panic!("{}", lexer.diagnostic(span, "unexpected token")),
        };

        variants.push(OneOfVarint {
            name: name.to_string(),
            number,
            typ,
            options,
        })
    }

    Ok(OneOf {
        name: name.to_string(),
        variants,
    })
}

// parse reserved
//
// - reserved 1;
// - reserved 1, 2;
// - reserved 1 to 2;
// - reserved 1, 2 to 3, 4 to max;
fn parse_reserved(lexer: &mut Lexer) -> Result<Vec<Reserved>, Error> {
    let mut reserved = Vec::new();

    loop {
        let (token, span) = take_next(lexer)?;
        let value = match token {
            Token::Integer(value) => {
                u32::try_from(value).map_err(|err| Error::InvalidFieldNumber {
                    number: value,
                    reason: err.to_string(),
                    span,
                })?
            }
            Token::Semicolon => {
                if reserved.is_empty() {
                    return Err(Error::Unexpected {
                        token: token.to_string(),
                        expected: "valid reserved values".to_string(),
                        span,
                    });
                }
                break;
            }
            _ => {
                return Err(Error::Unexpected {
                    token: token.to_string(),
                    expected: "".to_string(),
                    span,
                });
            }
        };

        let (token, span) = take_next(lexer)?;
        match token {
            Token::Comma => {
                reserved.push(Reserved::Single(value));
                continue;
            }
            Token::Semicolon => {
                reserved.push(Reserved::Single(value));
                break;
            }
            Token::Ident(ident) => {
                if ident != "to" {
                    return Err(Error::Unexpected {
                        token: token.to_string(),
                        expected: "comma, semicolon or `to`".to_string(),
                        span,
                    });
                }

                let (token, span) = take_next(lexer)?;
                match token {
                    Token::Integer(v) => {
                        let end = u32::try_from(v).map_err(|err| Error::InvalidFieldNumber {
                            number: v,
                            reason: err.to_string(),
                            span,
                        })?;
                        reserved.push(Reserved::Range(value, end));
                    }
                    Token::Ident(ident) if ident == "max" => {
                        reserved.push(Reserved::Range(value, u32::MAX));
                    }
                    _ => {
                        return Err(Error::Unexpected {
                            token: token.to_string(),
                            expected: "comma, semicolon or `to`".to_string(),
                            span,
                        });
                    }
                }
            }
            _ => {
                return Err(Error::Unexpected {
                    token: token.to_string(),
                    expected: "comma, semicolon or `to`".to_string(),
                    span,
                });
            }
        }
    }

    Ok(reserved)
}

fn parse_message(lexer: &mut Lexer, cx: &mut Context) -> Result<Message, Error> {
    let name = take_ident(lexer)?;

    assert_next(lexer, Token::LeftBrace)?;

    let mut fields: Vec<Field> = vec![];
    let mut messages = vec![];
    let mut options = HashMap::new();
    let mut enums = vec![];
    let mut oneofs = vec![];
    let mut reserved = vec![];
    let mut extensions = vec![];

    loop {
        let (ident, span) = match lexer.next() {
            Some(Ok((token, span))) => match token {
                Token::Ident(ident) => (ident, span),
                Token::RightBrace => break,
                Token::Semicolon => continue,
                _ => {
                    panic!("{}", lexer.diagnostic(span, "unexpected token"));
                }
            },
            Some(Err(err)) => return Err(err.into()),
            None => return Err(Error::Eof),
        };

        if ident == "message" {
            let msg = parse_message(lexer, cx)?;
            messages.push(msg);
            continue;
        }

        if ident == "enum" {
            let enumerator = parse_enum(lexer, cx)?;
            enums.push(enumerator);
            continue;
        }

        if ident == "oneof" {
            let oneof = parse_oneof(lexer, cx)?;
            oneofs.push(oneof);
            continue;
        }

        if ident == "group" {
            return Err(Error::Unsupported(
                "group is not supported, because it is deprecated already".to_string(),
            ));
        }

        if ident == "option" {
            let key = take_ident(lexer)?;
            assert_next(lexer, Token::Equals)?;
            let (token, span) = take_next(lexer)?;
            let value = match token {
                Token::Ident(ident) => ident.to_string(),
                Token::String(s) => s.to_string(),
                _ => {
                    panic!(
                        "{}",
                        lexer.diagnostic(span, "unknown token for option value")
                    )
                }
            };

            options.insert(key.to_string(), value);

            continue;
        }

        if ident == "reserved" {
            let partial = parse_reserved(lexer)?;

            for field in &fields {
                if !validate_number(field.number, &reserved) {
                    return Err(Error::InvalidFieldNumber {
                        number: field.number as i64,
                        reason: "reserved a predefined field".to_string(),
                        span,
                    });
                }
            }

            for oneof in &oneofs {
                for variant in &oneof.variants {
                    if !validate_number(variant.number, &reserved) {
                        return Err(Error::InvalidFieldNumber {
                            number: variant.number as i64,
                            reason: "reserved a predefined file".to_string(),
                            span,
                        });
                    }
                }
            }

            reserved.extend(partial);

            continue;
        }

        if ident == "map" {
            let typ = parse_field_type(lexer)?;
            let (name, number, options) = parse_field_and_next(lexer, cx)?;
            if PROTOBUF_RESERVED.contains(number) {
                return Err(Error::InvalidFieldNumber {
                    number: number as i64,
                    reason: format!(
                        "field number {number} is reserved for Protobuf implementation"
                    ),
                    span,
                });
            }

            if options.get("deprecated").map(|v| v.as_str()) == Some("true") {
                continue;
            }

            fields.push(Field {
                // todo: handle label by syntax
                label: Label::Required,
                typ,
                name,
                number,
                options,
            });
            continue;
        }

        if ident == "extensions" {
            let ext = parse_message_extensions(lexer, cx)?;
            extensions.push(ext);
            continue;
        }

        let label = match cx.syntax {
            Syntax::Proto2 => match ident {
                "required" => Label::Required,
                "optional" => Label::Optional,
                "repeated" => Label::Repeated,
                _ => {
                    panic!("{}", lexer.diagnostic(span, "unexpected token"))
                }
            },
            Syntax::Proto3 => match ident {
                "optional" => Label::Optional,
                "repeated" => Label::Repeated,
                "required" => {
                    return Err(Error::Unexpected {
                        token: ident.to_string(),
                        expected: "optional, repeated or map".to_string(),
                        span,
                    });
                }
                ident => {
                    let typ = FieldType::from(ident);
                    let (name, number, options) = parse_field_and_next(lexer, cx)?;
                    if options.get("deprecated").map(|x| x.as_str()) != Some("true") {
                        fields.push(Field {
                            label: Label::Optional,
                            typ,
                            name,
                            number,
                            options,
                        });
                    }

                    continue;
                }
            },
            _ => panic!("{}", lexer.diagnostic(span, "unknown field")),
        };

        let mut field = parse_field(lexer, cx)?;
        if field.options.get("deprecated").map(|v| v.as_str()) == Some("true") {
            continue;
        }
        field.label = label;
        fields.push(field);
    }

    Ok(Message {
        name: name.to_string(),
        fields,
        reserved,
        options,
        messages,
        enums,
        oneofs,
    })
}

fn parse_message_extensions(lexer: &mut Lexer, _cx: &Context) -> Result<Extension, Error> {
    let (token, span) = take_next(lexer)?;
    let start = match token {
        Token::Integer(value) => match u32::try_from(value) {
            Ok(value) => value,
            Err(err) => panic!("{}", lexer.diagnostic(span, err.to_string())),
        },
        _ => panic!("{}", lexer.diagnostic(span, "unknown token")),
    };

    assert_next(lexer, Token::Ident("to"))?;

    let (token, span) = take_next(lexer)?;
    let end = match token {
        Token::Ident(ident) if ident == "max" => u32::MAX,
        Token::Integer(value) => match u32::try_from(value) {
            Ok(value) => value,
            Err(err) => panic!("{}", lexer.diagnostic(span, err.to_string())),
        },
        _ => panic!("{}", lexer.diagnostic(span, "unknown token")),
    };

    let (token, _span) = take_next(lexer)?;
    match token {
        Token::LeftBracket => {}
        Token::Semicolon => {
            return Ok(Extension {
                start,
                end,
                properties: BTreeMap::new(),
            });
        }
        _ => {
            panic!(
                "{}",
                lexer.diagnostic(span, format!("unexpected token {:?}", token.to_string()))
            )
        }
    }

    assert_next(lexer, Token::LeftBracket)?;

    let mut properties = BTreeMap::new();
    loop {
        let (token, span) = take_next(lexer)?;
        let key = match token {
            Token::Ident(key) => key.to_string(),
            Token::RightBracket => break,
            _ => panic!("{}", lexer.diagnostic(span, "unknown token")),
        };

        assert_next(lexer, Token::Equals)?;

        let (token, span) = take_next(lexer)?;
        let value = match token {
            Token::Ident(value) => value.to_string(),
            Token::Integer(value) => value.to_string(),
            Token::Float(value) => value.to_string(),
            Token::String(value) => value.to_string(),
            _ => panic!("{}", lexer.diagnostic(span, "unknown token")),
        };

        properties.insert(key, value);
    }

    Ok(Extension {
        start,
        end,
        properties,
    })
}

fn parse_service(lexer: &mut Lexer) -> Result<Service, Error> {
    let name = take_ident(lexer)?;

    assert_next(lexer, Token::LeftBrace)?;

    let mut functions = Vec::new();
    loop {
        let (token, span) = take_next(lexer)?;
        match token {
            Token::RightBrace => {
                break;
            }
            Token::Ident(ident) if ident == "rpc" => {}
            _ => {
                panic!("{}", lexer.diagnostic(span, "unexpected token"))
            }
        }

        let name = take_ident(lexer)?;

        //
        assert_next(lexer, Token::LeftParentheses)?;

        let ident = take_ident(lexer)?;
        let (client_stream, request) = if ident == "stream" {
            let ident = take_ident(lexer)?;
            (true, ident)
        } else {
            (false, ident)
        };

        assert_next(lexer, Token::RightParentheses)?;

        assert_next(lexer, Token::Ident("returns"))?;

        assert_next(lexer, Token::LeftParentheses)?;

        let ident = take_ident(lexer)?;
        let (server_stream, response) = if ident == "stream" {
            let ident = take_ident(lexer)?;
            (true, ident)
        } else {
            (false, ident)
        };

        assert_next(lexer, Token::RightParentheses)?;

        match take_next(lexer)? {
            (Token::LeftBrace, _) => {
                assert_next(lexer, Token::RightBrace)?;
            }
            (Token::Semicolon, _) => {
                // do nothing
            }
            (_, span) => {
                panic!("{}", lexer.diagnostic(span, "unexpected token"))
            }
        }

        let method = match (client_stream, server_stream) {
            (false, false) => Method::Unary,
            (true, false) => Method::ClientStreaming,
            (false, true) => Method::ServerStreaming,
            (true, true) => Method::BidiStreaming,
        };

        functions.push(Function {
            name: name.to_string(),
            method,
            request: request.to_string(),
            response: response.to_string(),
        })
    }

    Ok(Service {
        name: name.to_string(),
        functions,
    })
}

fn validate_number(tag: u32, all: &[Reserved]) -> bool {
    for reserved in all {
        match reserved {
            Reserved::Single(value) => {
                if tag == *value {
                    return false;
                }
            }
            Reserved::Range(start, end) => {
                if tag >= *start && tag < *end {
                    return false;
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service() {
        // service is consumed already
        let input = r#"
        SearchService {
            rpc GetFeature(Point) returns (Feature) {}

            rpc Get(Point) returns (Feature);

            rpc ListFeatures(Rectangle) returns (stream Feature) {}
            rpc RecordRoute(stream Point) returns (RouteSummary) {}
            rpc RouteChat(stream RouteNote) returns (stream RouteNote) {}
        }
        "#;

        let mut lexer = Lexer::new(input.as_bytes());
        let service = parse_service(&mut lexer).unwrap();

        assert_eq!(
            service,
            Service {
                name: "SearchService".to_string(),
                functions: vec![
                    Function {
                        name: "GetFeature".to_string(),
                        method: Method::Unary,
                        request: "Point".to_string(),
                        response: "Feature".to_string(),
                    },
                    Function {
                        name: "Get".to_string(),
                        method: Method::Unary,
                        request: "Point".to_string(),
                        response: "Feature".to_string(),
                    },
                    Function {
                        name: "ListFeatures".to_string(),
                        method: Method::ServerStreaming,
                        request: "Rectangle".to_string(),
                        response: "Feature".to_string(),
                    },
                    Function {
                        name: "RecordRoute".to_string(),
                        method: Method::ClientStreaming,
                        request: "Point".to_string(),
                        response: "RouteSummary".to_string(),
                    },
                    Function {
                        name: "RouteChat".to_string(),
                        method: Method::BidiStreaming,
                        request: "RouteNote".to_string(),
                        response: "RouteNote".to_string(),
                    },
                ]
            }
        )
    }
}
