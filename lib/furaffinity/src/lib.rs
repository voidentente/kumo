#![feature(exit_status_error)]

use bevy::prelude::*;
use pyo3::prelude::*;

pub struct FurAffinityPlugin;

impl Plugin for FurAffinityPlugin {
    fn build(&self, _app: &mut App) {
        let exe_path = std::env::current_exe().unwrap();
        let exe_dir = exe_path.parent().unwrap();

        let mut venv_dir = exe_dir.join("python/");

        // Python dies at the thought of \\?\, so we have to strip this here manually
        if let std::path::Component::Prefix(prefix) = venv_dir.components().next().unwrap() {
            if prefix.kind().is_verbatim() {
                let literal = venv_dir.to_str().unwrap().to_owned();
                venv_dir.push(&literal[4..]);
            }
        }

        let io = std::fs::File::create(exe_dir.join("faapi.log")).unwrap();

        if !venv_dir.exists() {
            create_venv(&venv_dir, &io);
        }

        let packages_dir = venv_dir.join("Lib").join("site-packages");
        let faapi_dir = packages_dir.join("faapi");

        if faapi_dir.exists() {
            update_faapi(&venv_dir, &io);
        } else {
            install_faapi(&venv_dir, &io);
        }

        pyo3::prepare_freethreaded_python();

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

#[cfg(windows)]
fn create_venv(venv_dir: &std::path::Path, io: &std::fs::File) {
    info!("Creating venv...");
    std::process::Command::new("py")
        .args([
            "-3",
            "-m",
            "venv",
            venv_dir.to_str().unwrap()
        ])
        .stdout(std::process::Stdio::from(io.try_clone().unwrap()))
        .stderr(std::process::Stdio::from(io.try_clone().unwrap()))
        .spawn()
        .expect("Failed to create venv: Could not spawn py")
        .wait()
        .unwrap()
        .exit_ok()
        .expect("Failed to create venv: Py returned an error");
}

#[cfg(unix)]
fn create_venv() {
    todo!()
}

#[cfg(windows)]
fn install_faapi(venv_dir: &std::path::Path, io: &std::fs::File) {
    info!("Installing FAAPI...");
    std::process::Command::new(venv_dir.join("Scripts/pip3.exe"))
        .args([
            "--no-input",
            "install",
            "faapi"
        ])
        .stdout(std::process::Stdio::from(io.try_clone().unwrap()))
        .stderr(std::process::Stdio::from(io.try_clone().unwrap()))
        .spawn()
        .expect("Failed to install FAAPI: Could not spawn pip3")
        .wait()
        .unwrap()
        .exit_ok()
        .expect("Failed to install FAAPI: pip3 returned an error");
}

#[cfg(unix)]
fn install_faapi() {
    todo!()
}

#[cfg(windows)]
fn update_faapi(venv_dir: &std::path::Path, io: &std::fs::File) {
    info!("Updating FAAPI...");
    std::process::Command::new(venv_dir.join("Scripts/pip3.exe"))
        .args([
            "--no-input",
            "install",
            "faapi",
            "-U"
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

#[cfg(unix)]
fn update_faapi() {
    todo!()
}