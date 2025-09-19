mod config;
mod context;
mod deserialize;
mod generate;
mod sanitize;
mod serialize;
mod service;
mod sizeof;

pub use config::Config;

#[derive(Default)]
pub struct Buffer {
    inner: String,
    pub indent: usize,
}

impl Buffer {
    pub fn push(&mut self, s: impl AsRef<str>) {
        for _ in 0..self.indent {
            self.inner.push_str("    ");
        }

        self.inner.push_str(s.as_ref());
    }

    #[inline]
    pub fn into_inner(self) -> String {
        self.inner
    }

    pub fn block(&mut self, f: impl FnOnce()) {
        self.indent += 1;
        f();
        self.indent -= 1;
    }
}

pub trait ServiceGenerator {
    fn generate(&self, buf: &mut Buffer, cx: &context::Context);
}
