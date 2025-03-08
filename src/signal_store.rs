use crate::signal_proto::*;

use anyhow::Result;
use prost::Message;
use redb::{Database, TableDefinition};
use std::borrow::Cow;

pub trait SignalStore {
    fn save_signal_task(&self, task: &LwSignalTask) -> Result<()>;
}

pub struct RedbStore {
    db: Database,
}

const TASK_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("tasks");

impl SignalStore for RedbStore {
    fn save_signal_task(&self, task: &LwSignalTask) -> Result<()> {
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(TASK_TABLE)?;
            table.insert("key", task.encode_to_vec())?;
        }
        Ok(())
    }
}
