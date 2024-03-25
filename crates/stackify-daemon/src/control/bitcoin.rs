use bitcoincore_rpc::{Auth, Client, RpcApi};
use color_eyre::{eyre::{bail, eyre}, Result};
use log::warn;
use reqwest::Url;
use stackify_common::ServiceState;
use tokio::process::Command;

use crate::db::model::{self, Service};

use super::{Monitor, MonitorContext, MonitorData};

impl Monitor {
    pub async fn local_bitcoin_miner(&self, ctx: &mut MonitorContext, service: &model::Service, data: &mut MonitorData) -> Result<()> {
        if data.child.is_none() && data.expected_state == ServiceState::Running {
            warn!("Local Bitcoin miner is expected to be running, but no child process is running. Starting...");

            // Call the start-bitcoind-miner.sh script to start the miner.
            let child = Command::new("/home/stacks/start-bitcoind-miner.sh")
                .spawn()?;

            // We didn't get an error, so the process should be running. Set the state to running.
            ctx.db.set_service_state(service.id, ServiceState::Running as i32)?;

            data.child = Some(child);
            data.last_state = ServiceState::Running;
        } else if data.child.is_some() && data.expected_state == ServiceState::Stopped {
            warn!("Local Bitcoin miner is expected to be stopped, but a child process is running. Stopping...");

            // Kill the child process.
            let child = data.child.as_mut().unwrap();
            child.kill().await?;

            // We didn't get an error, so the process should be stopped. Set the state to stopped.
            ctx.db.set_service_state(service.id, ServiceState::Stopped as i32)?;

            data.child = None;
            data.last_state = ServiceState::Stopped;
        }

        Ok(())
    }

    pub fn local_bitcoin_follower(&self, _service: &model::Service, _data: &mut MonitorData) -> Result<()> {
        todo!()
    }

    pub fn remote_bitcoin_node(&self, service: &model::Service, _data: &mut MonitorData) -> Result<()> {
        let host = service.host.clone()
            .ok_or(eyre!("Service host is not set"))?;
        let port = service.rpc_port
            .ok_or(eyre!("Service RPC port is not set"))?;
        let url = Url::parse(&format!("http://{}:{}/", host, port))?;
        let username = service.rpc_username.clone()
            .ok_or(eyre!("Service username is not set"))?;
        let password = service.rpc_password.clone()
            .ok_or(eyre!("Service password is not set"))?;

        if bitcoin_ping(&host, port as u16, &username, &password) {
            return Ok(())
        } else {
            bail!("Failed to ping remote Bitcoin node at {}", url.to_string());
        }
    }
}

fn bitcoin_ping(host: &str, port: u16, username: &str, password: &str) -> bool {
    let url = match Url::parse(&format!("http://{}:{}/", host, port)) {
        Ok(url) => url,
        Err(_) => { return false; }
    };

    match Client::new(
        &url.to_string(), 
        Auth::UserPass(username.to_string(), password.to_string())
    ) {
        Ok(client) => {
            match client.ping() {
                Ok(_) => true,
                Err(_) => false
            }
        },
        Err(_) => false
    }
}