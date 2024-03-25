use color_eyre::Result;

use crate::db::model;

use super::{Monitor, MonitorData};

impl Monitor {
    pub fn local_stacks_miner(
        &self,
        _service: &model::Service,
        _data: &mut MonitorData,
    ) -> Result<()> {
        todo!()
    }

    pub fn local_stacks_follower(
        &self,
        _service: &model::Service,
        _data: &mut MonitorData,
    ) -> Result<()> {
        todo!()
    }

    pub fn remote_stacks_node(
        &self,
        _service: &model::Service,
        _data: &mut MonitorData,
    ) -> Result<()> {
        todo!()
    }
}
