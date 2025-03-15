use super::signal_proto::*;

use anyhow::{bail, Result};
use prost::Message;
use redb::{Database, TableDefinition};
use std::sync::{Arc, RwLock};

pub trait SignalStore: Sync {
    fn save_signal_proto<T>(&self, table: TableDefinition<&str, Vec<u8>>, entity: &T) -> Result<()>
    where
        T: prost::Message;
}

pub struct RedbStore {
    db: Arc<RwLock<Database>>,
}

unsafe impl Sync for RedbStore {}
unsafe impl Send for RedbStore {}

const TASK_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("tasks");

impl SignalStore for RedbStore {
    fn save_signal_proto<T>(&self, table: TableDefinition<&str, Vec<u8>>, entity: &T) -> Result<()>
    where
        T: prost::Message,
    {
        if let Ok(db) = self.db.write() {
            let txn = db.begin_write()?;
            let mut table = txn.open_table(table)?;
            table.insert("key", entity.encode_to_vec())?;
            Ok(())
        } else {
            bail!("error open db");
        }
    }
}

mod test {
    use crate::signal::signal_proto::SignalTypes;

    #[test]
    fn test_descriptor() {
        let p = SignalTypes
            .file
            .get(0)
            .unwrap()
            .message_type
            .get(0)
            .unwrap();
        print!(">>>{:?}<<<\n", p.name());
    }
}
