use anyhow::Result;
use std::fs::File;
use std::io::{Read, Write};

pub trait Persist {
    fn new_empty() -> Self;

    fn from_str(s: &str) -> Result<Self>
    where
        Self: Sized;

    fn to_string(&self) -> Result<String>;

    fn write_to_disk(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;
        let string = self.to_string()?;
        file.write_all(string.as_bytes())?;
        Ok(())
    }

    fn load_from_disk(path: &str) -> Result<Self>
    where
        Self: Sized,
    {
        let mut file = File::open(path)?;
        let mut string = String::new();
        file.read_to_string(&mut string)?;
        Self::from_str(&string)
    }

    fn load(path: &str) -> Result<Self>
    where
        Self: Sized,
    {
        let on_disk = Self::load_from_disk(path);
        if on_disk.is_err() {
            let slf = Self::new_empty();
            slf.write_to_disk(path)?;
            return Ok(slf);
        }
        on_disk
    }

    // TODO: remove the param of this function?
    // TODO: change reload into a sync
    fn reload(&mut self, path: &str) -> Result<()>
    where
        Self: Sized,
    {
        let new = Self::load(path)?;
        *self = new;

        Ok(())
    }
}
