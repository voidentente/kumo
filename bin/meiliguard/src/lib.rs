//! Contains the guard logic for meilisearch.
//! On Windows, Meili is directly executed by Kumo.
//! On Unix, Meili is executed by a thin wrapper process (main.rs).

use bevy::prelude::*;
use meilisearch_sdk::Client;

#[derive(Resource)]
pub struct Meilisearch(pub Client);

impl Default for Meilisearch {
    fn default() -> Self {
        Self(start())
    }
}

pub struct MeilisearchPlugin;

impl Plugin for MeilisearchPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Meilisearch>();
    }
}

const ADDR: &str = "localhost:11212";

const HOST: &str = "http://localhost:11212";

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Returns the meilisearch operating directory.<br>
/// This directory contains the meilisearch executable,<br>
/// the meilisearch guard executable, and the database.<br>
/// This is path/to/exe by default, or whichever path was<br>
/// provided through `--meili={path}`.
pub fn meili_dir(exe_dir: impl Into<PathBuf>) -> PathBuf {
    std::env::args()
        .find(|s| s.starts_with("--meili="))
        .map(|s| PathBuf::from(s.split_once('=').unwrap().1))
        .unwrap_or(exe_dir.into())
}

/// Returns the path to the meilisearch executable.
pub fn meili_path(meili_dir: impl AsRef<Path>) -> PathBuf {
    meili_dir
        .as_ref()
        .join("meilisearch")
        .with_extension(std::env::consts::EXE_EXTENSION)
}

/// Returns the path to the meiliguard executable.<br>
/// This is only available on Unix.
#[cfg(unix)]
fn guard_path(meili_dir: impl AsRef<Path>) -> PathBuf {
    meili_dir.as_ref().join("meiliguard")
}

/// Starts meilisearch with the Unix-guard.<br>
/// Works by spawning a thin wrapper that spawns meilisearch.<br>
/// This wrapper is inherited by init if Kumo exits and terminates<br>
/// meilisearch and then itself by listening to PDEATHSIG.
#[cfg(unix)]
fn start() -> Client {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();

    let meili_dir = meili_dir(exe_dir);
    let guard_path = guard_path(&meili_dir);

    let out_path = meili_dir.join("meilisearch.log");
    let out = std::fs::File::create(out_path).unwrap();

    let db_path = meili_dir.join("data.ms");
    let dump_dir = meili_dir.join("dumps/");

    let mut command = Command::new(guard_path);
    command.arg(format!("--meili={}", meili_dir.display()));
    command.arg("--");
    command.arg("--no-analytics");
    command.arg(format!("--http-addr={}", ADDR));
    command.arg(format!("--db-path={}", db_path.display()));
    command.arg(format!("--dump-dir={}", dump_dir.display()));
    command.stdout(Stdio::from(out.try_clone().unwrap()));
    command.stderr(Stdio::from(out));
    command.spawn().unwrap();

    Client::new(HOST, None::<String>)
}

/// Starts meilisearch with the Windows-guard.<br>
/// Works by designating meilisearch as job object and<br>
/// setting the `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` flag.
#[cfg(windows)]
fn start() -> Client {
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();

    let meili_dir = meili_dir(exe_dir);
    let meili_path = meili_path(&meili_dir);

    let out_path = meili_dir.join("meilisearch.log");
    let out = std::fs::File::create(out_path).unwrap();

    let db_path = meili_dir.join("data.ms");
    let dump_dir = meili_dir.join("dumps/");

    let mut command = Command::new(meili_path);
    command.arg("--no-analytics");
    command.arg(format!("--http-addr={}", ADDR));
    command.arg(format!("--db-path={}", db_path.display()));
    command.arg(format!("--dump-dir={}", dump_dir.display()));
    command.stdout(Stdio::from(out.try_clone().unwrap()));
    command.stderr(Stdio::from(out));

    let meili = command.spawn().unwrap();

    unsafe {
        use std::os::windows::prelude::AsRawHandle;
        use windows::Win32::System::JobObjects::{self, JobObjectExtendedLimitInformation};

        let hjob = JobObjects::CreateJobObjectW(None, windows::w!("kumo_meili_job")).unwrap();

        assert!(!hjob.is_invalid());

        let hprocess = windows::Win32::Foundation::HANDLE(meili.as_raw_handle() as isize);

        let result = JobObjects::AssignProcessToJobObject(hjob, hprocess);

        assert!(result.as_bool());

        let mut eli = JobObjects::JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();

        eli.BasicLimitInformation.LimitFlags = JobObjects::JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        JobObjects::SetInformationJobObject(
            hjob,
            JobObjectExtendedLimitInformation,
            &eli as *const _ as *const _,
            std::mem::size_of_val(&eli) as _,
        );
    }

    Client::new(HOST, None::<String>)
}
