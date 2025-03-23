use super::signal_store::*;

use anyhow::{bail, Result};
use redis::*;
use std::{
    fmt::format,
    sync::{Arc, RwLock},
};

pub(crate) struct RedisStore {
    pool: r2d2::Pool<redis::Client>,
}

unsafe impl Sync for RedisStore {}
unsafe impl Send for RedisStore {}

impl RedisStore {
    pub(crate) fn new(host: &str, port: u32, password: &str) -> Result<Self> {
        let client = redis::Client::open(format!("redis://:{2}@{0}:{1}/", host, port, password))?;
        let pool = r2d2::Pool::builder().max_size(3).build(client)?;
        Ok(RedisStore { pool })
    }

    fn redis_key<T>(entity_type: &str, entity: &T) -> String
    where
        T: HasKey + prost::Message,
    {
        if let Some(key) = entity.key() {
            return format!("{0}::{1}", entity_type, key);
        } else {
            log::warn!("entity has no key: {:#?}", entity);
            return String::new();
        }
    }
}

impl SignalStore for RedisStore {
    fn save_signal_proto<T>(&self, entity_type: &str, entity: &T) -> Result<()>
    where
        T: prost::Message + HasKey,
    {
        let mut conn = self.pool.get()?;
        let _: () = conn.set(
            Self::redis_key(entity_type, entity),
            entity.encode_to_vec().as_slice(),
        )?;
        Ok(())
    }

    fn for_each<T>(&self, entity_type: &str, visitor: impl Visitor) -> Result<()>
    where
        T: prost::Message + Default,
    {
        let mut conn = self.pool.get()?;
        conn.scan_match::<String, String>(format!("{0}::*", entity_type))?
            .for_each(|k| {
                let v: Vec<u8> = self.pool.get().unwrap().get(k).unwrap();
                match T::decode(v.as_slice()) {
                    Ok(entity) => {
                        visitor.visit(&entity);
                    }
                    Err(err) => {
                        log::error!("err decoding entity {0}", err)
                    }
                }
            });
        Ok(())
    }
}
