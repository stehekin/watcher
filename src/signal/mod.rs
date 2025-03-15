pub(crate) mod bpf;
mod converter;
mod signal_proto {
    use lazy_static::lazy_static;
    use prost::Message;

    include!(concat!(env!("OUT_DIR"), "/signal.rs"));
    lazy_static! {
        pub(crate) static ref SignalTypes: prost_types::FileDescriptorSet = {
            let file_descriptor_set_bytes: &[u8] =
                include_bytes!(concat!(env!("OUT_DIR"), "/signal_descriptor_set.bin"));
            prost_types::FileDescriptorSet::decode(&file_descriptor_set_bytes[..]).unwrap()
        };
    }
}
mod signal_store;
