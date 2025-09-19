mod ast;
mod codegen;
mod parse;

use std::path::Path;

pub use codegen::Config;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),

    Parse(parse::Error),

    ImportNotFound(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

pub fn compile<P: AsRef<Path>>(includes: &[P], files: &[P]) -> Result<(), Error> {
    Config::default().compile(includes, files)
}
