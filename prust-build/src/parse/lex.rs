use std::fmt::{Display, Formatter};
use std::ops::{Neg, Range};

#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    Ident(&'a str),
    Integer(i64),
    Float(f64),
    String(&'a str),
    LeftBrace,         // {
    RightBrace,        // }
    LeftBracket,       // [
    RightBracket,      // ]
    LeftParentheses,   // (
    RightParentheses,  // )
    LeftAngleBracket,  // <
    RightAngleBracket, // >
    Comma,             // ,
    Equals,            // =
    Colon,             // :
    Semicolon,         // ;
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(ident) => f.write_str(ident),
            Token::Integer(v) => v.fmt(f),
            Token::Float(v) => v.fmt(f),
            Token::String(s) => f.write_str(s),
            Token::LeftBrace => f.write_str("{"),
            Token::RightBrace => f.write_str("}"),
            Token::LeftBracket => f.write_str("["),
            Token::RightBracket => f.write_str("]"),
            Token::LeftParentheses => f.write_str("("),
            Token::RightParentheses => f.write_str(")"),
            Token::LeftAngleBracket => f.write_str("<"),
            Token::RightAngleBracket => f.write_str(">"),
            Token::Comma => f.write_str(","),
            Token::Equals => f.write_str("="),
            Token::Colon => f.write_str(":"),
            Token::Semicolon => f.write_str(";"),
        }
    }
}

pub struct Lexer<'a> {
    source: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self { source, pos: 0 }
    }

    fn skip_whitespaces(&mut self) {
        while self.pos < self.source.len() {
            let ch = self.source[self.pos];
            if ch.is_ascii_whitespace() {
                self.pos += 1;
                continue;
            }

            // handle comment
            if ch == b'/' {
                if self.pos + 1 < self.source.len() {
                    let next = self.source[self.pos + 1];
                    if next == b'/' {
                        // meet comments with double slash
                        while self.pos < self.source.len() && self.source[self.pos] != b'\n' {
                            self.pos += 1;
                        }

                        // this line skipped already
                        self.pos += 1;
                    } else if next == b'*' {
                        /* meet comment with slash and start */
                        while self.pos < self.source.len() {
                            self.pos += 1;

                            let ch = self.source[self.pos];
                            if ch == b'/' && self.source[self.pos - 1] == b'*' {
                                self.pos += 1;
                                break;
                            }
                        }
                    }

                    continue;
                }
            }

            break;
        }
    }

    pub fn diagnostic(&self, span: Span, msg: impl AsRef<str>) -> String {
        use std::iter::repeat_n;

        let mut pending = 0;
        let mut start = 0;
        let mut lines = 1;
        for pos in (0..span.start).rev() {
            let ch = self.source[pos];
            if ch == b'\n' {
                if pending != 0 {
                    pending = pos;
                }
                lines += 1;

                if start == 0 {
                    start = pos + 1;
                }
            }
        }

        let mut end = span.end;
        while end < self.source.len() {
            if self.source[end] == b'\n' {
                break;
            }

            end += 1;
        }

        let line_part = lines.to_string();

        let mut output = String::new();
        output.push_str(&format!(
            "{}: {}\n",
            line_part,
            String::from_utf8_lossy(&self.source[start..end])
        ));
        output.push_str(&format!(
            "{}{} {}",
            repeat_n(' ', line_part.len() + 2 + pending).collect::<String>(),
            repeat_n('-', span.end - span.start).collect::<String>(),
            msg.as_ref()
        ));

        output
    }
}

#[derive(Debug)]
pub enum Error {
    Unknown,

    Eof,

    // underflow or overflow
    InvalidInteger,

    InvalidFlota,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<(Token<'a>, Span), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.skip_whitespaces();

            let start = self.pos;
            while self.pos < self.source.len() {
                let ch = self.source[self.pos];

                match ch {
                    b'=' => {
                        self.pos += 1;
                        return Some(Ok((
                            Token::Equals,
                            Span {
                                start,
                                end: self.pos,
                            },
                        )));
                    }
                    b'{' => {
                        self.pos += 1;
                        return Some(Ok((
                            Token::LeftBrace,
                            Span {
                                start,
                                end: self.pos,
                            },
                        )));
                    }
                    b'}' => {
                        self.pos += 1;
                        return Some(Ok((
                            Token::RightBrace,
                            Span {
                                start,
                                end: self.pos,
                            },
                        )));
                    }
                    b'[' => {
                        self.pos += 1;
                        return Some(Ok((
                            Token::LeftBracket,
                            Span {
                                start,
                                end: self.pos,
                            },
                        )));
                    }
                    b']' => {
                        self.pos += 1;
                        return Some(Ok((
                            Token::RightBracket,
                            Span {
                                start,
                                end: self.pos,
                            },
                        )));
                    }
                    b'(' => {
                        self.pos += 1;
                        return Some(Ok((
                            Token::LeftParentheses,
                            Span {
                                start,
                                end: self.pos,
                            },
                        )));
                    }
                    b')' => {
                        self.pos += 1;
                        return Some(Ok((
                            Token::RightParentheses,
                            Span {
                                start,
                                end: self.pos,
                            },
                        )));
                    }
                    b'<' => {
                        self.pos += 1;
                        return Some(Ok((
                            Token::LeftAngleBracket,
                            Span {
                                start,
                                end: self.pos,
                            },
                        )));
                    }
                    b'>' => {
                        self.pos += 1;
                        return Some(Ok((
                            Token::RightAngleBracket,
                            Span {
                                start,
                                end: self.pos,
                            },
                        )));
                    }
                    b',' => {
                        self.pos += 1;
                        return Some(Ok((
                            Token::Comma,
                            Span {
                                start,
                                end: self.pos,
                            },
                        )));
                    }
                    b':' => {
                        self.pos += 1;
                        return Some(Ok((
                            Token::Colon,
                            Span {
                                start,
                                end: self.pos,
                            },
                        )));
                    }
                    b';' => {
                        self.pos += 1;
                        return Some(Ok((
                            Token::Semicolon,
                            Span {
                                start,
                                end: self.pos,
                            },
                        )));
                    }
                    b'\'' => {
                        let start = self.pos + 1;

                        while self.pos < self.source.len() {
                            self.pos += 1;

                            let ch = self.source[self.pos];
                            if ch == b'\'' {
                                if self.source[self.pos - 1] != b'\\' {
                                    let s = unsafe {
                                        std::str::from_utf8_unchecked(&self.source[start..self.pos])
                                    };
                                    let span = Span {
                                        start,
                                        end: self.pos,
                                    };
                                    self.pos += 1;
                                    return Some(Ok((Token::String(s), span)));
                                }
                            }
                        }

                        return Some(Err(Error::Unknown));
                    }
                    b'"' => {
                        let start = self.pos + 1;

                        while self.pos < self.source.len() {
                            self.pos += 1;

                            let ch = self.source[self.pos];
                            if ch == b'"' {
                                if self.source[self.pos - 1] != b'\\' {
                                    let s = unsafe {
                                        std::str::from_utf8_unchecked(&self.source[start..self.pos])
                                    };
                                    let span = Span {
                                        start,
                                        end: self.pos,
                                    };
                                    self.pos += 1;
                                    return Some(Ok((Token::String(s), span)));
                                }
                            }
                        }

                        return Some(Err(Error::Unknown));
                    }
                    b'a'..=b'z' | b'A'..=b'Z' => {
                        let start = self.pos;
                        while self.pos < self.source.len() {
                            let ch = self.source[self.pos];
                            if ch.is_ascii_whitespace()
                                || ch == b':'
                                || ch == b';'
                                || ch == b']'
                                || ch == b')'
                                || ch == b'('
                                || ch == b','
                                || ch == b'='
                                || ch == b'-'
                                || ch == b'<'
                                || ch == b'>'
                            {
                                let ident = unsafe {
                                    std::str::from_utf8_unchecked(&self.source[start..self.pos])
                                };

                                return Some(Ok((
                                    Token::Ident(ident),
                                    Span {
                                        start,
                                        end: self.pos,
                                    },
                                )));
                            }

                            self.pos += 1;
                        }

                        return Some(Err(Error::Unknown));
                    }
                    // tag is u32, but option's value field might be negative
                    b'-' | b'0'..=b'9' => {
                        let start = self.pos;
                        let negative = if ch == b'-' {
                            self.pos += 1;
                            true
                        } else {
                            false
                        };

                        let mut dots = 0;
                        while self.pos < self.source.len() {
                            let ch = self.source[self.pos];
                            if ch == b'.' {
                                self.pos += 1;
                                dots += 1;
                                continue;
                            }

                            if ch.is_ascii_digit() || b"afin".contains(&ch) {
                                self.pos += 1;
                                continue;
                            } else {
                                break;
                            }
                        }

                        if dots >= 2 {
                            return Some(Err(Error::Unknown));
                        }

                        let n = &self.source[start + negative as usize..self.pos];
                        let s = unsafe { std::str::from_utf8_unchecked(n) };
                        return if dots == 0 {
                            if s == "nan" {
                                if negative {
                                    return Some(Err(Error::Unknown));
                                }

                                return Some(Ok((
                                    Token::Float(f64::NAN),
                                    Span {
                                        start,
                                        end: self.pos,
                                    },
                                )));
                            }
                            if s == "inf" {
                                let v = if negative {
                                    f64::NEG_INFINITY
                                } else {
                                    f64::INFINITY
                                };
                                return Some(Ok((
                                    Token::Float(v),
                                    Span {
                                        start,
                                        end: self.pos,
                                    },
                                )));
                            }
                            // integer
                            match s.parse::<i64>().map_err(|_err| Error::InvalidInteger) {
                                Ok(value) => Some(Ok((
                                    Token::Integer(if negative { value.neg() } else { value }),
                                    Span {
                                        start,
                                        end: self.pos,
                                    },
                                ))),
                                Err(_err) => Some(Err(Error::InvalidInteger)),
                            }
                        } else {
                            // float
                            match s.parse::<f64>().map_err(|_err| Error::InvalidInteger) {
                                Ok(value) => Some(Ok((
                                    Token::Float(if negative { value.neg() } else { value }),
                                    Span {
                                        start,
                                        end: self.pos,
                                    },
                                ))),
                                Err(_err) => Some(Err(Error::InvalidFlota)),
                            }
                        };
                    }
                    _ => return Some(Err(Error::Unknown)),
                }
            }

            return None;
        }
    }
}
