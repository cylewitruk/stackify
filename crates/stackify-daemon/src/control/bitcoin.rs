use bitcoincore_rpc::{Auth, Client, RpcApi};
use color_eyre::{eyre::{bail, eyre}, Result};
use reqwest::Url;
use stackify_common::ServiceState;
use tokio::process::Command;

use crate::db::model;

use super::{Monitor, MonitorData};

impl Monitor {
    pub fn local_bitcoin_miner(&self, service: &model::Service, data: &mut MonitorData) -> Result<()> {
        if data.child.is_none() && data.expected_state == ServiceState::Running {
            self.db.insert_log_entry(
                service.service_type_id, 
                "INFO", 
                "Local Bitcoin miner node is not running. Attempting to start..."
            )?;

            let child = Command::new("/home/stacks/start-bitcoind-miner.sh")
                .spawn()?;
            data.child = Some(child);
            data.last_state = ServiceState::Running;
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