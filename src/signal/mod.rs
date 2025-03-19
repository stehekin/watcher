pub(crate) mod bpf;
mod converter;
mod signal_proto {
    include!(concat!(env!("OUT_DIR"), "/signal.rs"));
}
mod signal_store;
