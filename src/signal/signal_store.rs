use anyhow::Result;

pub(crate) trait Visitor {
    fn visit<T>(&self, entity: &T)
    where
        T: prost::Message + Default;
}

pub(crate) trait SignalStore: Sync {
    fn save_signal_proto<T>(&self, entity_type: &str, entity: &T) -> Result<()>
    where
        T: prost::Message + HasKey;

    fn for_each<T>(&self, entity_type: &str, visitor: impl Visitor) -> Result<()>
    where
        T: prost::Message + Default;
}

pub(crate) trait HasKey {
    fn key(&self) -> Option<String>;
}

pub(crate) const ENTITY_TASK_PROTO: &str = "tasks";
