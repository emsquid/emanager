use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{io::Write, marker::PhantomData};

const DIR: &str = "/home/emanuel/.local/state/emanager";

pub struct Stater<T: Serialize + for<'a> Deserialize<'a>> {
    file: String,
    phantom: PhantomData<T>,
}

impl<T: Serialize + for<'a> Deserialize<'a>> Stater<T> {
    pub fn new(name: &str) -> Self {
        Self {
            file: format!("{DIR}/{name}"),
            phantom: PhantomData,
        }
    }

    pub fn write(&self, state: &T) -> anyhow::Result<()> {
        std::fs::create_dir_all(DIR)?;
        self.truncate()?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file)?;
        let json = serde_json::to_vec(&state)?;
        file.write_all(&json)?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn read(&self) -> anyhow::Result<T> {
        let state = std::fs::read_to_string(&self.file)?
            .lines()
            .last()
            .map(String::from)
            .ok_or(anyhow!("State not found"))?;
        Ok(serde_json::from_str(&state)?)
    }

    fn truncate(&self) -> anyhow::Result<()> {
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.file)
        {
            if file.metadata()?.len() > 2_u64.pow(16) {
                file.set_len(0)?;
            }
        }
        Ok(())
    }
}
