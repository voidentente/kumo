#![feature(exit_status_error)]

//! Thin wrapper process (guard) for meilisearch.
//! Listens for PDEATHSIG to determine when to kill meili and self.
//! Forwards all arguments after `--` to meilisearch.
//!
//! Kills meilisearch by dropping UnwindGuard.
//! This happens either if this process panics
//! *or* if the parent process exits *under any circumstance*.
//! It does *not* kill meilisearch if meiliguard aborts.

#[cfg(unix)]
mod inner {
    use std::process::Child;
    use std::sync::OnceLock;

    pub(super) extern "C" fn handle_sigusr1(_: libc::c_int) {
        unsafe {
            MEILI.take().unwrap();
        }
    }

    pub(super) static mut MEILI: OnceLock<UnwindGuard> = OnceLock::new();

    #[derive(Debug)]
    pub(super) struct UnwindGuard(pub(super) Child);

    impl Drop for UnwindGuard {
        fn drop(&mut self) {
            #[allow(unused_must_use)]
            {
                self.0.kill();
            }
        }
    }
}

#[cfg(unix)]
fn main() {
    unsafe {
        libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGUSR1);
    }

    let sig_action = nix::sys::signal::SigAction::new(
        nix::sys::signal::SigHandler::Handler(inner::handle_sigusr1),
        nix::sys::signal::SaFlags::empty(),
        nix::sys::signalfd::SigSet::empty(),
    );

    unsafe {
        nix::sys::signal::sigaction(nix::sys::signal::Signal::SIGUSR1, &sig_action).unwrap();
    }

    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();

    let meili_dir = meiliguard::meili_dir(exe_dir);
    let meili_path = meiliguard::meili_path(meili_dir);

    let mut command = std::process::Command::new(meili_path);
    command.args(std::env::args().skip_while(|s| s != "--").skip(1));

    unsafe {
        inner::MEILI
            .set(inner::UnwindGuard(command.spawn().unwrap()))
            .unwrap();
    }

    std::thread::sleep(std::time::Duration::MAX);
}

#[cfg(windows)]
fn main() {}
