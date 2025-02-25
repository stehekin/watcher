use std::time::Duration;

mod bpf;
mod lock;

fn main() {
    let mut l = lock::Lock::default();
    l.lock().expect("cannot lock");
    std::thread::sleep(Duration::from_secs(60));
}
