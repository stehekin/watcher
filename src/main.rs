use crate::signal::bpf;

use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

mod lock;
mod signal;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let mut l = lock::Lock::default();
    l.lock().expect("cannot lock");

    let (sender, mut receiver) = unbounded_channel();

    tokio::spawn(async move {
        loop {
            let t = receiver.recv().await;
            print!("--->{:?}\n", t);
        }
    });

    bpf::start_bpf(sender).await;
}
