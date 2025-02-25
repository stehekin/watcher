use anyhow::Result;
use nix::fcntl::{Flock, FlockArg};
use std::{
    fs::{File, OpenOptions},
    io::Write,
};

const _LOCK_FILE: &str = "/var/.lw_watcher.lock";

#[derive(Default)]
pub struct Lock {
    flock: Option<Flock<File>>,
}

impl Drop for Lock {
    fn drop(&mut self) {
        if self.flock.is_some() {
            let _ = std::fs::remove_file(_LOCK_FILE);
        }
    }
}

impl Lock {
    pub(crate) fn lock(&mut self) -> Result<()> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(_LOCK_FILE)?;
        let mut file = Flock::lock(file, FlockArg::LockExclusiveNonblock)
            .map_err(|(_, err)| anyhow::Error::msg(err))?;
        file.write(std::process::id().to_string().as_bytes())?;
        self.flock = Some(file);
        Ok(())
    }
}
