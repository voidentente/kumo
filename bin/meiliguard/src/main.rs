//! This guard wraps around the meilisearch process to guarantee its termination,
//! even when the parent child is not unwinded but non-gracefully aborted.
//! Instead of spawning meilisearch directly, spawn meiliguard!

use std::ffi::{c_void};
use std::thread;
use std::time::Duration;

use libc::{prctl, PR_SET_PDEATHSIG, STDOUT_FILENO, write};
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, SIGUSR1};
use nix::unistd::{getpid, getppid, Pid};

extern "C" fn handle_sigusr1(_: libc::c_int) {
    print_signal_safe("[sleepy] Parent died!\n");
}

fn print_signal_safe(s: &str) {
    unsafe {
        write(STDOUT_FILENO, s.as_ptr() as (* const c_void), libc::c_uint::from(s.len()));
    }
}

fn main() {
    println!("Hello, world!");
}
