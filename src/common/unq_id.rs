use askama::FastWritable;
use core::str;
use serde::Serialize;
use std::{fmt::Display, hash::Hash, u64};

// unique id salt
const UNQ_ID_SALT: u64 = 10393820183;

pub type UidBuf = [u8; 10];
#[derive(Clone)]
pub struct UnqID {
    id: u64,
    buf: UidBuf,
}
impl Serialize for UnqID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.uid_str())
    }
}

impl<'a> Display for UnqID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res: &str = self.into();
        write!(f, "{}", res)
    }
}

impl<'a> FastWritable for UnqID {
    fn write_into<W: core::fmt::Write + ?Sized>(
        &self,
        dest: &mut W,
        _values: &dyn askama::Values,
    ) -> askama::Result<()> {
        dest.write_str(self.into())?;
        Ok(())
    }
}

impl Hash for UnqID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.id);
    }
}

impl<'a> UnqID {
    pub const fn id(&self) -> u64 {
        return self.id;
    }

    pub const fn uid_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.buf) }
    }

    pub const fn new(value: u64) -> UnqID {
        let mut hash_id = value as u64;
        let mut id: UidBuf = [0; 10];
        id[0] = 'u' as u8;
        id[1] = '-' as u8;
        {
            let mut k = 0;
            while k < 8 {
                id[k + 2] = ([
                    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
                ])[(hash_id % 16) as usize] as u8;
                hash_id /= 16;
                k += 1;
            }
        }
        {
            let mut s: usize = 2;
            let mut p: usize = 9;
            while s < p {
                let aux = id[s];
                id[s] = id[p];
                id[p] = aux;
                s += 1;
                p -= 1;
            }
        }

        UnqID { id: value, buf: id }
    }

    pub const fn new_str(s: &str) -> UnqID {
        let ss = s.as_bytes();
        let mut value: u64 = 0x100000001b3;
        value = value.overflowing_add(UNQ_ID_SALT).0;

        {
            let mut i = 0;
            while i < ss.len() {
                value = value.overflowing_mul(0x100000001b3).0 ^ ss[i] as u64;
                i += 1;
            }
        }
        Self::new(value)
    }
}

impl<'a> Into<&'a str> for &'a UnqID {
    fn into(self) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(&self.buf.as_ref()) }
    }
}

