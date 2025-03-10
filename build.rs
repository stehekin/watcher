fn main() {
    prost_build::compile_protos(&["src/signal/signal.proto"], &["src/"]).unwrap();
}
