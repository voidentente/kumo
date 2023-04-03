use bevy::prelude::*;

pub struct MeiliPlugin;

impl Plugin for MeiliPlugin {
    fn build(&self, _app: &mut App) {
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
        let mut meili_version = String::from_utf8(response.stdout).unwrap();
        if meili_version.ends_with('\n') {
            meili_version.pop();
        }
        info!("Running {meili_version}");

        let exe_path = std::env::current_exe().unwrap();
        let exe_dir = exe_path.parent().unwrap();

        let out = std::fs::File::create(exe_dir.join("meilisearch.log")).unwrap();

        #[cfg(windows)] {
            // On Windows, Meilisearch is a direct child of Kumo.
            // It is a JobObject that NT will terminate if Kumo terminates.

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
            command.stdout(std::process::Stdio::from(out.try_clone().unwrap()));
            command.stderr(std::process::Stdio::from(out));

            let meili = command.spawn().unwrap();
            info!("Now running on http://localhost:11212");

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
        }

        #[cfg(unix)] {
            // On Unix, Meilisearch is run by a guard process.
            // This way, the minimal guard continues to live after Kumo exits.
            // and can ensure that Meilisearch terminates on Kumo termination.

            let mut guard_path = meili_dir.join("meiliguard");

            let mut command = std::process::Command::new(&meili_path);
            command.arg("--no-analytics");
            command.arg("--http-addr=localhost:11212");
            command.arg(format!("--master-key={master_key}"));
            command.arg(format!("--db-path={}", db_path.display()));
            command.arg(format!("--dump-dir={}", dump_dir.display()));

            todo!()
        }

        /*
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

         */
    }
}
