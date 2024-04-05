use std::{path::PathBuf, sync::Arc, time::Duration};

use diesel::{Connection, SqliteConnection};
use futures_util::Future;
use tokio_util::sync::CancellationToken;

use crate::{
    db::{cli_db::CliDatabase, AppDb},
    docker::api::DockerApi,
};

use super::StackifyHostDirs;

pub struct CliContext {
    pub host_dirs: StackifyHostDirs,
    /// The database file for Stackify. Defaults to `$HOME/.stackify/stackify.db`
    pub db_file: PathBuf,
    /// Instance of Stackify's application database.
    pub db: AppDb,
    /// The user id of the current user.
    pub user_id: u32,
    /// The group id of the current user.
    pub group_id: u32,
    /// Instance of Stackify's Docker client.
    //pub docker: StackifyDocker,
    pub docker_api: DockerApi,
    pub tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl Default for CliContext {
    fn default() -> Self {
        let uid;
        let gid;
        unsafe {
            uid = libc::geteuid();
            gid = libc::getegid();
        }

        let host_dirs = StackifyHostDirs::default();

        let db_file = host_dirs.app_root.join("stackify.db");
        let db_conn = SqliteConnection::establish(&db_file.to_string_lossy()).unwrap();
        Self {
            host_dirs,
            db_file,
            db: AppDb::new(db_conn),
            user_id: uid,
            group_id: gid,
            docker_api: DockerApi::default(),
            tx: None,
        }
    }
}

impl CliContext {
    pub fn clidb(&self) -> &impl CliDatabase {
        self.db.as_clidb()
    }

    pub fn docker(&self) -> &DockerApi {
        &self.docker_api
    }

    pub async fn register_shutdown<F, Fut>(&self, f: F)
    where
        F: Send + Sync + 'static + FnOnce(DockerApi) -> Fut,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        eprintln!("Registering shutdown hook");
        let mut receiver = self.tx.clone().unwrap().subscribe();
        let docker = self.docker_api.clone();
        tokio::spawn(async move {
            loop {
                println!("loop");
                if receiver.recv().await.is_ok() {
                    f(docker).await;
                    break;
                }
                std::thread::sleep(Duration::from_millis(500));
            }
        });
    }
}
