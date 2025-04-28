use crate::signal::signal_proto::LwSignalTask;
use anyhow::Result;
use lwbpf::CGroupIterLoader;

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
        let cgroup_id = if let Some(task) = &msg.body {
            task.exec.as_ref().map_or(0, |e| e.cgroup_id)
        } else {
            0
        };

        if cgroup_id == 0 {
            return msg;
        }

        if let Ok(ancestors) = self.iter.ancestors(cgroup_id, 16) {
            for a in ancestors {
                if a.id == 0 {
                    break;
                }
                let cgroup_name = String::from_utf8_lossy(&a.name);
                if self.is_container(&cgroup_name) {
                    msg.body.as_mut().unwrap().container_id = Some(cgroup_name.into());
                }
            }
        }
        msg
    }
}

impl CGroupHandler {
    fn is_container(&self, cgroup_name: &str) -> bool {
        cgroup_name.starts_with("cri-containerd-")
    }
}
