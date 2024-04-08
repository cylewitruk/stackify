use color_eyre::{eyre::eyre, Result};
use diesel::{Connection, SqliteConnection};
use stackify_common::types::EnvironmentName;

use crate::util::FilterByServiceType;

use super::{apply_db_migrations, cli_db::CliDatabase, AppDb};

#[test]
pub fn test_load_empty_environment() -> Result<()> {
    let db = get_db()?;

    let env_name_str = "foo";
    let env_name = EnvironmentName::new(env_name_str)?;
    let bitcoin_block_speed = 30;

    db.create_environment(env_name_str, bitcoin_block_speed)?;

    let loaded = db.load_environment("foo")?;

    let epochs = db.list_epochs()?;
    for i in 0..epochs.len() {
        let epoch = &epochs[i];
        let loaded_epoch = loaded
            .epochs
            .iter()
            .find(|e| e.epoch.id == epochs[i].id)
            .ok_or(eyre!("Epoch not found"))?;

        assert_eq!(loaded_epoch.epoch.name, epoch.name);
        assert_eq!(
            loaded_epoch.epoch.default_block_height,
            epoch.default_block_height as u32
        );
        assert_eq!(
            loaded_epoch.starts_at_block_height,
            epoch.default_block_height as u32
        );
        assert!(loaded_epoch.ends_at_block_height.is_none());
    }

    assert_eq!(loaded.name, env_name);
    assert!(loaded.services.is_empty());

    Ok(())
}

#[test]
pub fn test_load_all_service_types() -> Result<()> {
    let db = get_db()?;

    let db_service_types = db.list_service_types()?;
    let service_count = db_service_types.len();
    let versions = db.list_service_versions()?;
    assert!(service_count > 0);

    let db = db.as_clidb();
    let service_types = db.load_all_service_types()?;
    assert_eq!(service_types.len(), service_count);

    for st in service_types.iter() {
        let db_st = db_service_types
            .iter()
            .find(|st2| st2.id == st.id)
            .ok_or(eyre!("Service type not found"))?;

        assert_eq!(st.name, db_st.name);
        assert_eq!(st.cli_name, db_st.cli_name);
        assert_eq!(st.allow_max_epoch, db_st.allow_maximum_epoch);
        assert_eq!(st.allow_min_epoch, db_st.allow_minimum_epoch);
        assert_eq!(st.allow_git_target, db_st.allow_git_target);

        let db_versions = versions.filter_by_service_type(st.id);
        assert_eq!(st.versions.len(), db_versions.len());

        for sv in st.versions.iter() {
            let db_sv = *db_versions
                .iter()
                .find(|sv2| sv2.id == sv.id)
                .ok_or(eyre!("Service version not found"))?;

            assert_eq!(sv.version, db_sv.version);
            //eprintln!("V: {:?} | MIN: {:?} / {:?}", sv.version, sv.min_epoch, db_sv.minimum_epoch_id);
            assert_eq!(sv.min_epoch.as_ref().map(|e| e.id), db_sv.minimum_epoch_id);
            //eprintln!("V: {:?} | MAX: {:?} / {:?}", sv.version, sv.max_epoch, db_sv.maximum_epoch_id);
            assert_eq!(sv.max_epoch.as_ref().map(|e| e.id), db_sv.maximum_epoch_id);
            assert_eq!(sv.git_target.is_some(), db_sv.git_target.is_some());
        }
    }

    Ok(())
}

pub fn get_db() -> Result<AppDb> {
    let mut db_conn = SqliteConnection::establish(":memory:").unwrap();
    apply_db_migrations(&mut db_conn)?;
    Ok(AppDb::new(db_conn))
}
