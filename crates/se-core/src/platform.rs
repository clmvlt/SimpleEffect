//! What differs between operating systems for the core process itself (golden rule 10).

/// Ties every child process to the core: when the core exits or is killed, its sidecars die with
/// it.
///
/// On Windows the core joins a job object with `KILL_ON_JOB_CLOSE`; children inherit the job, so
/// even an ffmpeg killed with the core in the instant process-wrap holds it suspended (before it
/// joins its own job) cannot outlive it. The job's handle is deliberately never closed: closing
/// the last handle would kill the core itself, so the OS closes it when the process ends.
/// Elsewhere each sidecar's process group is stopped when its job is cancelled, and stdin EOF
/// stops the core gracefully. Failing to join the job is logged, never fatal.
#[cfg(windows)]
pub fn tie_children_to_this_process() {
    let joined = win32job::Job::create_with_limit_info(
        win32job::ExtendedLimitInfo::new().limit_kill_on_job_close(),
    )
    .and_then(|job| job.assign_current_process().map(|()| job));
    match joined {
        // Leaked on purpose (see above): the handle lives as long as the process.
        Ok(job) => drop(job.into_handle()),
        Err(error) => tracing::warn!(
            %error,
            "the core could not join a kill-on-close job: a sidecar may outlive a crash"
        ),
    }
}

/// Ties every child process to the core (see the Windows version): nothing to do here.
#[cfg(not(windows))]
pub fn tie_children_to_this_process() {}
