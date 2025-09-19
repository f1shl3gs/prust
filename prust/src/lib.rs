mod encoding;

pub use encoding::*;

#[cfg(feature = "tonic")]
pub mod tonic_codec {
    use std::marker::PhantomData;

    use bytes::{Buf, BufMut};
    use tonic::Status;
    use tonic::codec::{DecodeBuf, EncodeBuf};

    use super::*;

    #[derive(Debug)]
    pub struct Codec<T, U> {
        _phantom: PhantomData<(T, U)>,
    }

    impl<T, U> Default for Codec<T, U> {
        fn default() -> Self {
            Self {
                _phantom: PhantomData,
            }
        }
    }

    pub struct Encoder<T> {
        _pd: PhantomData<T>,
    }

    impl<T: Serialize> ::tonic::codec::Encoder for Encoder<T> {
        type Item = T;
        type Error = Status;

        fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
            let required = item.encoded_len();
            dst.reserve(required);

            let buf = unsafe { std::slice::from_raw_parts_mut(dst.chunk_mut().as_mut_ptr(), required) };
            let written = item
                .encode(buf)
                .map_err(|err| Status::internal(err.to_string()))?;
            unsafe {
                dst.advance_mut(written);
            }

            Ok(())
        }
    }

    pub struct Decoder<T> {
        _pd: PhantomData<T>,
    }
    impl<T: Deserialize> ::tonic::codec::Decoder for Decoder<T> {
        type Item = T;
        type Error = Status;

        fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
            let len = src.chunk().len();
            let item =
                Deserialize::decode(src.chunk()).map_err(|err| Status::internal(err.to_string()))?;
            src.advance(len);

            Ok(Some(item))
        }
    }

    impl<T, U> ::tonic::codec::Codec for Codec<T, U>
    where
        T: Serialize + Send + 'static,
        U: Deserialize + Send + 'static,
    {
        type Encode = T;
        type Decode = U;

        type Encoder = Encoder<T>;
        type Decoder = Decoder<U>;

        fn encoder(&mut self) -> Self::Encoder {
            Encoder { _pd: PhantomData }
        }

        fn decoder(&mut self) -> Self::Decoder {
            Decoder { _pd: PhantomData }
        }
    }
}
