use color_eyre::Result;

use crate::db::model;

use super::{Monitor, MonitorData};

impl Monitor {
    pub fn local_stacks_stacker(
        &self,
        _service: &model::Service,
        _data: &mut MonitorData,
    ) -> Result<()> {
        todo!()
    }
}
