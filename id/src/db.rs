use super::DepValue;
use eat::*;
use spit::*;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::hash::Hash;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
pub struct DB<Id> {
    pub db: HashMap<Id, DepValue<Id>>,
    pub top: HashSet<Id>,
    pub db_path: PathBuf,
    pub top_path: PathBuf,
}

#[derive(Debug)]
pub enum Push<Id> {
    Found,
    DepNotFound(Id),
    Spit(Id),
    IO(std::io::Error),
}

#[derive(Debug)]
pub enum LoadDB<Id> {
    DBPath(std::io::Error),
    TopPath(std::io::Error),
    DBNotFullEat,
    TopNotFullEat,
    TopInsert(Id),
    TopRemove(Id),
}

impl<Id> DB<Id>
where
    Id: Copy
        + Debug
        + Eq
        + Hash
        + From<DepValue<Id>>
        + Spit<Vec<u8>, ()>
        + for<'a> Eat<&'a [u8], (), ()>,
{
    pub fn load(db_path: &str, top_path: &str) -> Result<DB<Id>, LoadDB<Id>> {
        let bytes = std::fs::read(db_path).map_err(LoadDB::DBPath)?;
        let (i, db) = <(Id, DepValue<Id>)>::eat_many(&bytes[..], ());
        if !i.is_empty() {
            return Err(LoadDB::DBNotFullEat);
        }
        let bytes = std::fs::read(top_path).map_err(LoadDB::TopPath)?;
        let (i, commands) = <(u8, Id)>::eat_many(&bytes[..], ());
        if !i.is_empty() {
            return Err(LoadDB::TopNotFullEat);
        }
        let mut top = HashSet::new();
        for (command, id) in commands {
            match command {
                b'+' => {
                    if !top.insert(id) {
                        return Err(LoadDB::TopInsert(id));
                    }
                }
                b'-' => {
                    if !top.remove(&id) {
                        return Err(LoadDB::TopRemove(id));
                    }
                }
                _ => {}
            }
        }
        Ok(DB {
            db: db.into_iter().collect(),
            top,
            db_path: PathBuf::from(db_path),
            top_path: PathBuf::from(top_path),
        })
    }

    pub fn push(&mut self, value: DepValue<Id>) -> Result<Id, Push<Id>> {
        let id = Id::from(value.clone());
        if self.db.contains_key(&id) {
            return Err(Push::Found);
        }
        if let Some(dep) = value.deps.iter().find(|dep| !self.db.contains_key(dep)) {
            return Err(Push::DepNotFound(*dep));
        }
        let top_deps = value.deps.iter().filter(|dep| self.top.contains(dep));
        let top_deps: Vec<_> = top_deps.map(|x| *x).collect();
        {
            let commands = top_deps.iter().map(|x| ('-' as u8, *x));
            let bytes =
                <(u8, Id)>::spit_many(commands, Vec::new()).map_err(|pair| Push::Spit(pair.1))?;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.top_path)
                .map_err(Push::IO)?;
            file.write_all(&bytes).map_err(Push::IO)?;
        }
        for dep in top_deps.iter() {
            self.top.remove(dep);
        }
        {
            let bytes = value.clone().spit(Vec::new()).map_err(|_| Push::Spit(id))?;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.db_path)
                .map_err(Push::IO)?;
            file.write_all(&bytes).map_err(Push::IO)?;
        }
        self.db.insert(id, value);
        {
            let bytes = vec!['+' as u8];
            let bytes = id.spit(bytes).map_err(|_| Push::Spit(id))?;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.top_path)
                .map_err(Push::IO)?;
            file.write_all(&bytes).map_err(Push::IO)?;
        }
        self.top.insert(id);
        Ok(id)
    }

    pub fn migrate(self: &mut DB<Id>, other: &DB<Id>, id: Id) -> Result<usize, ()> {
        let value = other.db.get(&id).ok_or(())?;
        if self.db.contains_key(&id) {
            return Ok(0);
        }
        let mut migrated = 0;
        for dep in value.deps.iter() {
            migrated += self.migrate(other, *dep)?;
        }
        self.push(value.clone()).map_err(|_| ())?;
        Ok(migrated + 1)
    }
}
