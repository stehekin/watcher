use crate::signal::signal_proto::LwSignalTask;
use lwbpf::CGroupIterLoader;

use anyhow::Result;
use std::mem::MaybeUninit;

pub(crate) struct CGroupHandler {
    iter: CGroupIterLoader,
}

impl CGroupHandler {
    pub fn new() -> Result<Self> {
        let iter = CGroupIterLoader::new()?;
        Ok(Self { iter })
    }
}

impl super::Handler<LwSignalTask> for CGroupHandler {
    fn handle(&self, mut msg: LwSignalTask) -> LwSignalTask {
        if let Some(task) = &msg.body {
            let cgroup_id = match &task.exec {
                Some(exec) => exec.cgroup_id,
                None => 0,
            };

            if let Some(exec) = &task.exec {
                print!(
                    "--> {0} \n ------------------- \n",
                    exec.filename.as_ref().unwrap()
                );
            };

            if cgroup_id == 0 {
                return msg;
            }

            if let Ok(ancestors) = self.iter.ancestors(cgroup_id, 16) {
                for i in ancestors {
                    if i.id == 0 {
                        break;
                    }
                    print!("{0}\t\t", i);
                }
                println!("");
            }
        }
        msg
    }
}
