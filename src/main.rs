use std::time::Duration;

mod lock;
mod signal;

fn main() {
    let mut l = lock::Lock::default();
    l.lock().expect("cannot lock");
    std::thread::sleep(Duration::from_secs(600));
}
