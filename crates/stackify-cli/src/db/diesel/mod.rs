use diesel::result::QueryResult;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SqliteConnection};

pub mod model;
pub mod model_ext;
pub mod schema;

use model::*;
use schema::*;

pub trait InnerDb {
    fn find_environment_by_name(
        conn: &mut SqliteConnection,
        name: &str,
    ) -> QueryResult<Option<Environment>> {
        environment::table
            .filter(environment::name.eq(name))
            .first::<Environment>(conn)
            .optional()
    }

    fn find_environment_service_param(
        conn: &mut SqliteConnection,
        environment_service_id: i32,
        service_type_param_id: i32,
    ) -> QueryResult<Option<EnvironmentServiceParam>> {
        environment_service_param::table
            .filter(environment_service_param::environment_service_id.eq(environment_service_id))
            .filter(environment_service_param::service_type_param_id.eq(service_type_param_id))
            .first::<EnvironmentServiceParam>(conn)
            .optional()
    }

    fn load_service_type_params_for_service_type(
        conn: &mut SqliteConnection,
        service_type_id: i32,
    ) -> QueryResult<Vec<ServiceTypeParam>> {
        service_type_param::table
            .filter(service_type_param::service_type_id.eq(service_type_id))
            .load::<ServiceTypeParam>(conn)
    }

    fn load_service_types(conn: &mut SqliteConnection) -> QueryResult<Vec<ServiceType>> {
        service_type::table.load::<ServiceType>(conn)
    }

    fn load_epochs(conn: &mut SqliteConnection) -> QueryResult<Vec<Epoch>> {
        epoch::table.load::<Epoch>(conn)
    }

    fn load_service_type_versions(conn: &mut SqliteConnection) -> QueryResult<Vec<ServiceVersion>> {
        service_version::table.load::<ServiceVersion>(conn)
    }

    fn load_epochs_for_environment(
        conn: &mut SqliteConnection,
        environment_id: i32,
    ) -> QueryResult<Vec<EnvironmentEpoch>> {
        environment_epoch::table
            .filter(environment_epoch::environment_id.eq(environment_id))
            .load::<EnvironmentEpoch>(conn)
    }
}
