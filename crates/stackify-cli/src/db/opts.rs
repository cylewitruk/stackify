pub struct NewServiceVersionOpts {
    pub service_type_id: i32,
    pub version: String,
    pub cli_name: String,
    pub git_target: Option<String>,
    pub minimum_epoch_id: Option<i32>,
    pub maximum_epoch_id: Option<i32>,
}