use std::{path::PathBuf, time::Duration};

use color_eyre::Result;
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
    pub cancellation_token: CancellationToken,
}

impl CliContext {
    pub async fn new(
        host_dirs: StackifyHostDirs,
        docker_api: DockerApi,
        tx: tokio::sync::broadcast::Sender<()>,
    ) -> Result<Self> {
        let db_file = host_dirs.app_root.join("stackify.db");
        let db_conn = SqliteConnection::establish(&db_file.to_string_lossy()).unwrap();
        let db = AppDb::new(db_conn);

        let cancellation_token = CancellationToken::new();

        unsafe {
            Ok(Self {
                host_dirs,
                db_file,
                db,
                user_id: libc::getuid(),
                group_id: libc::getgid(),
                docker_api,
                tx: Some(tx),
                cancellation_token,
            })
        }
    }

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
        let mut receiver = self.tx.clone().unwrap().subscribe();
        let docker = self.docker_api.clone();
        let token = self.cancellation_token.clone();
        tokio::spawn(async move {
            loop {
                if receiver.recv().await.is_ok() {
                    token.cancel();
                    f(docker).await;
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    }
}
