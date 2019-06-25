#[macro_use]
extern crate sleigh_derive;
use serde::{Deserialize, Serialize};

use sleigh::Sleigh;
#[derive(Debug, Deserialize, Serialize, Sleigh)]
struct TestRecord {
    #[sleigh(key, auto)]
    key: u64,
    name: String,
}

#[derive(Debug, Deserialize, Serialize, Sleigh)]
struct TestStringRecord {
    #[sleigh(key)]
    key: String,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleigh_bucket() {
        assert_eq!(TestRecord::bucket(), b"TestRecord");
    }

    #[test]
    fn sleigh_key_name() {
        println!("{}", TestRecord::key_name());
        assert_eq!(TestRecord::key_name(), "key");
    }

    #[test]
    fn sleigh_key_value() {
        let p = TestRecord {
            key: 1,
            name: "test".to_string(),
        };
        println!("{:?}", p.key_value());
        assert_eq!(p.key_value(), vec![0, 0, 0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn sleigh_put_get() {
        let dir = tempfile::tempdir().unwrap();
        {
            let db = sleigh::Db::new(dir.path()).unwrap();

            let mut p = TestRecord {
                key: 1,
                name: "test".to_string(),
            };

            db.put(&mut p).unwrap();
            let key: u64 = 1;
            let p1: TestRecord = db.get(&key).unwrap();
            assert_eq!(p1.key, 1);
            assert_eq!(p1.name, "test".to_string());
        }
    }

    #[test]
    fn sleigh_auto() {
        let dir = tempfile::tempdir().unwrap();
        {
            let db = sleigh::Db::new(dir.path()).unwrap();

            let mut p = TestRecord {
                key: 0,
                name: "test".to_string(),
            };

            db.put(&mut p).unwrap();
            assert_ne!(p.key, 0);

            let key: u64 = p.key;
            let p1: TestRecord = db.get(&key).unwrap();
            assert_eq!(p1.key, p.key);
            assert_eq!(p1.name, "test".to_string());
        }
    }
}
