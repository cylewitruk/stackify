use crate::db::diesel::model::{Epoch, ServiceType, ServiceUpgradePath, ServiceVersion};

pub mod git;
pub mod names;
pub mod print;
pub mod stacks_cli;

pub trait FindById<T> {
    fn find_by_id(&self, id: i32) -> Option<&T>;
    fn find_by_id_opt(&self, id: Option<i32>) -> Option<&T> {
        if let Some(id) = id {
            self.find_by_id(id)
        } else {
            None
        }
    }
}

pub trait FindByCliName<T> {
    fn find_by_cli_name(&self, cli_name: &str) -> Option<&T>;
}

pub trait FilterByServiceType<T> {
    fn filter_by_service_type(&self, service_type_id: i32) -> Vec<&T>;
}

pub trait FilterByServiceVersion<T> {
    fn filter_by_service_version(&self, service_version_id: i32) -> Vec<&T>;
}

pub trait FilterByMinMaxEpoch<T> {
    fn filter_by_max_epoch(&self, epoch_id: i32) -> Vec<&T>;
    fn filter_by_min_epoch(&self, epoch_id: i32) -> Vec<&T>;
}

pub trait FilterByEpoch<T> {
    fn filter_by_epoch(&self, epoch_id: i32) -> Vec<&T>;
}

impl FilterByMinMaxEpoch<ServiceVersion> for Vec<ServiceVersion> {
    fn filter_by_max_epoch(&self, epoch_id: i32) -> Vec<&ServiceVersion> {
        self.iter()
            .filter(|v| v.maximum_epoch_id == Some(epoch_id))
            .collect()
    }

    fn filter_by_min_epoch(&self, epoch_id: i32) -> Vec<&ServiceVersion> {
        self.iter()
            .filter(|v| v.minimum_epoch_id == Some(epoch_id))
            .collect()
    }
}

impl FilterByMinMaxEpoch<ServiceUpgradePath> for Vec<ServiceUpgradePath> {
    fn filter_by_max_epoch(&self, epoch_id: i32) -> Vec<&ServiceUpgradePath> {
        self.iter()
            .filter(|p| p.maximum_epoch_id == Some(epoch_id))
            .collect()
    }

    fn filter_by_min_epoch(&self, epoch_id: i32) -> Vec<&ServiceUpgradePath> {
        self.iter()
            .filter(|p| Some(p.minimum_epoch_id) == Some(epoch_id))
            .collect()
    }
}

impl FindByCliName<ServiceVersion> for Vec<ServiceVersion> {
    fn find_by_cli_name(&self, cli_name: &str) -> Option<&ServiceVersion> {
        self.iter().find(|v| v.cli_name == cli_name)
    }
}

impl FilterByServiceType<ServiceVersion> for Vec<ServiceVersion> {
    fn filter_by_service_type(&self, service_type_id: i32) -> Vec<&ServiceVersion> {
        self.iter()
            .filter(|v| v.service_type_id == service_type_id)
            .collect()
    }
}

impl FilterByServiceVersion<ServiceUpgradePath> for &[ServiceUpgradePath] {
    fn filter_by_service_version(&self, service_version_id: i32) -> Vec<&ServiceUpgradePath> {
        self.iter()
            .filter(|p| p.from_service_version_id == service_version_id)
            .collect()
    }
}

impl FindById<ServiceType> for Vec<ServiceType> {
    fn find_by_id(&self, id: i32) -> Option<&ServiceType> {
        self.iter().find(|t| t.id == id)
    }
}

impl FindById<ServiceType> for &[ServiceType] {
    fn find_by_id(&self, id: i32) -> Option<&ServiceType> {
        self.iter().find(|t| t.id == id)
    }
}

impl FindById<Epoch> for Vec<Epoch> {
    fn find_by_id(&self, id: i32) -> Option<&Epoch> {
        self.iter().find(|t| t.id == id)
    }
}

impl FindById<ServiceVersion> for Vec<ServiceVersion> {
    fn find_by_id(&self, id: i32) -> Option<&ServiceVersion> {
        self.iter().find(|t| t.id == id)
    }
}

impl FindById<ServiceVersion> for &[ServiceVersion] {
    fn find_by_id(&self, id: i32) -> Option<&ServiceVersion> {
        self.iter().find(|t| t.id == id)
    }
}
