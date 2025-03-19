use anyhow::{bail, Result};
use redb::{Database, TableDefinition};
use std::sync::{Arc, RwLock};

pub(crate) trait SignalStore: Sync {
    fn save_signal_proto<T>(&self, entity_type: &str, entity: &T) -> Result<()>
    where
        T: prost::Message + HasKey;
}

pub(crate) trait HasKey {
    fn key(&self) -> Option<String>;
}

pub(crate) const ENTITY_TASK_PROTO: &str = "tasks";

pub(crate) struct RedbStore {
    db: Arc<RwLock<Database>>,
}

unsafe impl Sync for RedbStore {}
unsafe impl Send for RedbStore {}

impl RedbStore {
    pub(crate) fn new(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let db = Database::create(path)?;
        Ok(RedbStore {
            db: Arc::new(RwLock::new(db)),
        })
    }
}

impl SignalStore for RedbStore {
    fn save_signal_proto<T>(&self, entity_type: &str, entity: &T) -> Result<()>
    where
        T: prost::Message + HasKey,
    {
        match entity.key() {
            Some(key) => {
                if let Ok(db) = self.db.write() {
                    let table: TableDefinition<String, Vec<u8>> = TableDefinition::new(entity_type);
                    let txn = db.begin_write()?;
                    let mut table = txn.open_table(table)?;
                    table.insert(key, entity.encode_to_vec())?;
                    Ok(())
                } else {
                    bail!("error opening db");
                }
            }
            None => {
                bail!("entity has no key: {:#?}", entity);
            }
        }
    }
}
