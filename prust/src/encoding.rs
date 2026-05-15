#[derive(Debug, PartialEq)]
pub enum DecodeError {
    Malformed,
    // Unexpected EOF, buffer is too short
    Eof,
    // Invalid varint.
    Varint,
    // Unknown WireType
    WireType(u8),
    // Deprecated feature (in protocol buffer specification)
    Deprecated(&'static str),
    // Unknown variant of enum
    UnknownVariant(&'static str, i32),
    // utf8 validate error
    Utf8,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::Malformed => f.write_str("malformed data"),
            DecodeError::Eof => f.write_str("unexpected EOF"),
            DecodeError::Varint => f.write_str("invalid varint"),
            DecodeError::WireType(typ) => write!(f, "unknown wire type: {}", typ),
            DecodeError::Deprecated(typ) => write!(f, "deprecated \"{}\" is not supported", typ),
            DecodeError::UnknownVariant(typ, value) => {
                write!(f, "unknown enum value {typ}: {value}")
            }
            DecodeError::Utf8 => f.write_str("invalid UTF-8"),
        }
    }
}

impl std::error::Error for DecodeError {}

pub trait Deserialize: Sized {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError>;
}

/// EncodeError returned when encoding
#[derive(Debug)]
pub enum EncodeError {
    // Unexpected EOF, buffer is too short
    Eof,
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodeError::Eof => f.write_str("unexpected EOF"),
        }
    }
}

pub trait Serialize: Sized {
    fn encoded_len(&self) -> usize;

    fn encode(&self, buf: &mut [u8]) -> Result<usize, EncodeError>;
}

#[inline]
pub fn sizeof_int32(v: i32) -> usize {
    if v < 0 {
        // optimized for negative integer
        10
    } else if v < 1 << 7 {
        1
    } else if v < 1 << 14 {
        2
    } else if v < 1 << 21 {
        3
    } else if v < 1 << 28 {
        4
    } else {
        5
    }
}

#[inline]
fn encode_zigzag32(v: i32) -> u32 {
    ((v as u32) << 1) ^ ((v >> 31) as u32)
}

#[inline]
fn encode_zigzag64(v: i64) -> u64 {
    ((v as u64) << 1) ^ ((v >> 63) as u64)
}

#[inline]
pub fn sizeof_sint32(v: i32) -> usize {
    sizeof_varint(encode_zigzag32(v) as u64)
}

#[inline]
pub fn sizeof_sint64(v: i64) -> usize {
    sizeof_varint(encode_zigzag64(v))
}

/// Return the number of bytes required to store a variable-length unsigned
/// 64-bit integer in base-128 varint encoding
///
/// This function is deadly simple, and easy to understand, the performance
/// is good enough most cases.
#[inline]
pub fn sizeof_varint(v: u64) -> usize {
    match v {
        0x0..=0x7F => 1,
        0x80..=0x3FFF => 2,
        0x4000..=0x1FFFFF => 3,
        0x200000..=0xFFFFFFF => 4,
        0x10000000..=0x7FFFFFFFF => 5,
        0x0800000000..=0x3FFFFFFFFFF => 6,
        0x040000000000..=0x1FFFFFFFFFFFF => 7,
        0x02000000000000..=0xFFFFFFFFFFFFFF => 8,
        0x0100000000000000..=0x7FFFFFFFFFFFFFFF => 9,
        _ => 10,
    }
}

/// Computes the binary size of a variable length chunk of data (wire type 2)
///
/// The total size is the varint encoded length size plus the length itself
/// https://developers.google.com/protocol-buffers/docs/encoding
#[inline]
pub fn sizeof_len(len: usize) -> usize {
    sizeof_varint(len as u64) + len
}

pub struct Writer<'a> {
    pub buf: &'a mut [u8],
    pub pos: usize,
}

// inner methods
impl<'a> Writer<'a> {
    #[inline]
    pub fn write_length(&mut self, v: usize) -> Result<(), EncodeError> {
        self.write_varint(v as u64)
    }

    fn write_varint32(&mut self, mut v: u32) -> Result<(), EncodeError> {
        while v > 0x7f {
            if self.pos >= self.buf.len() {
                return Err(EncodeError::Eof);
            }
            self.buf[self.pos] = (v as u8) | 0x80;
            self.pos += 1;

            v >>= 7;
        }

        if self.pos >= self.buf.len() {
            return Err(EncodeError::Eof);
        }
        self.buf[self.pos] = v as u8;
        self.pos += 1;

        Ok(())
    }

    pub fn write_varint(&mut self, v: u64) -> Result<(), EncodeError> {
        let mut hi = (v >> 32) as u32;
        if hi == 0 {
            return self.write_varint32(v as u32);
        }

        // 4 for lo, 1 for hi
        let lo = v as u32;
        if self.buf.len() - self.pos < 5 {
            return Err(EncodeError::Eof);
        }
        self.buf[self.pos] = lo as u8 | 0x80;
        self.buf[self.pos + 1] = (lo >> 7) as u8 | 0x80;
        self.buf[self.pos + 2] = (lo >> 14) as u8 | 0x80;
        self.buf[self.pos + 3] = (lo >> 21) as u8 | 0x80;
        self.pos += 4;

        if hi < 8 {
            self.buf[self.pos] = (hi << 4) as u8 | (lo >> 28) as u8;
            self.pos += 1;
            return Ok(());
        }

        self.buf[self.pos] = ((hi & 7) << 4) as u8 | (lo >> 28) as u8 | 0x80;
        self.pos += 1;
        hi >>= 3;

        while hi >= 128 {
            if self.pos >= self.buf.len() {
                return Err(EncodeError::Eof);
            }

            self.buf[self.pos] = hi as u8 | 0x80;
            self.pos += 1;
            hi >>= 7;
        }

        if self.pos >= self.buf.len() {
            return Err(EncodeError::Eof);
        }
        self.buf[self.pos] = hi as u8;
        self.pos += 1;

        Ok(())
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn write_varint32_unchecked(&mut self, mut v: u32) {
        while v > 0x7f {
            *self.buf.get_unchecked_mut(self.pos) = (v as u8) | 0x80;

            self.pos += 1;
            v >>= 7;
        }

        *self.buf.get_unchecked_mut(self.pos) = v as u8;
        self.pos += 1;
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn write_varint64_unchecked(&mut self, v: u64) {
        let mut hi = (v >> 32) as u32;
        let lo = v as u32;
        if hi == 0 {
            self.write_varint32_unchecked(lo);
            return;
        }

        *self.buf.get_unchecked_mut(self.pos) = lo as u8 | 0x80;
        *self.buf.get_unchecked_mut(self.pos + 1) = (lo >> 7) as u8 | 0x80;
        *self.buf.get_unchecked_mut(self.pos + 2) = (lo >> 14) as u8 | 0x80;
        *self.buf.get_unchecked_mut(self.pos + 3) = (lo >> 21) as u8 | 0x80;
        self.pos += 4;

        if hi < 8 {
            *self.buf.get_unchecked_mut(self.pos) = (hi << 4) as u8 | (lo >> 28) as u8;
            self.pos += 1;
            return;
        }

        *self.buf.get_unchecked_mut(self.pos) = ((hi & 7) << 4) as u8 | (lo >> 28) as u8 | 0x80;
        self.pos += 1;
        hi >>= 3;

        while hi >= 128 {
            *self.buf.get_unchecked_mut(self.pos) = hi as u8 | 0x80;
            self.pos += 1;
            hi >>= 7;
        }

        *self.buf.get_unchecked_mut(self.pos) = hi as u8;
        self.pos += 1;
    }

    #[inline]
    fn write_tag(&mut self, v: u32) -> Result<(), EncodeError> {
        self.write_varint32(v)
    }

    fn write(&mut self, v: &[u8]) -> Result<(), EncodeError> {
        if self.buf.len() - self.pos < v.len() {
            return Err(EncodeError::Eof);
        }

        unsafe {
            core::ptr::copy_nonoverlapping(
                v.as_ptr(),
                self.buf.as_mut_ptr().add(self.pos),
                v.len(),
            );
        }
        self.pos += v.len();

        Ok(())
    }
}

// write Protobuf types
impl<'a> Writer<'a> {
    #[inline]
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    #[inline]
    pub fn write_float(&mut self, tag: u32, v: f32) -> Result<(), EncodeError> {
        self.write_tag(tag)?;
        self.write(v.to_le_bytes().as_ref())
    }

    #[inline]
    pub fn write_double(&mut self, tag: u32, v: f64) -> Result<(), EncodeError> {
        self.write_tag(tag)?;
        self.write(v.to_le_bytes().as_ref())
    }

    pub fn write_int32(&mut self, tag: u32, v: i32) -> Result<(), EncodeError> {
        self.write_tag(tag)?;

        if v >= 0 {
            return self.write_varint32(v as u32);
        }

        if self.buf.len() - self.pos < 10 {
            return Err(EncodeError::Eof);
        }

        self.buf[self.pos] = v as u8 | 0x80;
        self.buf[self.pos + 1] = (v >> 7) as u8 | 0x80;
        self.buf[self.pos + 2] = (v >> 14) as u8 | 0x80;
        self.buf[self.pos + 3] = (v >> 21) as u8 | 0x80;
        self.buf[self.pos + 4] = (v >> 28) as u8 | 0x80;

        unsafe {
            std::ptr::copy_nonoverlapping(
                [0xff, 0xff, 0xff, 0xff, 0x01].as_ptr(),
                self.buf.as_mut_ptr().add(self.pos + 5),
                5,
            )
        }

        self.pos += 10;

        Ok(())
    }

    #[inline]
    pub fn write_int64(&mut self, tag: u32, v: i64) -> Result<(), EncodeError> {
        self.write_tag(tag)?;
        self.write_varint(v as u64)
    }

    #[inline]
    pub fn write_uint32(&mut self, tag: u32, v: u32) -> Result<(), EncodeError> {
        self.write_tag(tag)?;
        self.write_varint32(v)
    }

    #[inline]
    pub fn write_uint64(&mut self, tag: u32, v: u64) -> Result<(), EncodeError> {
        self.write_tag(tag)?;
        self.write_varint(v)
    }

    #[inline]
    pub fn write_sint32(&mut self, tag: u32, v: i32) -> Result<(), EncodeError> {
        self.write_tag(tag)?;
        self.write_varint32(encode_zigzag32(v))
    }

    #[inline]
    pub fn write_sint64(&mut self, tag: u32, v: i64) -> Result<(), EncodeError> {
        self.write_tag(tag)?;
        self.write_varint(encode_zigzag64(v))
    }

    #[inline]
    pub fn write_fixed32(&mut self, tag: u32, v: u32) -> Result<(), EncodeError> {
        self.write_tag(tag)?;
        self.write(v.to_le_bytes().as_ref())
    }

    #[inline]
    pub fn write_fixed64(&mut self, tag: u32, v: u64) -> Result<(), EncodeError> {
        self.write_tag(tag)?;
        self.write(v.to_le_bytes().as_ref())
    }

    #[inline]
    pub fn write_sfixed32(&mut self, tag: u32, v: i32) -> Result<(), EncodeError> {
        self.write_tag(tag)?;
        self.write((v as u32).to_le_bytes().as_ref())
    }

    #[inline]
    pub fn write_sfixed64(&mut self, tag: u32, v: i64) -> Result<(), EncodeError> {
        self.write_tag(tag)?;
        self.write((v as u64).to_le_bytes().as_ref())
    }

    pub fn write_bool(&mut self, tag: u32, v: bool) -> Result<(), EncodeError> {
        self.write_tag(tag)?;

        if self.pos >= self.buf.len() {
            return Err(EncodeError::Eof);
        }

        self.buf[self.pos] = v as u8;
        self.pos += 1;

        Ok(())
    }

    #[inline]
    pub fn write_bytes(&mut self, tag: u32, v: &[u8]) -> Result<(), EncodeError> {
        self.write_tag(tag)?;
        self.write_varint(v.len() as u64)?;
        self.write(v)
    }

    #[inline]
    pub fn write_string(&mut self, tag: u32, v: &str) -> Result<(), EncodeError> {
        let bytes = v.as_bytes();
        self.write_bytes(tag, bytes)
    }

    pub fn write_msg<T: Serialize>(&mut self, tag: u32, v: &T) -> Result<(), EncodeError> {
        self.write_tag(tag)?;

        let len = v.encoded_len();
        self.write_varint(len as u64)?;

        // write elements
        if self.buf.len() - self.pos < len {
            return Err(EncodeError::Eof);
        }

        self.pos += v.encode(&mut self.buf[self.pos..self.pos + len])?;

        Ok(())
    }
}

// packed
impl<'a> Writer<'a> {
    pub fn write_packed<T>(&mut self, tag: u32, array: &[T]) -> Result<(), EncodeError> {
        if array.is_empty() {
            return Ok(());
        }

        // write tag
        self.write_tag(tag)?;

        // write length delimiter
        let len = array.len() * size_of::<T>();
        self.write_varint32(len as u32)?;

        if self.buf.len() - self.pos < len {
            return Err(EncodeError::Eof);
        }

        // write elements
        //
        // This is fine for most common platform(aka. little-endian),
        // e.g. x86, Apple Silicon, ARM, AArch64
        unsafe {
            std::ptr::copy_nonoverlapping(
                array.as_ptr() as *const u8,
                self.buf.as_mut_ptr().add(self.pos),
                len,
            )
        };
        self.pos += len;

        Ok(())
    }

    pub fn write_packed_int32(&mut self, tag: u32, array: &[i32]) -> Result<(), EncodeError> {
        if array.is_empty() {
            return Ok(());
        }

        self.write_tag(tag)?;

        let len = array.iter().map(|v| sizeof_int32(*v)).sum::<usize>();
        self.write_varint32(len as u32)?;

        // write elements
        if self.buf.len() - self.pos < len {
            return Err(EncodeError::Eof);
        }

        for v in array {
            let v = *v;

            unsafe {
                if v >= 0 {
                    self.write_varint32_unchecked(v as u32)
                } else {
                    self.write_varint64_unchecked(v as i64 as u64)
                }
            }
        }

        Ok(())
    }

    pub fn write_packed_int64(&mut self, tag: u32, array: &[i64]) -> Result<(), EncodeError> {
        if array.is_empty() {
            return Ok(());
        }

        self.write_tag(tag)?;

        let len = array
            .iter()
            .map(|v| sizeof_varint(*v as u64))
            .sum::<usize>();
        self.write_varint32(len as u32)?;

        // write elements
        if self.buf.len() - self.pos < len {
            return Err(EncodeError::Eof);
        }

        for v in array {
            unsafe { self.write_varint64_unchecked(*v as u64) }
        }

        Ok(())
    }

    pub fn write_packed_uint32(&mut self, tag: u32, array: &[u32]) -> Result<(), EncodeError> {
        if array.is_empty() {
            return Ok(());
        }

        self.write_tag(tag)?;

        let len = array
            .iter()
            .map(|v| sizeof_varint(*v as u64))
            .sum::<usize>();
        self.write_varint32(len as u32)?;

        // write elements
        if self.buf.len() - self.pos < len {
            return Err(EncodeError::Eof);
        }

        for v in array {
            unsafe { self.write_varint32_unchecked(*v) }
        }

        Ok(())
    }

    pub fn write_packed_uint64(&mut self, tag: u32, array: &[u64]) -> Result<(), EncodeError> {
        if array.is_empty() {
            return Ok(());
        }

        self.write_tag(tag)?;

        let len = array.iter().map(|v| sizeof_varint(*v)).sum::<usize>();
        self.write_varint32(len as u32)?;

        // write elements
        if self.buf.len() - self.pos < len {
            return Err(EncodeError::Eof);
        }

        for v in array {
            unsafe { self.write_varint64_unchecked(*v) };
        }

        Ok(())
    }

    pub fn write_packed_sint32(&mut self, tag: u32, array: &[i32]) -> Result<(), EncodeError> {
        if array.is_empty() {
            return Ok(());
        }

        self.write_tag(tag)?;

        let len = array
            .iter()
            .map(|v| sizeof_varint(encode_zigzag32(*v) as u64))
            .sum::<usize>();
        self.write_varint32(len as u32)?;

        // write elements
        if self.buf.len() - self.pos < len {
            return Err(EncodeError::Eof);
        }

        for v in array {
            unsafe {
                self.write_varint32_unchecked(encode_zigzag32(*v));
            };
        }

        Ok(())
    }

    pub fn write_packed_sint64(&mut self, tag: u32, array: &[i64]) -> Result<(), EncodeError> {
        if array.is_empty() {
            return Ok(());
        }

        self.write_tag(tag)?;

        let len = array
            .iter()
            .map(|v| sizeof_varint(encode_zigzag64(*v)))
            .sum::<usize>();
        self.write_varint32(len as u32)?;

        // write elements
        if self.buf.len() - self.pos < len {
            return Err(EncodeError::Eof);
        }

        for v in array {
            unsafe {
                self.write_varint64_unchecked(encode_zigzag64(*v));
            }
        }

        Ok(())
    }

    pub fn write_packed_enum<T: Copy + Into<i32>>(
        &mut self,
        tag: u32,
        array: &[T],
    ) -> Result<(), EncodeError> {
        if array.is_empty() {
            return Ok(());
        }

        self.write_tag(tag)?;

        let len = array
            .iter()
            .map(|v| sizeof_int32((*v).into()))
            .sum::<usize>();
        self.write_varint32(len as u32)?;

        // write elements
        if self.buf.len() - self.pos < len {
            return Err(EncodeError::Eof);
        }

        for v in array {
            let v = (*v).into();

            unsafe {
                if v >= 0 {
                    self.write_varint32_unchecked(v as u32);
                } else {
                    self.write_varint64_unchecked(v as i64 as u64);
                }
            }
        }

        Ok(())
    }
}

pub struct Reader<'a> {
    pub src: &'a [u8],
    pub pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        Self { src, pos: 0 }
    }

    #[inline]
    pub fn read_varint(&mut self) -> Result<u64, DecodeError> {
        let len = (self.src.len() - self.pos).min(10);
        if len == 0 {
            return Err(DecodeError::Eof);
        }

        let b = self.src[self.pos] as u64;
        self.pos += 1;
        if b & 0x80 == 0 {
            return Ok(b);
        }

        let mut v = b & 0x7f;
        for i in 1..len {
            let b = self.src[self.pos] as u64;
            self.pos += 1;

            // no need to so strict
            //
            // if i == 9 && (b & 0x7e) != 0 {
            //     return Err(DecodeError::Varint);
            // }

            v |= (b & 0x7f) << (i * 7);
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }

        if len == 10 {
            Err(DecodeError::Varint)
        } else {
            Err(DecodeError::Eof)
        }
    }

    pub fn read_double(&mut self) -> Result<f64, DecodeError> {
        if self.src.len() - self.pos < 8 {
            return Err(DecodeError::Eof);
        }

        let value = unsafe {
            self.src
                .as_ptr()
                .add(self.pos)
                .cast::<f64>()
                .read_unaligned()
        };
        self.pos += 8;

        Ok(value)
    }
    pub fn read_float(&mut self) -> Result<f32, DecodeError> {
        if self.src.len() - self.pos < 4 {
            return Err(DecodeError::Eof);
        }

        let value = unsafe {
            self.src
                .as_ptr()
                .add(self.pos)
                .cast::<f32>()
                .read_unaligned()
        };
        self.pos += 4;

        Ok(value)
    }
    pub fn read_int32(&mut self) -> Result<i32, DecodeError> {
        self.read_varint().map(|v| v as i32)
    }
    pub fn read_int64(&mut self) -> Result<i64, DecodeError> {
        self.read_varint().map(|v| v as i64)
    }
    pub fn read_uint32(&mut self) -> Result<u32, DecodeError> {
        self.read_varint().map(|v| v as u32)
    }
    pub fn read_uint64(&mut self) -> Result<u64, DecodeError> {
        self.read_varint()
    }
    pub fn read_sint32(&mut self) -> Result<i32, DecodeError> {
        let v = self.read_varint()? as u32;
        Ok(((v >> 1) as i32) ^ (-((v & 1) as i32)))
    }
    pub fn read_sint64(&mut self) -> Result<i64, DecodeError> {
        let v = self.read_varint()?;
        Ok(((v >> 1) as i64) ^ (-((v & 1) as i64)))
    }
    pub fn read_fixed32(&mut self) -> Result<u32, DecodeError> {
        if self.src.len() - self.pos < 4 {
            return Err(DecodeError::Eof);
        }

        // fine for little-endian
        let value = unsafe {
            self.src
                .as_ptr()
                .add(self.pos)
                .cast::<u32>()
                .read_unaligned()
        };
        self.pos += 4;

        Ok(value)
    }
    pub fn read_fixed64(&mut self) -> Result<u64, DecodeError> {
        if self.src.len() - self.pos < 8 {
            return Err(DecodeError::Eof);
        }

        // fine for little-endian
        let value = unsafe {
            self.src
                .as_ptr()
                .add(self.pos)
                .cast::<u64>()
                .read_unaligned()
        };
        self.pos += 8;

        Ok(value)
    }
    pub fn read_sfixed32(&mut self) -> Result<i32, DecodeError> {
        if self.src.len() - self.pos < 4 {
            return Err(DecodeError::Eof);
        }

        // fine for little-endian
        let value = unsafe {
            self.src
                .as_ptr()
                .add(self.pos)
                .cast::<i32>()
                .read_unaligned()
        };
        self.pos += 4;

        Ok(value)
    }
    pub fn read_sfixed64(&mut self) -> Result<i64, DecodeError> {
        if self.src.len() - self.pos < 8 {
            return Err(DecodeError::Eof);
        }

        // fine for little-endian
        let value = unsafe {
            self.src
                .as_ptr()
                .add(self.pos)
                .cast::<i64>()
                .read_unaligned()
        };
        self.pos += 8;

        Ok(value)
    }
    pub fn read_bool(&mut self) -> Result<bool, DecodeError> {
        if self.pos >= self.src.len() {
            return Err(DecodeError::Eof);
        }

        let v = self.src[self.pos];
        self.pos += 1;

        Ok(v > 0)
    }
    pub fn read_string(&mut self) -> Result<String, DecodeError> {
        let len = self.read_varint()? as usize;
        if self.src.len() - self.pos < len {
            return Err(DecodeError::Eof);
        }

        match core::str::from_utf8(&self.src[self.pos..self.pos + len]) {
            Ok(s) => {
                self.pos += len;
                Ok(s.to_string())
            }
            Err(_) => Err(DecodeError::Utf8),
        }
    }

    pub fn read_bytes(&mut self) -> Result<Vec<u8>, DecodeError> {
        let len = self.read_varint()? as usize;
        if len == 0 {
            return Ok(Vec::new());
        }

        if self.src.len() - self.pos < len {
            return Err(DecodeError::Eof);
        }

        let data = self.src[self.pos..self.pos + len].to_vec();
        self.pos += len;

        Ok(data)
    }

    #[inline]
    pub fn read_enum<E: TryFrom<i32, Error = DecodeError>>(&mut self) -> Result<E, DecodeError> {
        E::try_from(self.read_int32()?)
    }
    pub fn read_msg<D: Deserialize>(&mut self) -> Result<D, DecodeError> {
        let len = self.read_varint()? as usize;
        if self.src.len() - self.pos < len {
            return Err(DecodeError::Eof);
        }

        let msg = D::decode(&self.src[self.pos..self.pos + len])?;
        self.pos += len;

        Ok(msg)
    }
    pub fn read_packed<T, R>(&mut self, mut read: R) -> Result<Vec<T>, DecodeError>
    where
        R: FnMut(&mut Self) -> Result<T, DecodeError>,
    {
        let len = self.read_varint()? as usize;
        if self.src.len() - self.pos < len {
            return Err(DecodeError::Eof);
        }

        // This capacity is just a guess, which is trying to reduce alloc
        // not avoid realloc.
        //
        // NOTE: protobuf only allow scalar type, whose size is always > 0
        let mut array = Vec::with_capacity(len / size_of::<T>());

        let end = self.pos + len;
        while self.pos < end {
            array.push(read(self)?);
        }

        if self.pos != end {
            Err(DecodeError::Malformed)
        } else {
            Ok(array)
        }
    }

    // NOTE:　bool is handled here, Protobuf only handle 'byte's not 'bit's,
    //   so, we might receive some wire bytes like [0, 1, 2, 4].
    pub fn read_packed_fixed<T>(&mut self) -> Result<Vec<T>, DecodeError> {
        let len = self.read_varint()? as usize;
        if len % size_of::<T>() != 0 {
            return Err(DecodeError::Malformed);
        }

        if self.src.len() - self.pos < len {
            return Err(DecodeError::Eof);
        }

        let mut array = Vec::<T>::with_capacity(len / size_of::<T>());
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.src.as_ptr().add(self.pos),
                array.as_mut_ptr() as *mut u8,
                len,
            );
            array.set_len(len / size_of::<T>());
        }
        self.pos += len;

        Ok(array)
    }

    pub fn read_key_value<K, V, KF, VF>(
        &mut self,
        mut read_key: KF,
        mut read_value: VF,
    ) -> Result<(K, V), DecodeError>
    where
        K: Default,
        V: Default,
        KF: FnMut(&mut Self) -> Result<K, DecodeError>,
        VF: FnMut(&mut Self) -> Result<V, DecodeError>,
    {
        // kind of lossy for other language implement
        let len = self.read_varint()? as usize;
        let end = self.pos.saturating_add(len).min(self.src.len());

        let mut key = Default::default();
        let mut value = Default::default();
        while self.pos < end {
            let num = self.src[self.pos];
            self.pos += 1;

            match num >> 3 {
                1 => key = read_key(self)?,
                2 => value = read_value(self)?,
                _ => return Err(DecodeError::Varint),
            }
        }

        Ok((key, value))
    }

    pub fn read_unknown(&mut self, tag: u32) -> Result<(), DecodeError> {
        let offset = match (tag & 0x7) as u8 {
            // WireType::Varint
            0 => {
                self.read_varint()?;
                return Ok(());
            }
            // WireType::Fixed64
            1 => 8,
            // WireType::Fixed32
            5 => 4,
            // WireType::LengthDelimited
            2 => self.read_varint()? as usize,
            // WireType::StartGroup | WireType::EndGroup
            3 | 4 => {
                return Err(DecodeError::Deprecated("group"));
            }
            wire_type => return Err(DecodeError::WireType(wire_type)),
        };

        // meant to prevent overflowing. comparison used is *strictly* lesser,
        // since `self.src.len()` is given by `len()`; i.e. `self.src.len()`
        // is 1 more than highest index
        if self.src.len() - self.pos < offset {
            Err(DecodeError::Varint)
        } else {
            self.pos += offset;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint() {
        let data = [0x96, 0x01];
        let mut reader = Reader::new(&data[..]);
        let got = reader.read_varint().unwrap();
        assert_eq!(got, 150);
        assert_eq!(reader.pos, 2);
    }

    #[test]
    fn read_size_overflowing_unknown() {
        let data = &[
            200, 250, 35, // varint tag with WIRE_TYPE_VARINT -- 589128
            //
            //
            47, // varint itself
            //
            //
            250, 36, // varint tag with WIRE_TYPE_LENGTH_DELIMITED -- 4730
            //
            //
            255, 255, 255, 255, 255, 255, 255, 255, 255, 3, // huge 10-byte length
            //
            //
            255, 255, 227, // unused extra bytes
        ];

        let mut reader = Reader::new(data);

        let tag = reader.read_uint32().unwrap();
        assert_eq!(tag, 589128);
        reader.read_unknown(tag).unwrap();

        assert_ne!(reader.pos, reader.src.len());
        let tag = reader.read_uint32().unwrap();
        assert_eq!(tag, 4730);
        let err = reader.read_unknown(tag).unwrap_err();
        assert_eq!(err, DecodeError::Varint);
    }
}
