use color_eyre::Result;

use crate::db::model;

use super::{Monitor, MonitorData};

impl Monitor {
    pub fn local_stacks_signer(
        &self,
        _service: &model::Service,
        _data: &mut MonitorData,
    ) -> Result<()> {
        todo!()
    }

    pub fn remote_stacks_signer(
        &self,
        _service: &model::Service,
        _data: &mut MonitorData,
    ) -> Result<()> {
        todo!()
    }
}
