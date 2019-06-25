extern crate serde;
extern crate sleigh;
#[macro_use]
extern crate sleigh_derive;

#[cfg(test)]
mod tests {
    use super::*;

    use serde::{Deserialize, Serialize};

    use sleigh::Sleigh;
    #[derive(Debug, Deserialize, Serialize, Sleigh)]
    struct TestRecord {
        #[sleigh(key)]
        key: u64,
        name: String,
    }

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
        // TODO: get a tempfile name here.
        let db = sleigh::Db::new("test.db").unwrap();

        let p = TestRecord {
            key: 1,
            name: "test".to_string(),
        };

        db.put(&p).unwrap();
        let key: u64 = 1;
        let p1: TestRecord = db.get(&key).unwrap();
        println!("{:?}", p1);
    }
}
