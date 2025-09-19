#[derive(Debug, PartialEq)]
pub enum DecodeError {
    // Unexpected EOF, buffer is too short
    UnexpectedEof,
    // Invalid varint.
    Varint,
    // Unknown WireType
    WireType(u8),
    Deprecated(&'static str),
    UnknownEnumValue(&'static str, i32),
    Utf8,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::UnexpectedEof => f.write_str("unexpected EOF"),
            DecodeError::Varint => f.write_str("invalid varint"),
            DecodeError::WireType(typ) => write!(f, "unknown wire type: {}", typ),
            DecodeError::Deprecated(typ) => write!(f, "deprecated \"{}\" is not supported", typ),
            DecodeError::UnknownEnumValue(typ, value) => {
                write!(f, "unknown enum value {typ}: {value}")
            }
            DecodeError::Utf8 => f.write_str("invalid UTF-8"),
        }
    }
}

pub trait Deserialize: Sized {
    fn decode(buf: &[u8]) -> Result<Self, DecodeError>;
}

/// EncodeError returned when encoding
#[derive(Debug)]
pub enum EncodeError {
    // Unexpected EOF, buffer is too short
    UnexpectedEof,
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodeError::UnexpectedEof => f.write_str("unexpected EOF"),
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
pub fn sizeof_sint32(v: i32) -> usize {
    sizeof_varint(((v << 1) ^ (v >> 31)) as u32 as u64)
}

#[inline]
pub fn sizeof_sint64(v: i64) -> usize {
    sizeof_varint(((v << 1) ^ (v >> 63)) as u64)
}

/// Return the number of bytes required to store a variable-length unsigned
/// 64-bit integer in base-128 varint encoding
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

impl<'a> Writer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    #[inline]
    pub fn write_length(&mut self, v: usize) -> Result<(), EncodeError> {
        self.write_varint(v as u64)
    }
    pub fn write_varint(&mut self, v: u64) -> Result<(), EncodeError> {
        let mut hi = (v >> 32) as u32;
        if hi == 0 {
            return self.write_uint32(v as u32);
        }

        // 4 for lo, 1 for hi
        let lo = v as u32;
        if self.pos + 5 > self.buf.len() {
            return Err(EncodeError::UnexpectedEof);
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
            if self.pos + 1 > self.buf.len() {
                return Err(EncodeError::UnexpectedEof);
            }

            self.buf[self.pos] = hi as u8 | 0x80;
            self.pos += 1;
            hi >>= 7;
        }

        if self.pos >= self.buf.len() {
            return Err(EncodeError::UnexpectedEof);
        }
        self.buf[self.pos] = hi as u8;
        self.pos += 1;

        Ok(())
    }
    pub fn write_raw_bytes(&mut self, v: &[u8]) -> Result<(), EncodeError> {
        if self.pos + v.len() > self.buf.len() {
            return Err(EncodeError::UnexpectedEof);
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

    pub fn write_double(&mut self, v: f64) -> Result<(), EncodeError> {
        self.write_raw_bytes(v.to_le_bytes().as_ref())
    }
    pub fn write_float(&mut self, v: f32) -> Result<(), EncodeError> {
        self.write_raw_bytes(v.to_le_bytes().as_ref())
    }
    pub fn write_int32(&mut self, v: i32) -> Result<(), EncodeError> {
        if v >= 0 {
            return self.write_uint32(v as u32);
        }

        if self.pos + 10 > self.buf.len() {
            return Err(EncodeError::UnexpectedEof);
        }

        let v = v as u32;
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
    pub fn write_int64(&mut self, v: i64) -> Result<(), EncodeError> {
        self.write_varint(v as u64)
    }
    pub fn write_uint32(&mut self, mut v: u32) -> Result<(), EncodeError> {
        while v > 0x7f {
            if self.pos + 1 > self.buf.len() {
                return Err(EncodeError::UnexpectedEof);
            }
            self.buf[self.pos] = (v as u8 & 0x7f) | 0x80;
            self.pos += 1;

            v >>= 7;
        }

        if self.pos + 1 > self.buf.len() {
            return Err(EncodeError::UnexpectedEof);
        }
        self.buf[self.pos] = v as u8;
        self.pos += 1;

        Ok(())
    }
    pub fn write_uint64(&mut self, v: u64) -> Result<(), EncodeError> {
        self.write_varint(v)
    }
    pub fn write_sint32(&mut self, v: i32) -> Result<(), EncodeError> {
        self.write_varint(((v << 1) ^ (v >> 31)) as u32 as u64)
    }
    pub fn write_sint64(&mut self, v: i64) -> Result<(), EncodeError> {
        self.write_varint(((v << 1) ^ (v >> 63)) as u64)
    }
    pub fn write_fixed32(&mut self, v: u32) -> Result<(), EncodeError> {
        self.write_raw_bytes(v.to_le_bytes().as_ref())
    }
    pub fn write_fixed64(&mut self, v: u64) -> Result<(), EncodeError> {
        self.write_raw_bytes(v.to_le_bytes().as_ref())
    }
    pub fn write_sfixed32(&mut self, v: i32) -> Result<(), EncodeError> {
        self.write_raw_bytes((v as u32).to_le_bytes().as_ref())
    }
    pub fn write_sfixed64(&mut self, v: i64) -> Result<(), EncodeError> {
        self.write_raw_bytes((v as u64).to_le_bytes().as_ref())
    }
    pub fn write_bool(&mut self, v: bool) -> Result<(), EncodeError> {
        if self.pos + 1 > self.buf.len() {
            return Err(EncodeError::UnexpectedEof);
        }

        self.buf[self.pos] = v as u8;
        self.pos += 1;

        Ok(())
    }
    pub fn write_string(&mut self, v: &str) -> Result<(), EncodeError> {
        self.write_bytes(v.as_bytes())
    }
    pub fn write_bytes(&mut self, v: &[u8]) -> Result<(), EncodeError> {
        self.write_uint32(v.len() as u32)?;
        self.write_raw_bytes(v)
    }
}

impl<'a> Writer<'a> {
    pub fn write<T, W>(&mut self, tag: u32, v: T, mut write: W) -> Result<(), EncodeError>
    where
        W: FnMut(&mut Self, T) -> Result<(), EncodeError>,
    {
        self.write_uint32(tag)?;
        write(self, v)
    }
    pub fn write_msg<T: Serialize>(&mut self, v: &T) -> Result<(), EncodeError> {
        let len = v.encoded_len();
        self.write_varint(len as u64)?;

        if self.pos + len > self.buf.len() {
            return Err(EncodeError::UnexpectedEof);
        }

        self.pos += v.encode(&mut self.buf[self.pos..self.pos + len])?;

        Ok(())
    }
    pub fn write_packed<T>(
        &mut self,
        tag: u32,
        array: &[T],
        sizeof: impl Fn(&T) -> usize,
        mut write: impl FnMut(&mut Self, &T) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError> {
        if array.is_empty() {
            return Ok(());
        }

        // write tag
        self.write_uint32(tag)?;

        // write length delimiter
        let size = array.iter().map(sizeof).sum::<usize>();
        self.write_uint32(size as u32)?;

        // write elements
        for v in array {
            write(self, v)?;
        }

        Ok(())
    }
    pub fn write_packed_fixed<T>(&mut self, tag: u32, array: &[T]) -> Result<(), EncodeError> {
        if array.is_empty() {
            return Ok(());
        }

        // write tag
        self.write_uint32(tag)?;

        // write length delimiter
        let len = array.len() * size_of::<T>();
        self.write_uint32(len as u32)?;

        // write elements
        if self.pos + len > self.buf.len() {
            return Err(EncodeError::UnexpectedEof);
        }

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
}

pub struct Reader<'a> {
    pub src: &'a [u8],
    pub pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        Self { src, pos: 0 }
    }
    pub fn read_varint(&mut self) -> Result<u64, DecodeError> {
        let len = (self.src.len() - self.pos).min(10);
        if len == 0 {
            return Err(DecodeError::UnexpectedEof);
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

            v |= (b & 0x7f) << (i * 7);
            if b & 0x80 == 0 {
                break;
            }
        }

        Ok(v)
    }

    pub fn read_double(&mut self) -> Result<f64, DecodeError> {
        if self.pos + 8 > self.src.len() {
            return Err(DecodeError::UnexpectedEof);
        }

        let value = unsafe {
            self.src
                .as_ptr()
                .add(self.pos)
                .cast::<f64>()
                .read_unaligned()
        };
        // let value = f64::from_bits(raw);
        // let value = f64::from_le_bytes((&self.src[self.pos..self.pos + 8]).try_into().unwrap());
        self.pos += 8;

        Ok(value)
    }
    pub fn read_float(&mut self) -> Result<f32, DecodeError> {
        if self.pos + 4 > self.src.len() {
            return Err(DecodeError::UnexpectedEof);
        }

        let value = unsafe {
            self.src
                .as_ptr()
                .add(self.pos)
                .cast::<f32>()
                .read_unaligned()
        };
        // let value = f32::from_bits(raw);
        // let value = f32::from_le_bytes((&self.src[self.pos..self.pos + 4]).try_into().unwrap());
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
        if self.pos + 4 > self.src.len() {
            return Err(DecodeError::UnexpectedEof);
        }

        let value = u32::from_le_bytes((&self.src[self.pos..self.pos + 4]).try_into().unwrap());
        self.pos += 4;

        Ok(value)
    }
    pub fn read_fixed64(&mut self) -> Result<u64, DecodeError> {
        if self.pos + 8 > self.src.len() {
            return Err(DecodeError::UnexpectedEof);
        }

        let value = u64::from_le_bytes((&self.src[self.pos..self.pos + 8]).try_into().unwrap());
        self.pos += 8;

        Ok(value)
    }
    pub fn read_sfixed32(&mut self) -> Result<i32, DecodeError> {
        if self.pos + 4 > self.src.len() {
            return Err(DecodeError::UnexpectedEof);
        }

        let value = i32::from_le_bytes((&self.src[self.pos..self.pos + 4]).try_into().unwrap());
        self.pos += 4;

        Ok(value)
    }
    pub fn read_sfixed64(&mut self) -> Result<i64, DecodeError> {
        if self.pos + 8 > self.src.len() {
            return Err(DecodeError::UnexpectedEof);
        }

        let value = i64::from_le_bytes((&self.src[self.pos..self.pos + 8]).try_into().unwrap());
        self.pos += 8;

        Ok(value)
    }
    pub fn read_bool(&mut self) -> Result<bool, DecodeError> {
        if self.pos >= self.src.len() {
            return Err(DecodeError::UnexpectedEof);
        }

        let v = self.src[self.pos];
        self.pos += 1;

        Ok(v > 0)
    }
    pub fn read_string(&mut self) -> Result<String, DecodeError> {
        let len = self.read_varint()? as usize;
        if self.pos + len > self.src.len() {
            return Err(DecodeError::UnexpectedEof);
        }

        match core::str::from_utf8(&self.src[self.pos..self.pos + len]) {
            Ok(s) => {
                self.pos += len;
                Ok(s.to_string())
            }
            Err(_) => Err(DecodeError::UnexpectedEof),
        }
    }
    pub fn read_bytes(&mut self) -> Result<Vec<u8>, DecodeError> {
        let len = self.read_varint()? as usize;
        if self.pos + len > self.src.len() {
            return Err(DecodeError::UnexpectedEof);
        }

        let data = unsafe {
            let layout = std::alloc::Layout::array::<u8>(len).unwrap();
            let ptr = std::alloc::alloc(layout);

            std::ptr::copy_nonoverlapping(self.src.as_ptr().add(self.pos), ptr, len);

            Vec::from_raw_parts(ptr, len, len)
        };
        self.pos += len;

        Ok(data)
    }

    #[inline]
    pub fn read_enum<E: TryFrom<i32, Error = DecodeError>>(&mut self) -> Result<E, DecodeError> {
        E::try_from(self.read_int32()?)
    }
    pub fn read_msg<D: Deserialize>(&mut self) -> Result<D, DecodeError> {
        let len = self.read_varint()? as usize;
        if self.pos + len > self.src.len() {
            return Err(DecodeError::UnexpectedEof);
        }

        let msg = D::decode(&self.src[self.pos..self.pos + len])?;
        self.pos += len;

        Ok(msg)
    }
    pub fn read_map<K: Default + Ord, V: Default>(
        &mut self,
        dst: &mut std::collections::BTreeMap<K, V>,
        read_key: impl Fn(&mut Self) -> Result<K, DecodeError>,
        read_value: impl Fn(&mut Self) -> Result<V, DecodeError>,
    ) -> Result<(), DecodeError> {
        let len = self.read_varint()? as usize;
        let end = std::cmp::min(self.pos + len, self.src.len());

        let mut key: K = Default::default();
        let mut value: V = Default::default();
        while self.pos < end {
            // read_variant32 should be called, but max tag is 1 << 3 | 5
            // is less than 127, u8 is fine here. and the size is checked
            // by the `while` condition, so
            let num = self.src[self.pos] >> 3;
            self.pos += 1;

            match num {
                1 => key = read_key(self)?,
                2 => value = read_value(self)?,
                _ => return Err(DecodeError::Varint),
            }
        }

        dst.insert(key, value);

        Ok(())
    }
    pub fn read_packed<T>(
        &mut self,
        mut read: impl FnMut(&mut Reader) -> Result<T, DecodeError>,
    ) -> Result<Vec<T>, DecodeError> {
        let len = self.read_varint()? as usize;
        if self.pos + len > self.src.len() {
            return Err(DecodeError::UnexpectedEof);
        }

        // This capacity is just a nonsense guess, and trying to reduce allocations
        let mut array = Vec::with_capacity(len / size_of::<T>());
        let mut buf = Reader {
            src: &self.src[self.pos..self.pos + len],
            pos: 0,
        };
        while buf.pos < buf.src.len() {
            let v = read(&mut buf)?;
            array.push(v);
        }
        self.pos += len;

        Ok(array)
    }
    pub fn read_packed_fixed<T: Clone>(&mut self) -> Result<Vec<T>, DecodeError> {
        let len = self.read_varint()? as usize;
        if self.pos + len > self.src.len() {
            return Err(DecodeError::UnexpectedEof);
        }

        let mut array = Vec::<T>::with_capacity(len / size_of::<T>());
        unsafe {
            core::ptr::copy(
                self.src.as_ptr().add(self.pos),
                array.as_mut_ptr() as *mut u8,
                len,
            );
            array.set_len(len / size_of::<T>());
        }
        self.pos += len;

        Ok(array)
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

        if self.pos + offset > self.src.len() {
            Err(DecodeError::UnexpectedEof)
        } else {
            self.pos += offset;
            Ok(())
        }
    }
}
