use crate::converter::*;
use crate::signal_proto::LwSignalTask;

use bpf_lib::lw_signal_task;
use bpf_lib::run;
use moka::future::{Cache, CacheBuilder};
use moka::policy::EvictionPolicy;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::oneshot;

const _MAX_BLOB_CACHE_SIZE: u64 = 256;
const _MAX_TASK_BLOB_GAP: Duration = Duration::from_secs(1);

async fn convert_task_and_send(
    task: lw_signal_task,
    blob_cache: Cache<u64, String>,
    task_proto_sender: UnboundedSender<LwSignalTask>,
) {
    let mut filename: Option<String> = None;
    let mut interp: Option<String> = None;
    let mut args: Option<String> = None;
    let mut env: Option<String> = None;

    unsafe {
        let blob = &task.body.exec.filename.blob;
        if blob.flag == 0 && blob.blob_id != 0 {
            filename = blob_cache.remove(&blob.blob_id).await;
        } else {
            filename = Some(slice_to_string(task.body.exec.filename.str_.as_slice()));
        }

        let blob = &task.body.exec.interp.blob;
        if blob.flag == 0 && blob.blob_id != 0 {
            interp = blob_cache.remove(&blob.blob_id).await;
        } else {
            interp = Some(slice_to_string(task.body.exec.interp.str_.as_slice()));
        }
    }

    if task.body.exec.args != 0 {
        args = blob_cache.remove(&task.body.exec.args).await;
    }

    if task.body.exec.env != 0 {
        env = blob_cache.remove(&task.body.exec.env).await;
    }

    let mut task: LwSignalTask = task.into();
    task.body
        .get_or_insert_default()
        .exec
        .get_or_insert_default()
        .filename = filename;
    task.body
        .get_or_insert_default()
        .exec
        .get_or_insert_default()
        .interp = interp;
    task.body
        .get_or_insert_default()
        .exec
        .get_or_insert_default()
        .args = args;
    task.body
        .get_or_insert_default()
        .exec
        .get_or_insert_default()
        .env = env;

    match task_proto_sender.send(task) {
        Ok(_) => {}
        Err(e) => {
            log::error!("error sending task proto {0}", e)
        }
    }
}

async fn handle_signal_tasks(
    task_sender: UnboundedSender<lw_signal_task>,
    mut task_receiver: UnboundedReceiver<lw_signal_task>,
    blob_cache: Cache<u64, String>,
    task_proto_sender: UnboundedSender<LwSignalTask>,
) {
    loop {
        if let Some(t) = task_receiver.recv().await {
            let mut all_arrived = true;
            // Check if all blobs have arrived.
            unsafe {
                let blob = &t.body.exec.filename.blob;
                if all_arrived && blob.flag == 0 && blob.blob_id != 0 {
                    all_arrived = all_arrived && blob_cache.contains_key(&blob.blob_id);
                }

                let blob = &t.body.exec.interp.blob;
                if all_arrived && blob.flag == 0 && blob.blob_id != 0 {
                    all_arrived = all_arrived && blob_cache.contains_key(&blob.blob_id);
                }
            }

            if all_arrived && t.body.exec.args != 0 {
                all_arrived = all_arrived && blob_cache.contains_key(&t.body.exec.args);
            }

            if all_arrived && t.body.exec.env != 0 {
                all_arrived = all_arrived && blob_cache.contains_key(&t.body.exec.env);
            }

            if !all_arrived {
                let task_instant = Instant::now()
                    .checked_sub(Duration::from_nanos(t.header.submit_time_ns))
                    .unwrap();

                if Instant::now()
                    .duration_since(task_instant)
                    .le(&_MAX_TASK_BLOB_GAP)
                {
                    match task_sender.send(t) {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("error sending task {0}", e);
                        }
                    }
                    continue;
                }
            }

            convert_task_and_send(t, blob_cache.clone(), task_proto_sender.clone()).await;
        }
    }
}

pub(crate) async fn start_bpf(task_proto_sender: UnboundedSender<LwSignalTask>) {
    let (task_sender, task_receiver) = unbounded_channel();
    let (merged_blob_sender, mut merged_blob_receiver) = unbounded_channel();
    let (exit_sender, exit_receiver) = oneshot::channel();

    let run_handle = run(task_sender.clone(), merged_blob_sender, exit_receiver);

    let blob_cache = CacheBuilder::new(_MAX_BLOB_CACHE_SIZE)
        .eviction_policy(EvictionPolicy::lru())
        .build();

    let task_sender = task_sender.clone();
    let blob_cache_clone = blob_cache.clone();
    tokio::spawn(handle_signal_tasks(
        task_sender,
        task_receiver,
        blob_cache_clone,
        task_proto_sender,
    ));

    tokio::spawn(async move {
        loop {
            if let Some(blob) = merged_blob_receiver.recv().await {
                blob_cache
                    .insert(blob.0, slice_to_string(blob.1.as_slice()))
                    .await;
            }
        }
    });

    run_handle.await;
}

mod test {
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_start_bpf() {
        let (sender, mut receiver) = unbounded_channel();

        tokio::spawn(async move {
            loop {
                let t = receiver.recv().await;
                print!("--->{:?}\n", t);
            }
        });

        super::start_bpf(sender).await;
    }
}
