#![feature(exit_status_error)]

use bevy::prelude::*;
use pyo3::prelude::*;

pub struct FurAffinityPlugin;

impl Plugin for FurAffinityPlugin {
    fn build(&self, _app: &mut App) {
        pyo3::prepare_freethreaded_python();

        let exe_path = std::env::current_exe().unwrap();
        let exe_dir = exe_path.parent().unwrap();

        let mut venv_dir = exe_dir.join("python/");

        // Python dies at the thought of \\?\, 
        // so we have to strip this here manually
        if let std::path::Component::Prefix(prefix) = 
            venv_dir.components().next().unwrap() {
            if prefix.kind().is_verbatim() {
                let literal = venv_dir.to_str().unwrap().to_owned();
                venv_dir.push(&literal[4..]);
            }
        }

        if !venv_dir.exists() {
            create_venv(&venv_dir);
        }

        #[cfg(windows)]
        let packages_dir = venv_dir.join("Lib")
            .join("site-packages");

        #[cfg(unix)]
        let packages_dir = venv_dir.join("lib").read_dir()
            .unwrap()
            .flatten()
            .find(|entry| entry.file_type().unwrap().is_dir())
            .unwrap()
            .path()
            .join("site-packages");

        let io = std::fs::File::create(
                exe_dir.join("python.log")).unwrap();

        update_faapi(&venv_dir, &io);

        Python::with_gil(|py| {
            let sys = py.import("sys").unwrap();

            let syspath: &pyo3::types::PyList = pyo3::PyTryInto::try_into(
                sys.getattr("path").unwrap()).unwrap();
            syspath.insert(0, packages_dir).unwrap();

            // https://github.com/python/cpython/issues/100171
            let mut dllpath = std::path::PathBuf::from(
                sys.getattr("base_prefix").unwrap().to_string());
            dllpath.push("DLLs");
            syspath.insert(1, dllpath).unwrap();

            let _faapi = py.import("faapi").unwrap();
        });
    }
}

fn create_venv(venv_dir: &std::path::Path) {
    Python::with_gil(|py| {
        py.import("venv")
            .unwrap()
            .getattr("create")
            .unwrap()
            .call1((venv_dir.to_str().unwrap(),false,false,false,true))
            .unwrap();
    });
}

fn update_faapi(venv_dir: &std::path::Path, io: &std::fs::File) {
    info!("Updating FAAPI...");

    #[cfg(unix)]
    let pip_path = venv_dir.join("bin").join("pip3");

    #[cfg(windows)]
    let pip_path = venv_dir.join("Scripts").join("pip3.exe");

    std::process::Command::new(pip_path)
        .args([
            "--no-input",
            "install",
            "-U",
            "faapi"
        ])
        .stdout(std::process::Stdio::from(io.try_clone().unwrap()))
        .stderr(std::process::Stdio::from(io.try_clone().unwrap()))
        .spawn()
        .expect("Failed to update FAAPI: Could not spawn pip3")
        .wait()
        .unwrap()
        .exit_ok()
        .expect("Failed to update FAAPI: pip3 returned an error");
}