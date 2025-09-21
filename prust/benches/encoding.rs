use criterion::{Criterion, criterion_group, criterion_main};
use prust::{DecodeError, Reader};

pub fn read(c: &mut Criterion) {
    let data = include_bytes!("1000.data");
    c.bench_function("read_varint", |b| {
        b.iter(|| {
            let mut reader = Reader::new(data);
            while reader.pos < reader.src.len() {
                let _ = reader.read_varint().unwrap();
            }
        })
    });
    c.bench_function("read_varint v2", |b| {
        b.iter(|| {
            let mut reader = Reader2::new(data);
            while reader.len > 0 {
                let _ = reader.read_varint().unwrap();
            }
        })
    });

    let data = include_bytes!("1000_packed.data");
    c.bench_function("read_packed_varint", |b| {
        b.iter(|| {
            let mut reader = Reader::new(data);
            reader.read_packed(|buf| buf.read_varint()).unwrap();
        });
    });
}

struct Reader2 {
    pub src: *const u8,
    pub len: usize,
}

impl Reader2 {
    pub fn new(src: &[u8]) -> Reader2 {
        Reader2 {
            src: src.as_ptr(),
            len: src.len(),
        }
    }

    pub fn read_varint(&mut self) -> Result<u64, DecodeError> {
        let mut v = 0;
        let mut shifts = 0;
        let remaining = 10.min(self.len);
        while remaining > 0 {
            let b = unsafe { self.src.read() };
            self.src = self.src.wrapping_offset(1);
            self.len = self.len.wrapping_sub(1);

            v |= (b as u64) << shifts;
            if b & 0x80 == 0 {
                return Ok(v);
            }

            shifts += 7;
        }

        Err(DecodeError::Varint)
    }
}

criterion_group!(benches, read);
criterion_main!(benches);
