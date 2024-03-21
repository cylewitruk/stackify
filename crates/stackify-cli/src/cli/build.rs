use std::{fmt::Write, time::Duration};

use clap::Args;
use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use regex::Regex;
use stackify_common::docker::stackify_docker::BuildStackifyArtifacts;

use crate::context::CliContext;

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(
        short = 'b',
        long,
        default_value = "26.0",
        required = false,
    )]
    pub bitcoin_version: String
}

pub fn exec(ctx: CliContext, args: BuildArgs) -> Result<()> {
    let build = BuildStackifyArtifacts {
        user_id: ctx.user_id,
        group_id: ctx.group_id,
        bitcoin_version: args.bitcoin_version,
    };

    let regex = Regex::new(r#"^Step (\d+)\/(\d+) :(.*)$"#)?;

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(200));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.dim.bold} docker: {wide_msg}")
            .unwrap()
            .tick_chars("/|\\- "),
    );
    
    println!("Building stackify artifacts...");
    let start = std::time::Instant::now();

    let stream = ctx.docker.build_stackify_artifacts(build)?;

    tokio::runtime::Runtime::new()?.block_on(async {
        stream
            .for_each(|result| async {
                match result {
                    Ok(info) => {
                        regex.captures(&info.message).map(|captures| {
                            let step = captures.get(1).unwrap().as_str();
                            let total = captures.get(2).unwrap().as_str();
                            let msg = captures.get(3).unwrap().as_str();
                            pb.set_message(format!("Step {}/{}: {}", step, total, msg));
                        });
                    },
                    Err(e) => eprintln!("Error: {}", e),
                }
            })
            .await
    });

    pb.finish_and_clear();
    println!("{} Build completed in {:?}s", "✔️".green(), start.elapsed().as_secs());

    Ok(())
}