use color_eyre::{eyre::bail, Result};
use futures_util::StreamExt;
use regex::Regex;

use crate::{
    cli::{context::CliContext, log::clilog, theme::ThemedObject},
    docker::BuildResult,
    docker_api::{models::ImageBuildChunk, opts::ImageBuildOpts},
};

pub async fn clean_images(ctx: &CliContext) -> Result<()> {
    // Remove existing build image if it exists
    let _ = ctx
        .docker()
        .api()
        .images()
        .get("stackify-build:latest")
        .delete()
        .await;

    // Remove existing runtime image if it exists
    let _ = ctx
        .docker()
        .api()
        .images()
        .get("stackify-runtime:latest")
        .delete()
        .await;

    let _ = ctx
        .docker()
        .api()
        .images()
        .get("stacks-cli:latest")
        .delete()
        .await;

    Ok(())
}

pub async fn build_image(ctx: &CliContext, opts: &ImageBuildOpts) -> Result<BuildResult> {
    let regex = Regex::new(r#"^Step (\d+)\/(\d+) :(.*)$"#)?;

    let images = ctx.docker().api().images();
    let mut stream = images.build_par(opts);

    while let Some(result) = stream.next().await {
        match result {
            Ok(info) => {
                match info {
                    ImageBuildChunk::Digest { aux } => {
                        //spinner.start(format!("Digest: {}", aux.id));
                        clilog!("Digest: {}", aux.id);
                        return Ok(BuildResult::Success(aux.id));
                    }
                    ImageBuildChunk::Error {
                        error,
                        error_detail,
                    } => {
                        clilog!("Error: {}", error);
                        clilog!("Error Detail: {:?}", error_detail);
                        return Ok(BuildResult::Failed(error, error_detail.message));
                    }
                    ImageBuildChunk::PullStatus {
                        status: _,
                        id: _,
                        progress: _,
                        progress_detail: _,
                    } => {
                        //spinner.start(format!("Pulling: {}", status));
                    }
                    ImageBuildChunk::Update { stream } => {
                        clilog!("{}", stream);
                        regex.captures(&stream).map(|captures| {
                            let _step = captures.get(1).unwrap().as_str();
                            let _total = captures.get(2).unwrap().as_str();
                            let _msg = captures.get(3).unwrap().as_str();
                            //spinner.start(format!("[{}/{}]: {}", step, total, msg));
                        });
                    }
                }
            }
            Err(e) => clilog!("Error: {}", e),
        }
    }

    bail!("Error building image, ended up in an unknown state.");
}
