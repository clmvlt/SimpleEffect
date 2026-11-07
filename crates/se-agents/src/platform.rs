//! Everything that differs between operating systems for CLI processes (golden rule 10): how a
//! process is isolated so its whole tree can be stopped and no console window opens.

use process_wrap::tokio::CommandWrap;

/// Puts the process in a job object (Windows: killing the job kills the tree, including the
/// `node` behind a `.cmd` shim; no console window opens) or a new process group (Unix: killing
/// the group kills the tree). The tree is also killed when the process handle is dropped.
#[cfg(windows)]
pub fn isolate(command: &mut CommandWrap) {
    use process_wrap::tokio::{CreationFlags, JobObject, KillOnDrop};
    use windows::Win32::System::Threading::CREATE_NO_WINDOW;
    command
        .wrap(CreationFlags(CREATE_NO_WINDOW))
        .wrap(JobObject)
        .wrap(KillOnDrop);
}

/// Puts the process in a job object (Windows: killing the job kills the tree, including the
/// `node` behind a `.cmd` shim; no console window opens) or a new process group (Unix: killing
/// the group kills the tree). The tree is also killed when the process handle is dropped.
#[cfg(not(windows))]
pub fn isolate(command: &mut CommandWrap) {
    use process_wrap::tokio::{KillOnDrop, ProcessGroup};
    command.wrap(ProcessGroup::leader()).wrap(KillOnDrop);
}
