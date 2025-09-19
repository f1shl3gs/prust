#[derive(Debug, PartialEq)]
pub enum Method {
    Unary,
    ClientStreaming,
    ServerStreaming,
    BidiStreaming,
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub method: Method,

    pub request: String,
    pub response: String,
}

#[derive(Debug, PartialEq)]
pub struct Service {
    pub name: String,

    pub functions: Vec<Function>,
}
