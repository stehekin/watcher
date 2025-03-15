use std::env;
use std::path::PathBuf;

fn main() {
    let mut config = prost_build::Config::new();
    config.file_descriptor_set_path(
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR environment variable not set"))
            .join("signal_descriptor_set.bin"),
    );
    config
        .compile_protos(&["src/signal/signal.proto"], &["src/"])
        .unwrap();
}
