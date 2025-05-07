use crate::signal::bpf;
use crate::signal::pipeline::CGroupHandler;
use crate::signal::pipeline::Pipeline;

use tokio::sync::mpsc::unbounded_channel;

mod lock;
mod signal;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let mut l = lock::Lock::default();
    l.lock().expect("cannot lock");

    let (sender, mut receiver) = unbounded_channel();

    tokio::spawn(async move {
        let mut pipeline = Pipeline::<signal::signal_proto::LwSignalTask>::default();
        if let Ok(cgroup_handler) = CGroupHandler::new() {
            pipeline.add_handler(Box::new(cgroup_handler));
        }

        loop {
            if let Some(mut t) = receiver.recv().await {
                let task = pipeline.process(t);
                if let Some(cid) = &task.body.as_ref().unwrap().container_id {
                    print!(
                        "{0} --> {1}\n",
                        task.body
                            .as_ref()
                            .unwrap()
                            .exec
                            .as_ref()
                            .unwrap()
                            .filename
                            .as_ref()
                            .unwrap(),
                        cid
                    );
                }
            }
        }
    });

    bpf::start_bpf(sender).await;
}
