use libsecp256k1::PublicKey;
use ripemd::{Digest, Ripemd160};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Debug, Serialize, Deserialize)]
pub struct Hash160(
    #[serde(
        serialize_with = "hash20_json_serialize",
        deserialize_with = "hash20_json_deserialize"
    )]
    pub [u8; 20],
);

impl Hash160 {
    pub fn from_sha256(sha256_hash: &[u8; 32]) -> Hash160 {
        let mut rmd = Ripemd160::new();
        let mut ret = [0u8; 20];
        rmd.update(sha256_hash);
        ret.copy_from_slice(rmd.finalize().as_slice());
        Hash160(ret)
    }

    /// Create a hash by hashing some data
    // (borrowed from Andrew Poelstra)
    pub fn from_data(data: &[u8]) -> Hash160 {
        let sha2_result = Sha256::digest(data);
        let ripe_160_result = Ripemd160::digest(sha2_result.as_slice());
        Self::from(ripe_160_result.as_slice())
    }

    pub fn from_public_key(pubkey: &PublicKey) -> Hash160 {
        Self::from_data(&pubkey.serialize_compressed())
    }
}

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
