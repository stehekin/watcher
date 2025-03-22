use super::signal_proto::LwSignalTask;
use super::signal_store::*;

use anyhow::{bail, Result};
use nix::libc::printf;
use prost::Message;
use redb::{
    Database, ReadableTable, ReadableTableMetadata, TableDefinition, TableHandle, TypeName, Value,
};
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
                        print!("--->there1 {:?}\n", table.name());
                    }
                    txn.commit()?;
                    Ok(())
                } else {
                    bail!("error opening db for writes");
                }
            }
            None => {
                bail!("entity has no key: {:#?}", entity);
            }
        }
    }

    fn for_each<T>(&self, entity_type: &str /*visitor: impl Visitor*/) -> Result<()>
    where
        T: prost::Message + Default,
    {
        print!("here1\n");
        if let Ok(db) = self.db.read() {
            print!("here2\n");
            let table: TableDefinition<String, &[u8]> = TableDefinition::new(entity_type);
            let txn = db.begin_read()?;
            print!("here3\n");
            let table = txn.open_table(table)?;
            print!("here4\n");
            print!("here5 {:?}\n", table.len());

            table.iter()?.for_each(move |v| {
                print!("for each reading \n");
                if let Ok(v) = v {
                    match T::decode(v.1.value()) {
                        Ok(entity) => {
                            print!("--> {:?} \n", entity);
                            // visitor.vist(&entity);
                        }
                        Err(err) => {
                            print!("eror decoding");
                        }
                    }
                }
            });
            Ok(())
        } else {
            bail!("error opening db for read");
        }
    }
}
