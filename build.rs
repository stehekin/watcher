fn main() {
    prost_build::compile_protos(&["src/signal.proto"], &["src/"]).unwrap();
}
