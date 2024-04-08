use color_eyre::{
    eyre::{bail, eyre},
    Result,
};

use ::diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use stackify_common::{
    types::{self, EnvironmentName},
    FileType, ServiceType,
};

use crate::db::errors::LoadEnvironmentError;

use super::{
    diesel::{model, schema::*},
    AppDb,
};

pub trait CliDatabase {
    fn load_all_environments(&self) -> Result<Vec<types::Environment>>;
    fn load_environment(
        &self,
        name: &str,
    ) -> std::result::Result<types::Environment, LoadEnvironmentError>;
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
            .map(|env| Ok(self.load_environment(&env.name)?))
            .collect::<Result<Vec<_>>>()
    }

    /// Loads an environment by name.
    /// TODO: Really ugly, refactor later
    fn load_environment(
        &self,
        name: &str,
    ) -> std::result::Result<types::Environment, LoadEnvironmentError> {
        let env_name = EnvironmentName::new(name)?;

        let env = environment::table
            .filter(environment::name.eq(name))
            .first::<model::Environment>(&mut *self.conn.borrow_mut())
            .optional()?;

        if !env.is_some() {
            return Err(LoadEnvironmentError::NotFound);
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
            .filter(environment_epoch::environment_id.eq(env.as_ref().unwrap().id))
            .load::<model::EnvironmentEpoch>(&mut *self.conn.borrow_mut())?;

        let env_services = environment_service::table
            .load::<model::EnvironmentService>(&mut *self.conn.borrow_mut())?;

        let services = env_services
            .iter()
            .map(|es| {
                // Service versions
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

                // Service files
                let mut file_headers = Vec::<types::EnvironmentServiceFileHeader>::new();

                let service_type_files = service_type_file::table
                    .filter(
                        service_type_file::service_type_id.eq(db_service_version.service_type_id),
                    )
                    .load::<model::ServiceTypeFile>(&mut *self.conn.borrow_mut())?;

                for st_file in service_type_files {
                    let file = types::EnvironmentServiceFileHeader {
                        id: st_file.id,
                        filename: st_file.filename.clone(),
                        destination_dir: st_file.destination_dir.clone().into(),
                        description: st_file.description.clone(),
                        file_type: FileType::from_i32(st_file.file_type_id)
                            .expect("File type not found"),
                        service_type: ServiceType::from_i32(st_file.service_type_id)
                            .expect("Service type not found"),
                    };

                    file_headers.push(file);
                }

                // Service params
                let mut params = Vec::<types::EnvironmentServiceParam>::new();

                let service_type_params = service_type_param::table
                    .filter(
                        service_type_param::service_type_id.eq(db_service_version.service_type_id),
                    )
                    .load::<model::ServiceTypeParam>(&mut *self.conn.borrow_mut())?;

                for st_param in service_type_params {
                    let allowed_values = st_param
                        .allowed_values
                        .clone()
                        .map(|av| av.split(',').map(|s| s.to_string()).collect::<Vec<_>>());

                    let env_value = environment_service_param::table
                        .filter(environment_service_param::environment_service_id.eq(es.id))
                        .filter(environment_service_param::service_type_param_id.eq(st_param.id))
                        .select(environment_service_param::value)
                        .first::<String>(&mut *self.conn.borrow_mut())
                        .optional()?;

                    if st_param.is_required && env_value.is_none() {
                        return Err(LoadEnvironmentError::MissingParam {
                            service_name: es.name.clone(),
                            param_name: st_param.name.clone(),
                        });
                    }

                    let param = types::EnvironmentServiceParam {
                        id: st_param.id,
                        param: types::ServiceTypeParam {
                            id: st_param.id,
                            name: st_param.name.clone(),
                            key: st_param.key.clone(),
                            description: st_param.description.clone(),
                            default_value: st_param.default_value.clone().unwrap_or_default(),
                            is_required: st_param.is_required,
                            value_type: stackify_common::ValueType::from_i32(
                                st_param.value_type_id,
                            )?,
                            allowed_values,
                            service_type: service_type.clone(),
                        },
                        value: "".to_string(),
                    };

                    params.push(param);
                }

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
                    file_headers,
                    params,
                };

                Ok(service)
            })
            .collect::<Result<Vec<_>, LoadEnvironmentError>>()?;

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

        Ok(ret)
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
