use std::collections::HashMap;

use color_eyre::{eyre::eyre, Result};

use ::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel::{JoinOnDsl, SelectableHelper};
use stackify_common::{
    types::{self, EnvironmentName, EnvironmentService},
    FileType, ServiceType,
};

use crate::{
    cli::{env, log::clilog},
    db::errors::LoadEnvironmentError,
};

use super::{
    diesel::{model, schema::*, InnerDb},
    AppDb,
};

pub trait CliDatabase {
    fn load_all_epochs(&self) -> Result<Vec<types::Epoch>>;
    fn load_all_environments(&self) -> Result<Vec<types::Environment>>;
    fn load_environment(
        &self,
        name: &str,
    ) -> std::result::Result<types::Environment, LoadEnvironmentError>;
    fn load_all_service_types(&self) -> Result<Vec<types::ServiceTypeFull>>;
    fn load_files_for_environment_service(
        &self,
        service: &EnvironmentService,
    ) -> Result<Vec<types::EnvironmentServiceFile>>;
}

impl AppDb {
    pub fn as_clidb(&self) -> &impl CliDatabase {
        self
    }
}

impl InnerDb for AppDb {}

impl CliDatabase for AppDb {
    fn load_files_for_environment_service(
        &self,
        service: &EnvironmentService,
    ) -> Result<Vec<types::EnvironmentServiceFile>> {
        let conn = &mut *self.conn.borrow_mut();
        let service_type_files = service_type_file::table
            .filter(service_type_file::service_type_id.eq(service.service_type.id))
            .load::<model::ServiceTypeFile>(conn)?;

        let env_service_files = environment_service_file::table
            .filter(environment_service_file::environment_service_id.eq(service.id))
            .load::<model::EnvironmentServiceFile>(conn)?;

        let mut files = Vec::<types::EnvironmentServiceFile>::new();
        for st_file in &service_type_files {
            let env_file = env_service_files
                .iter()
                .find(|esf| esf.service_type_file_id == st_file.id);

            let content = env_file
                .map(|esf| esf.contents.clone())
                .unwrap_or(st_file.default_contents.clone());

            files.push(types::EnvironmentServiceFile {
                header: types::EnvironmentServiceFileHeader {
                    id: st_file.id,
                    filename: st_file.filename.clone(),
                    destination_dir: st_file.destination_dir.clone().into(),
                    description: st_file.description.clone(),
                    file_type: FileType::from_i32(st_file.file_type_id)
                        .expect("File type not found"),
                    service_type: ServiceType::from_i32(st_file.service_type_id)
                        .expect("Service type not found"),
                },
                contents: types::ServiceFileContents { contents: content },
            })
        }

        Ok(files)
    }

    fn load_all_environments(&self) -> Result<Vec<types::Environment>> {
        let environments =
            environment::table.load::<model::Environment>(&mut *self.conn.borrow_mut())?;

        environments
            .into_iter()
            .map(|env| Ok(self.load_environment(&env.name)?))
            .collect::<Result<Vec<_>>>()
    }

    fn load_all_epochs(&self) -> Result<Vec<types::Epoch>> {
        let epochs = Self::load_epochs(&mut *self.conn.borrow_mut())?
            .into_iter()
            .map(|e| types::Epoch {
                id: e.id,
                name: e.name,
                default_block_height: e.default_block_height as u32,
            })
            .collect::<Vec<_>>();

        Ok(epochs)
    }

    /// Loads an environment by name.
    /// TODO: Really ugly, refactor later
    fn load_environment(
        &self,
        name: &str,
    ) -> std::result::Result<types::Environment, LoadEnvironmentError> {
        let env_name = EnvironmentName::new(name)?;
        let env = Self::find_environment_by_name(&mut *self.conn.borrow_mut(), name)?.ok_or(
            LoadEnvironmentError::NotFound {
                env_name: env_name.to_string(),
            },
        )?;

        let service_types = Self::load_service_types(&mut *self.conn.borrow_mut())?;
        let epochs = self.load_all_epochs()?;
        let service_type_versions = Self::load_service_type_versions(&mut *self.conn.borrow_mut())?;
        let env_epochs = Self::load_epochs_for_environment(&mut *self.conn.borrow_mut(), env.id)?;
        let env_services = environment_service::table
            .load::<model::EnvironmentService>(&mut *self.conn.borrow_mut())?;

        let services = env_services
            .iter()
            .map(|es| {
                // Service versions
                let db_service_version = service_type_versions
                    .iter()
                    .find(|sv| sv.id == es.service_version_id)
                    .ok_or(eyre!("Service version not found"))?;

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

                let db_service_type = service_types
                    .iter()
                    .find(|st| st.id == db_service_version.service_type_id)
                    .ok_or(eyre!("Service type not found"))?;

                let service_type = types::ServiceTypeSimple {
                    id: db_service_version.service_type_id,
                    name: db_service_type.name.clone(),
                    cli_name: db_service_type.cli_name.clone(),
                };

                let git_target =
                    types::GitTarget::parse_opt(db_service_version.git_target.as_ref());

                // Service files
                let mut file_headers = Vec::<types::EnvironmentServiceFileHeader>::new();

                let service_type_files = service_type_file::table
                    .filter(
                        service_type_file::service_type_id.eq(db_service_version.service_type_id),
                    )
                    .select(model::ServiceTypeFileHeader::as_select())
                    .load::<model::ServiceTypeFileHeader>(&mut *self.conn.borrow_mut())?;

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

                let service_type_params = Self::load_service_type_params_for_service_type(
                    &mut *self.conn.borrow_mut(),
                    db_service_version.service_type_id,
                )?;

                for st_param in service_type_params {
                    clilog!(
                        "Loading param for env {}: {}={}",
                        es.id,
                        st_param.id,
                        st_param.name
                    );
                    let allowed_values = st_param
                        .allowed_values
                        .clone()
                        .map(|av| av.split(',').map(str::to_owned).collect::<Vec<_>>());
                    clilog!("> Allowed values: {:?}", allowed_values);

                    clilog!("Finding param value: {}/{}", es.id, st_param.id);
                    let env_value = if let Some(param) = Self::find_environment_service_param(
                        &mut *self.conn.borrow_mut(),
                        es.id,
                        st_param.id,
                    )? {
                        clilog!("> Found param value: {}", param.value);
                        param.value
                    } else if let Some(default_value) = st_param.default_value.clone() {
                        clilog!("> Using default value: {}", default_value);
                        default_value
                    } else {
                        clilog!("> Missing param value");
                        return Err(LoadEnvironmentError::MissingParam {
                            service_name: es.name.clone(),
                            param_name: st_param.name.clone(),
                        });
                    };

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
                        value: env_value,
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
                        cli_name: db_service_type.cli_name.clone(),
                        rebuild_required: db_service_version.rebuild_required,
                        last_built_at: db_service_version.last_built_at,
                        last_build_commit_hash: db_service_version.last_build_commit_hash.clone(),
                    },
                    name: es.name.clone(),
                    remark: es.comment.clone(),
                    file_headers,
                    params,
                };

                Ok(service)
            })
            .collect::<Result<Vec<_>, LoadEnvironmentError>>()?;

        let epoch_map: HashMap<_, _> = epochs.iter().map(|e| (e.id, e.clone())).collect();
        let epochs = env_epochs
            .iter()
            .map(|ee| {
                let epoch = epoch_map
                    .get(&ee.epoch_id)
                    .ok_or(eyre!("Epoch not found"))?;

                Ok(types::EnvironmentEpoch {
                    id: ee.id,
                    epoch: epoch.clone(),
                    starts_at_block_height: ee.starts_at_block_height as u32,
                    ends_at_block_height: ee.ends_at_block_height.map(|h| h as u32),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let stacks_accounts = environment_stacks_account::table
            .inner_join(
                stacks_account::table
                    .on(environment_stacks_account::stacks_account_id.eq(stacks_account::id)),
            )
            .filter(environment_stacks_account::environment_id.eq(env.id))
            .load::<(model::EnvironmentStacksAccount, model::StacksAccount)>(
                &mut *self.conn.borrow_mut(),
            )?
            .iter()
            .map(|(esa, sa)| types::EnvironmentStacksAccount {
                id: esa.id,
                mnemonic: sa.mnemonic.clone(),
                address: sa.address.clone(),
                amount: sa.amount as u64,
                btc_address: sa.btc_address.clone(),
                private_key: sa.private_key.clone(),
                remark: esa.remark.clone(),
            })
            .collect::<Vec<_>>();

        let ret = types::Environment {
            id: env.id,
            name: env_name,
            services,
            epochs,
            stacks_accounts,
        };

        Ok(ret)
    }

    fn load_all_service_types(&self) -> Result<Vec<types::ServiceTypeFull>> {
        let conn = &mut *self.conn.borrow_mut();
        let service_types = service_type::table.load::<model::ServiceType>(conn)?;

        let service_type_versions = service_version::table.load::<model::ServiceVersion>(conn)?;

        let epochs = epoch::table
            .load::<model::Epoch>(conn)?
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
                            cli_name: service_type.cli_name.clone(),
                            rebuild_required: sv.rebuild_required,
                            last_built_at: sv.last_built_at,
                            last_build_commit_hash: sv.last_build_commit_hash.clone(),
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
