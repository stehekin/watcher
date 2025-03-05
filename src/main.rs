use std::time::Duration;

mod bpf;
mod converter;
mod lock;
mod signal_proto {
    include!(concat!(env!("OUT_DIR"), "/signal.rs"));
}

fn main() {
    let mut l = lock::Lock::default();
    l.lock().expect("cannot lock");
    std::thread::sleep(Duration::from_secs(600));
}
