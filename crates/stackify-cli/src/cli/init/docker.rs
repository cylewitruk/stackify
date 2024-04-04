use color_eyre::Result;
use docker_api::{models::ImageBuildChunk, opts::ImageBuildOpts};
use futures_util::StreamExt;
use regex::Regex;

use crate::cli::{context::CliContext, theme::ThemedObject};

pub async fn build_image(ctx: &CliContext, name: &str, opts: &ImageBuildOpts) -> Result<()> {
    let regex = Regex::new(r#"^Step (\d+)\/(\d+) :(.*)$"#)?;
    let spinner = cliclack::spinner();
    spinner.start(format!("Building {} image...", name.bold()));

    ctx.docker()
        .api()
        .images()
        .build(opts)
        .for_each(|result| async {
            match result {
                Ok(info) => {
                    match info {
                        ImageBuildChunk::Digest { aux: _ } => {
                            //spinner.start(format!("Digest: {}", aux.id));
                        }
                        ImageBuildChunk::Error {
                            error,
                            error_detail,
                        } => {
                            eprintln!("Error: {}", error);
                            eprintln!("Error Detail: {:?}", error_detail);
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
                            regex.captures(&stream).map(|captures| {
                                let _step = captures.get(1).unwrap().as_str();
                                let _total = captures.get(2).unwrap().as_str();
                                let _msg = captures.get(3).unwrap().as_str();
                                //spinner.start(format!("[{}/{}]: {}", step, total, msg));
                            });
                        }
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        })
        .await;

    spinner.stop("Build image complete");

    Ok(())
}
