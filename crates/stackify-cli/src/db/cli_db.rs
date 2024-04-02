use color_eyre::{eyre::eyre, Result};

use ::diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use stackify_common::types::{self, EnvironmentName};

use super::{diesel::model, diesel::schema::*, AppDb};

pub trait CliDatabase {
    fn load_all_environments(&self) -> Result<Vec<types::Environment>>;
    fn load_environment(&self, name: &str) -> Result<Option<types::Environment>>;
    fn load_all_service_types(&self) -> Result<Vec<types::ServiceTypeFull>>;
}

impl AppDb {
    pub fn as_clidb(&self) -> &impl CliDatabase {
        self
    }
}

impl CliDatabase for AppDb {
    fn load_all_environments(&self) -> Result<Vec<types::Environment>> {
        let environments =
            environment::table.load::<model::Environment>(&mut *self.conn.borrow_mut())?;

        environments
            .into_iter()
            .map(|env| Ok(self.load_environment(&env.name)?.unwrap()))
            .collect::<Result<Vec<_>>>()
    }

    /// Loads an environment by name.
    fn load_environment(&self, name: &str) -> Result<Option<types::Environment>> {
        let env_name = EnvironmentName::new(name)?;

        let env = environment::table
            .filter(environment::name.eq(name))
            .first::<model::Environment>(&mut *self.conn.borrow_mut())
            .optional()?;

        if !env.is_some() {
            return Ok(None);
        }

        let service_types =
            service_type::table.load::<model::ServiceType>(&mut *self.conn.borrow_mut())?;

        let epochs = epoch::table
            .load::<model::Epoch>(&mut *self.conn.borrow_mut())?
            .into_iter()
            .map(|e| types::Epoch {
                id: e.id,
                default_block_height: e.default_block_height as u32,
                name: e.name,
            })
            .collect::<Vec<_>>();

        let service_type_versions =
            service_version::table.load::<model::ServiceVersion>(&mut *self.conn.borrow_mut())?;

        let env_epochs = environment_epoch::table
            .load::<model::EnvironmentEpoch>(&mut *self.conn.borrow_mut())?;

        let env_services = environment_service::table
            .load::<model::EnvironmentService>(&mut *self.conn.borrow_mut())?;

        let services = env_services
            .iter()
            .map(|es| {
                let db_service_version = service_type_versions
                    .iter()
                    .find(|sv| sv.id == es.service_version_id)
                    .expect("Service version not found");

                let min_epoch = if let Some(min_epoch) = db_service_version.minimum_epoch_id {
                    epochs.iter().find(|e| e.id == min_epoch).cloned()
                } else {
                    None
                };

                let max_epoch = if let Some(max_epoch) = db_service_version.maximum_epoch_id {
                    epochs.iter().find(|e| e.id == max_epoch).cloned()
                } else {
                    None
                };

                let service_type = types::ServiceTypeSimple {
                    id: db_service_version.service_type_id,
                    name: service_types
                        .iter()
                        .find(|st| st.id == db_service_version.service_type_id)
                        .expect("Service type not found")
                        .name
                        .clone(),
                    cli_name: service_types
                        .iter()
                        .find(|st| st.id == db_service_version.service_type_id)
                        .expect("Service type not found")
                        .cli_name
                        .clone(),
                };

                let git_target =
                    types::GitTarget::parse_opt(db_service_version.git_target.as_ref());

                let service = types::EnvironmentService {
                    id: es.id,
                    service_type,
                    version: types::ServiceVersion {
                        id: db_service_version.id,
                        version: db_service_version.version.clone(),
                        min_epoch,
                        max_epoch,
                        git_target,
                    },
                    name: es.name.clone(),
                    remark: es.comment.clone(),
                };

                Ok(service)
            })
            .collect::<Result<Vec<_>>>()?;

        let epochs = env_epochs
            .iter()
            .map(|ee| {
                let epoch = epochs
                    .iter()
                    .find(|e| e.id == ee.epoch_id)
                    .cloned()
                    .ok_or(eyre!("Epoch not found"))?;

                Ok(types::EnvironmentEpoch {
                    id: ee.id,
                    epoch,
                    starts_at_block_height: ee.starts_at_block_height as u32,
                    ends_at_block_height: ee.ends_at_block_height.map(|h| h as u32),
                })
            })
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        let ret = types::Environment {
            name: env_name,
            services,
            epochs,
        };

        Ok(Some(ret))
    }

    fn load_all_service_types(&self) -> Result<Vec<types::ServiceTypeFull>> {
        let service_types =
            service_type::table.load::<model::ServiceType>(&mut *self.conn.borrow_mut())?;

        let service_type_versions =
            service_version::table.load::<model::ServiceVersion>(&mut *self.conn.borrow_mut())?;

        let epochs = epoch::table
            .load::<model::Epoch>(&mut *self.conn.borrow_mut())?
            .into_iter()
            .map(|e| types::Epoch {
                id: e.id,
                default_block_height: e.default_block_height as u32,
                name: e.name,
            })
            .collect::<Vec<_>>();

        let services = service_types
            .iter()
            .map(|service_type| {
                let service_versions = service_type_versions
                    .iter()
                    .filter(|sv| sv.service_type_id == service_type.id)
                    .map(|sv| {
                        let min_epoch = if let Some(min_epoch) = sv.minimum_epoch_id {
                            epochs.iter().find(|e| e.id == min_epoch).cloned()
                        } else {
                            None
                        };

                        let max_epoch = if let Some(max_epoch) = sv.maximum_epoch_id {
                            epochs.iter().find(|e| e.id == max_epoch).cloned()
                        } else {
                            None
                        };

                        types::ServiceVersion {
                            id: sv.id,
                            version: sv.version.clone(),
                            min_epoch,
                            max_epoch,
                            git_target: types::GitTarget::parse_opt(sv.git_target.as_ref()),
                        }
                    });

                let ret = types::ServiceTypeFull {
                    id: service_type.id,
                    name: service_type.name.clone(),
                    cli_name: service_type.cli_name.clone(),
                    allow_git_target: service_type.allow_git_target,
                    allow_max_epoch: service_type.allow_maximum_epoch,
                    allow_min_epoch: service_type.allow_minimum_epoch,
                    versions: service_versions.collect(),
                };

                Ok(ret)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(services)
    }
}
