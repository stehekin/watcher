use super::signal_store::*;

use anyhow::{bail, Result};
use redb::{Database, ReadableTable, TableDefinition};
use std::sync::{Arc, RwLock};

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
                    let table: TableDefinition<String, &[u8]> = TableDefinition::new(entity_type);
                    let txn = db.begin_write()?;
                    {
                        let mut table = txn.open_table(table)?;
                        table.insert(key, entity.encode_to_vec().as_slice())?;
                    }
                    txn.commit()?;
                    Ok(())
                } else {
                    bail!("error opening db for writing");
                }
            }
            None => {
                bail!("entity has no key: {:#?}", entity);
            }
        }
    }

    fn for_each<T>(&self, entity_type: &str, visitor: impl Visitor) -> Result<()>
    where
        T: prost::Message + Default,
    {
        if let Ok(db) = self.db.read() {
            let table: TableDefinition<String, &[u8]> = TableDefinition::new(entity_type);
            let txn = db.begin_read()?;
            let table = txn.open_table(table)?;

            table.iter()?.for_each(move |v| {
                if let Ok(v) = v {
                    match T::decode(v.1.value()) {
                        Ok(entity) => {
                            visitor.visit(&entity);
                        }
                        Err(err) => {
                            log::error!("err decoding entity {0}", err)
                        }
                    }
                }
            });
            Ok(())
        } else {
            bail!("error opening db for reading");
        }
    }
}
