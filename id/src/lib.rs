pub mod db;

use eat::*;
use spit::*;

#[derive(Debug, PartialEq, Clone)]
pub struct DepValue<Id> {
    pub bytes: Vec<u8>,
    pub deps: Vec<Id>,
}

impl<'a, Id> Eat<&'a [u8], (), ()> for DepValue<Id>
where
    Id: Eat<&'a [u8], (), ()>,
{
    fn eat(i: &'a [u8], _data: ()) -> Result<(&[u8], Self), ()> {
        let (i, len_bytes) = u32::eat(i, ())?;
        let (i, bytes) = u8::eat_len(i, (), len_bytes as usize)?;
        let (i, len_deps) = u32::eat(i, ())?;
        let (i, deps) = Id::eat_len(i, (), len_deps as usize)?;
        Ok((i, DepValue { bytes, deps }))
    }
}

impl<Id> Spit<Vec<u8>, ()> for DepValue<Id>
where
    Id: Spit<Vec<u8>, ()>,
{
    fn spit(self, mut o: Vec<u8>) -> Result<Vec<u8>, ()> {
        o = (self.bytes.len() as u32).spit(o)?;
        o = self.bytes.spit(o)?;
        o = (self.deps.len() as u32).spit(o)?;
        for dep in self.deps {
            o = dep.spit(o)?;
        }
        Ok(o)
    }
}

#[cfg(test)]
mod tests {
    use super::db::*;
    use super::*;
    use std::collections::HashMap;
    use std::hash::Hash;

    #[test]
    fn test_1_dep() {
        const VALUE: u32 = 2137;
        let mut old_deps = HashMap::new();
        let value = DepValue::<u32> {
            bytes: vec![],
            deps: vec![],
        };
        old_deps.insert(VALUE, value);
        let bytes = old_deps.clone().spit(Vec::new()).unwrap();
        let (i, id_value) = <(u32, DepValue<u32>)>::eat_many(&bytes[..], ());
        let new_deps: HashMap<_, _> = id_value.into_iter().collect();
        assert!(i.is_empty());
        assert!(new_deps.len() == old_deps.len());
        assert_eq!(
            new_deps.get_key_value(&VALUE),
            old_deps.get_key_value(&VALUE)
        );
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct Id(u32);

    impl From<DepValue<Id>> for Id {
        fn from(value: DepValue<Id>) -> Self {
            let mut id = value.bytes.len() as u32;
            for dep in value.deps {
                id += dep.0 + 1;
            }
            Id(id)
        }
    }

    impl Eat<&[u8], (), ()> for Id {
        fn eat(i: &[u8], _data: ()) -> Result<(&[u8], Self), ()> {
            let (i, id) = u32::eat(i, ())?;
            Ok((i, Id(id)))
        }
    }

    impl Spit<Vec<u8>, ()> for Id {
        fn spit(self, o: Vec<u8>) -> Result<Vec<u8>, ()> {
            self.0.spit(o)
        }
    }

    #[test]
    fn test_db_load() {
        if !std::path::Path::new("test").exists() {
            std::fs::create_dir("test").unwrap();
        }
        std::fs::write("test/a_db", []).unwrap();
        std::fs::write("test/a_top", []).unwrap();
        let mut a: DB<Id> = DB::load("test/a_db", "test/a_top").unwrap();
        let id = a
            .push(DepValue {
                bytes: vec![],
                deps: vec![],
            })
            .unwrap();
        println!("{:?}", id);
        let id2 = a
            .push(DepValue {
                bytes: vec![],
                deps: vec![id],
            })
            .unwrap();
        println!("{:?}", id2);
        std::fs::write("test/b_db", []).unwrap();
        std::fs::write("test/b_top", []).unwrap();
        let mut b: DB<Id> = DB::load("test/b_db", "test/b_top").unwrap();
        b.migrate(&a, id2).unwrap();
        assert!(b.db.contains_key(&id2));
    }
}
