use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Error, ErrorKind, Result, Write};

pub trait KeyEncoder<T = Self> {
    fn encode_key(&self, bytes: &mut [u8]) -> Result<usize>;
    fn key_len(&self) -> usize;
    fn decode_key(bytes: &[u8]) -> Result<T>;
}

macro_rules! num_key_encoder_impl {
    ($type: ty, $write: tt, $read: tt, $size: tt) => {
        impl KeyEncoder for $type {
            fn encode_key(&self, bytes: &mut [u8]) -> Result<usize> {
                if bytes.len() < $size {
                    return Err(Error::new(ErrorKind::UnexpectedEof, "Buffer too small."));
                }

                let mut c = std::io::Cursor::new(bytes);
                c.$write::<BigEndian>(*self)?;

                Ok($size)
            }

            fn key_len(&self) -> usize {
                $size
            }

            fn decode_key(bytes: &[u8]) -> Result<$type> {
                if bytes.len() < $size {
                    return Err(Error::new(ErrorKind::UnexpectedEof, "Buffer too small."));
                }

                let mut c = std::io::Cursor::new(bytes);
                c.$read::<BigEndian>()
            }
        }
    };
}

num_key_encoder_impl!(u16, write_u16, read_u16, 2);
num_key_encoder_impl!(i16, write_i16, read_i16, 2);
num_key_encoder_impl!(u32, write_u32, read_u32, 4);
num_key_encoder_impl!(i32, write_i32, read_i32, 4);
num_key_encoder_impl!(u64, write_u64, read_u64, 8);
num_key_encoder_impl!(i64, write_i64, read_i64, 8);

impl KeyEncoder for String {
    fn encode_key(&self, bytes: &mut [u8]) -> Result<usize> {
        if bytes.len() < self.len() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "Buffer too small."));
        }

        let mut c = std::io::Cursor::new(bytes);
        c.write_all(self.as_bytes())?;

        Ok(self.len())
    }

    fn key_len(&self) -> usize {
        self.len()
    }

    fn decode_key(bytes: &[u8]) -> Result<String> {
        String::from_utf8(bytes.to_vec()).map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }
}

// This allocates on every encode.  That's probably a bad idea.
pub fn encode_key<T>(key: &T) -> Vec<u8>
where
    T: KeyEncoder,
{
    let len = key.key_len();
    let mut data: Vec<u8> = vec![0; len];
    // TODO: propogate error.
    key.encode_key(&mut data).unwrap();

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u64_key() {
        let key: u64 = 0x01_23_45_67_89_ab_cd_ef;
        assert_eq!(
            encode_key(&key),
            vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
        );
    }

    #[test]
    fn i64_key() {
        let key: i64 = -2;
        assert_eq!(
            encode_key(&key),
            vec![0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe],
        );
    }

    #[test]
    fn u32_key() {
        let key: u32 = 0x01_23_45_67;
        assert_eq!(encode_key(&key), vec![0x01, 0x23, 0x45, 0x67],);
    }

    #[test]
    fn i32_key() {
        let key: i32 = -2;
        assert_eq!(encode_key(&key), vec![0xff, 0xff, 0xff, 0xfe],);
    }

    #[test]
    fn u16_key() {
        let key: u16 = 0x01_23;
        assert_eq!(encode_key(&key), vec![0x01, 0x23],);
    }

    #[test]
    fn i16_key() {
        let key: i16 = -2;
        assert_eq!(encode_key(&key), vec![0xff, 0xfe],);
    }

    #[test]
    fn str_key() {
        // TODO: test non ascii chars.
        let key = "hello world".to_string();
        assert_eq!(String::from_utf8(encode_key(&key)).unwrap(), key);
    }
}
