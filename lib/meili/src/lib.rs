use bevy::prelude::*;

/// This guard ensures that Meilisearch is terminated when the guard is dropped.
/// This works on rewind, but not on abort, in which case Meilisearch might continue as orphan.
#[derive(Resource)]
struct MeiliGuard(std::process::Child);

impl Drop for MeiliGuard {
    fn drop(&mut self) {
        info!("Killing Meili");
        #[allow(unused_must_use)] {
            self.0.kill();
        }
    }
}

pub struct MeiliPlugin;

impl Plugin for MeiliPlugin {
    fn build(&self, app: &mut App) {
        let meili_dir = std::env::args().find(|s| s.starts_with("--meili="))
            .map(|s| std::path::PathBuf::from(s.split_once('=').unwrap().1))
            .unwrap_or(std::env::current_exe().unwrap().parent().unwrap().to_owned());

        let mut meili_path = meili_dir.join("meilisearch");
        meili_path.set_extension(std::env::consts::EXE_EXTENSION);

        if !meili_path.exists() || !meili_path.is_file() {
            panic!("Meilisearch executable not found. Searched at {}", meili_path.display());
        }

        let response = std::process::Command::new(&meili_path).arg("-V")
            .stdout(std::process::Stdio::piped()).output().unwrap();
        let meili_version = String::from_utf8_lossy(&response.stdout);
        info!("Running {meili_version}");

        let db_path = meili_dir.join("data.ms");
        let dump_dir = meili_dir.join("dumps/");

        let mut buf = ['\0'; 16];
        eprng::digit_chars(&mut buf, eprng::initial_offset(), 10);
        let master_key = buf.iter().cloned().collect::<String>();

        let mut command = std::process::Command::new(&meili_path);
        command.arg("--no-analytics");
        command.arg("--http-addr=localhost:11212");
        command.arg(format!("--master-key={master_key}"));
        command.arg(format!("--db-path={}", db_path.display()));
        command.arg(format!("--dump-dir={}", dump_dir.display()));

        let exe_path = std::env::current_exe().unwrap();
        let exe_dir = exe_path.parent().unwrap();
        let fd = std::fs::File::create(exe_dir.join("meilisearch.log")).unwrap();
        command.stdout(std::process::Stdio::from(fd.try_clone().unwrap()));
        command.stderr(std::process::Stdio::from(fd));
        info!("{:?}", command);

        let meili = command.spawn().unwrap();
        app.insert_resource(MeiliGuard(meili));

        #[cfg(debug_assertions)]
        if let Err(e) = webbrowser::open("localhost:11212") {
            warn!("Failed to open Meilisearch web interface: {e}");
        }

        // Tests.

        #[derive(serde::Serialize, serde::Deserialize, Debug)]
        struct Movie {
            id: usize,
            title: String,
        }

        futures::executor::block_on(async {
            let client =
                meilisearch_sdk::Client::new("http://localhost:11212", master_key);

            let health = client.health().await.unwrap();
            info!("Meili Health: {}", health.status);

            /*
            let movies = client.index("movies");

            movies.add_documents(&[
                Movie { id: 1, title: String::from("Bridges") },
                Movie { id: 2, title: String::from("Pengo's Adventure") },
                Movie { id: 3, title: String::from("PengoSolvent") }
            ], Some("id")).await.unwrap();

            println!("{:?}", client.index("movies").search().with_query("pengo").execute::<Movie>().await.unwrap().hits);
             */
        });
    }
}
