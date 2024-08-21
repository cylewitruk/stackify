use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Hash160(
    #[serde(
        serialize_with = "hash20_json_serialize",
        deserialize_with = "hash20_json_deserialize"
    )]
    pub [u8; 20],
);

fn hash20_json_serialize<S>(hash: &[u8; 20], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&hex::encode(hash))
}

fn hash20_json_deserialize<'de, D>(deserializer: D) -> Result<[u8; 20], D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
    if bytes.len() != 20 {
        return Err(serde::de::Error::custom("invalid length"));
    }
    let mut hash = [0u8; 20];
    hash.copy_from_slice(&bytes);
    Ok(hash)
}

impl std::ops::Index<usize> for Hash160 {
    type Output = u8;
    #[inline]
    fn index(&self, index: usize) -> &u8 {
        let &Hash160(ref dat) = self;
        &dat[index]
    }
}
impl std::ops::Index<::std::ops::Range<usize>> for Hash160 {
    type Output = [u8];
    #[inline]
    fn index(&self, index: ::std::ops::Range<usize>) -> &[u8] {
        &self.0[index]
    }
}
impl std::ops::Index<::std::ops::RangeTo<usize>> for Hash160 {
    type Output = [u8];
    #[inline]
    fn index(&self, index: ::std::ops::RangeTo<usize>) -> &[u8] {
        &self.0[index]
    }
}
impl std::ops::Index<::std::ops::RangeFrom<usize>> for Hash160 {
    type Output = [u8];
    #[inline]
    fn index(&self, index: ::std::ops::RangeFrom<usize>) -> &[u8] {
        &self.0[index]
    }
}
impl std::ops::Index<::std::ops::RangeFull> for Hash160 {
    type Output = [u8];
    #[inline]
    fn index(&self, _: ::std::ops::RangeFull) -> &[u8] {
        &self.0[..]
    }
}
impl PartialEq for Hash160 {
    #[inline]
    fn eq(&self, other: &Hash160) -> bool {
        &self[..] == &other[..]
    }
}
impl Eq for Hash160 {}

impl PartialOrd for Hash160 {
    #[inline]
    fn partial_cmp(&self, other: &Hash160) -> Option<::std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}
impl Ord for Hash160 {
    #[inline]
    fn cmp(&self, other: &Hash160) -> ::std::cmp::Ordering {
        for i in 0..20 {
            if self[20 - 1 - i] < other[20 - 1 - i] {
                return ::std::cmp::Ordering::Less;
            }
            if self[20 - 1 - i] > other[20 - 1 - i] {
                return ::std::cmp::Ordering::Greater;
            }
        }
        ::std::cmp::Ordering::Equal
    }
}
impl Clone for Hash160 {
    #[inline]
    fn clone(&self) -> Hash160 {
        *self
    }
}
impl Copy for Hash160 {}

impl<'a> From<&'a [u8]> for Hash160 {
    fn from(data: &'a [u8]) -> Hash160 {
        match (&(data.len()), &20) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    todo!("Implement correct error handling")
                //     let kind = crate::panicking::AssertKind::Eq;
                //     $crate::panicking::assert_failed(
                //         kind,
                //         &*left_val,
                //         &*right_val,
                //         $crate::option::Option::None,
                //     );
                }
            }
        };
        let mut ret = [0; 20];
        ret.copy_from_slice(&data[..]);
        Hash160(ret)
    }
}

impl std::hash::Hash for Hash160 {
    #[inline]
    fn hash<H>(&self, state: &mut H)
    where
        H: ::std::hash::Hasher,
    {
        (&self[..]).hash(state);
    }
    fn hash_slice<H>(data: &[Hash160], state: &mut H)
    where
        H: ::std::hash::Hasher,
    {
        for d in data.iter() {
            (&d[..]).hash(state);
        }
    }
}