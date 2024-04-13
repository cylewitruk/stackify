use color_eyre::Result;

use super::context::CliContext;

pub async fn exec(ctx: &CliContext) -> Result<()> {
    // TODO: Remove all containers + images
    if ctx.host_dirs.app_root.ends_with(".stackify") {
        std::fs::remove_dir_all(ctx.host_dirs.app_root.clone()).unwrap();
    }
    Ok(())
}
