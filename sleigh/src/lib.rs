extern crate serde;
extern crate serde_json;
extern crate sled;

pub mod key_encode;

pub use key_encode::encode_key;

use std::path::Path;
use std::sync::Arc;

#[derive(Debug)]
pub enum Error {
    Sled(sled::Error),
    Json(serde_json::Error),
    StringEncoding,
    Empty,
}

pub trait Sleigh {
    fn bucket() -> &'static [u8];
    fn key_name() -> &'static str;
    fn key_value(&self) -> Vec<u8>;
}

pub struct Db {
    db: sled::Db,
}

impl Db {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Db, Error> {
        let db = sled::Db::start_default(path).map_err(|e| Error::Sled(e))?;

        Ok(Db { db: db })
    }

    pub fn put<T>(&self, obj: &T) -> Result<(), Error>
    where
        T: Sleigh + serde::Serialize,
    {
        let s = serde_json::to_string(obj).map_err(|e| Error::Json(e))?;
        let key = obj.key_value();
        let tree = self.db.open_tree(T::bucket()).map_err(|e| Error::Sled(e))?;
        tree.set(&key, s.as_bytes()).map_err(|e| Error::Sled(e))?;
        Ok(())
    }

    // TODO: there is no checking that the type of K is the correct ID
    // type for T.
    pub fn get<'a, 'b, T, K>(&self, key: &K) -> Result<T, Error>
    where
        T: Sleigh + serde::de::DeserializeOwned,
        K: key_encode::KeyEncoder,
    {
        let tree = self.db.open_tree(T::bucket()).map_err(|e| Error::Sled(e))?;
        let val: Arc<[u8]> = tree
            .get(encode_key(key))
            .map_err(|e| Error::Sled(e))?
            .ok_or(Error::Empty)?
            .into();
        serde_json::from_slice::<T>(val.as_ref()).map_err(|e| Error::Json(e))
    }
}
