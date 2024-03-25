use color_eyre::Result;
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

    pub fn remote_bitcoin_node(&self, _service: &model::Service, _data: &mut MonitorData) -> Result<()> {
        todo!()
    }
}