pub(crate) mod bpf;
mod converter;
mod signal_proto {
    include!(concat!(env!("OUT_DIR"), "/signal.rs"));
}
mod pipeline;
mod signal_store;
mod signal_store_redis;
