use super::signal_proto::*;
use bpf_lib::lw_blobstr;

pub(crate) fn slice_to_string(slice: &[u8]) -> String {
    let mut i = 0;
    while i < slice.len() && slice[i] != 0 {
        i += 1;
    }

    return String::from_utf8_lossy(&slice[..i]).to_string();
}

impl From<bpf_lib::lw_creds> for LwCreds {
    fn from(c_creds: bpf_lib::lw_creds) -> Self {
        LwCreds {
            uid: c_creds.uid,
            gid: c_creds.gid,
            euid: c_creds.euid,
            egid: c_creds.egid,
        }
    }
}

impl From<bpf_lib::lw_pid> for LwPid {
    fn from(c_pid: bpf_lib::lw_pid) -> Self {
        LwPid {
            pid: c_pid.pid,
            tgid: c_pid.tgid,
            pid_ns: c_pid.pid_ns,
            pid_vnr: c_pid.pid_vnr,
        }
    }
}

fn convert_lwblob_str(blobstr: &lw_blobstr) -> Option<String> {
    unsafe {
        if blobstr.blob.flag == 0 {
            let mut i: usize = 0;
            if i < blobstr.str_.len() && blobstr.str_[i] != 0 {
                i += 1;
            }
            return Some(String::from_utf8_lossy(&blobstr.str_[..i]).into());
        }
    }

    None
}

impl From<bpf_lib::lw_exec> for LwExec {
    fn from(c_exec: bpf_lib::lw_exec) -> Self {
        LwExec {
            filename: convert_lwblob_str(&c_exec.filename),
            interp: convert_lwblob_str(&c_exec.interp),
            cgroup_id: c_exec.cgroup_id,
            args: None,
            env: None,
        }
    }
}

impl From<bpf_lib::lw_parent> for LwParent {
    fn from(c_parent: bpf_lib::lw_parent) -> Self {
        LwParent {
            pid: c_parent.pid,
            tgid: c_parent.tgid,
            boot_ns: c_parent.boot_ns,
        }
    }
}

impl From<bpf_lib::lw_task> for LwTask {
    fn from(c_task: bpf_lib::lw_task) -> Self {
        LwTask {
            creds: Some(c_task.creds.into()),
            pid: Some(c_task.pid.into()),
            parent: Some(c_task.parent.into()),
            session_id: c_task.session_id,
            login_uid: c_task.login_uid,
            exec: Some(c_task.exec.into()),
            boot_ns: c_task.boot_ns,
        }
    }
}

impl From<bpf_lib::lw_signal_header> for LwSignalHeader {
    fn from(c_signal_header: bpf_lib::lw_signal_header) -> Self {
        LwSignalHeader {
            version: c_signal_header.version as u32,
            signal_type: LwSignalType::LwSignalTask.into(),
            cpu_id: c_signal_header.cpu_id as u32,
            submit_time_ns: c_signal_header.submit_time_ns,
        }
    }
}

impl From<bpf_lib::lw_signal_task> for LwSignalTask {
    fn from(c_signal_task: bpf_lib::lw_signal_task) -> Self {
        LwSignalTask {
            header: Some(c_signal_task.header.into()),
            body: Some(c_signal_task.body.into()),
        }
    }
}

impl super::signal_store::HasKey for LwSignalTask {
    fn key(&self) -> Option<String> {
        Some(format!(
            "{0}::{1}",
            self.body.as_ref()?.boot_ns,
            self.body.as_ref()?.pid?.pid,
        ))
    }
}
