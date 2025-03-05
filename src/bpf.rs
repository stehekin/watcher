use crate::converter::*;
use crate::signal_proto::{LwSignalTask, LwTask};

use bpf_lib::run;

use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::oneshot;

pub trait SignalStore {
    fn save_signal_task(&mut self, task: &LwSignalTask);
}

pub(crate) async fn start_bpf<T>(store: Arc<T>)
where
    T: SignalStore + Send + Clone,
{
    let (task_sender, mut task_receiver) = unbounded_channel();
    let (merged_blob_sender, mut merged_blob_receiver) = unbounded_channel();
    let (exit_sender, exit_receiver) = oneshot::channel();

    let run_handle = run(task_sender, merged_blob_sender, exit_receiver);

    let mut task_store = store.clone();
    tokio::spawn(async move {
        loop {
            if let Some(t) = task_receiver.recv().await {
                let task: LwSignalTask = t.into();
                task_store.save_signal_task(&task);
            }
        }
    });

    tokio::spawn(async move {
        loop {
            if let Some(t) = merged_blob_receiver.recv().await {
                unsafe {
                    print!("blog: {0}\n", String::from_utf8_lossy(t.1.as_slice()));
                }
            }
        }
    });
}
