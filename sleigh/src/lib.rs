pub mod key_encode;

pub use key_encode::encode_key;
use std::path::Path;
use std::sync::Arc;

pub const AUTO_ASSIGN: u64 = u64::max_value();

#[derive(Debug)]
pub enum Error {
    Sled(sled::Error),
    Json(serde_json::Error),
    Generic(String),
    StringEncoding,
    Empty,
}

impl std::convert::From<sled::Error> for Error {
    fn from(e: sled::Error) -> Error {
        Error::Sled(e)
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Generic(format!("{}", e))
    }
}

pub trait Sleigh {
    fn bucket() -> &'static [u8];
    fn key_name() -> &'static str;
    fn key_value(&self) -> Vec<u8>;
    fn prep(&mut self, db: &Db) -> Result<(), Error>;
}

pub trait Codec {
    fn encode<T>(obj: &T) -> Result<Vec<u8>, Error>;
    fn decode<T>(data: &[u8]) -> Result<(), Error>;
}

pub struct Db {
    db: sled::Db,
}

impl Db {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Db, Error> {
        let db = sled::Db::start_default(path)?;

        Ok(Db { db })
    }

    pub fn get_unique_key(&self) -> Result<u64, Error> {
        let mut id: u64 = 0;
        // Sled does not guarantee non zero ids.  Loop until we get one.
        while id == 0 {
            id = self.db.generate_id()?;
        }
        Ok(id)
    }

    pub fn put<T>(&self, obj: &mut T) -> Result<(), Error>
    where
        T: Sleigh + serde::Serialize,
    {
        obj.prep(&self)?;
        let s = serde_json::to_string(obj).map_err(Error::Json)?;
        let key = obj.key_value();
        let tree = self.db.open_tree(T::bucket()).map_err(Error::Sled)?;
        tree.set(&key, s.as_bytes()).map_err(Error::Sled)?;
        Ok(())
    }

    // TODO: there is no checking that the type of K is the correct ID
    // type for T.
    pub fn get<T, K>(&self, key: &K) -> Result<T, Error>
    where
        T: Sleigh + serde::de::DeserializeOwned,
        K: key_encode::KeyEncoder,
    {
        let tree = self.db.open_tree(T::bucket()).map_err(Error::Sled)?;
        let val: Arc<[u8]> = tree
            .get(encode_key(key))
            .map_err(Error::Sled)?
            .ok_or(Error::Empty)?
            .into();
        serde_json::from_slice::<T>(val.as_ref()).map_err(Error::Json)
    }
}
