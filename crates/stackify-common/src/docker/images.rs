use std::{collections::HashMap, io::Write};

use bollard::image::{BuildImageOptions, BuilderVersion};
use color_eyre::Result;
use futures_util::{Stream, StreamExt, TryStreamExt};

use crate::util::random_hex;

use super::{
    stackify_docker::StackifyDocker, util::TarAppend as _, BuildInfo, BuildStackifyBuildImage,
    BuildStackifyRuntimeImage, LabelKey, Progress, StackifyImage, StackifyLabel,
};

impl StackifyDocker {
    /// Lists all images with the label "local.stackify".
    pub fn list_stackify_images(&self) -> Result<Vec<StackifyImage>> {
        let mut filters = HashMap::new();
        filters.insert("label".to_string(), vec![LabelKey::Stackify.to_string()]);

        let opts = bollard::image::ListImagesOptions {
            filters,
            ..Default::default()
        };

        self.runtime.block_on(async {
            let images = self
                .docker
                .list_images(Some(opts))
                .await?
                .iter()
                .map(|image| StackifyImage {
                    id: image.id.clone(),
                    tags: image.repo_tags.clone(),
                    container_count: image.containers,
                    size: image.size,
                })
                .collect::<Vec<_>>();
            Ok(images)
        })
    }

    /// Builds the Stackify build image.
    pub fn build_stackify_build_image(
        &self,
        build: BuildStackifyBuildImage,
    ) -> Result<impl Stream<Item = Result<BuildInfo>> + Unpin + '_> {
        let mut tar = tar::Builder::new(Vec::new());
        tar.append_data2("Dockerfile.build", build.stackify_build_dockerfile)?;
        tar.append_data2("cargo-config.toml", build.stackify_cargo_config)?;
        let archive = tar.into_inner()?;

        let mut c = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        c.write_all(&archive).unwrap();
        let compressed = c.finish().unwrap();

        // docker build --tag stacks.local/runtime:latest --build-arg USER_ID=1000 --build-arg GROUP_ID=1000 --build-arg BITCOIN_VERSION=26.0 --target runtime .

        let image = "stackify-build".to_string();

        let build_args: HashMap<String, String> = [
            ("USER_ID".to_string(), build.user_id.to_string()),
            ("GROUP_ID".to_string(), build.group_id.to_string()),
            (
                "BITCOIN_VERSION".to_string(),
                build.bitcoin_version.to_string(),
            ),
            ("PRE_COMPILE".to_string(), build.pre_compile.to_string()),
        ]
        .iter()
        .cloned()
        .collect();

        let labels = vec![StackifyLabel(LabelKey::Stackify, String::new()).into()]
            .into_iter()
            .collect::<HashMap<_, _>>();

        let opts = BuildImageOptions {
            dockerfile: "Dockerfile.build".to_string(),
            t: image.clone(),
            session: Some(random_hex(8)),
            pull: true,
            buildargs: build_args,
            labels,
            version: BuilderVersion::BuilderV1,
            rm: true,
            ..Default::default()
        };

        let stream = self.docker.build_image(opts, None, Some(compressed.into()));

        return Ok(Box::pin(stream.map(|msg| match msg {
            Ok(info) => {
                Ok(BuildInfo {
                    message: info.stream.unwrap_or_else(|| "".to_string()),
                    error: info.error,
                    progress:
                        info.progress_detail.map(|p| {
                            Progress::new(p.current.unwrap() as u32, p.total.unwrap() as u32)
                        }),
                })
            }
            Err(e) => Err(e.into()),
        })));
    }

    /// Builds the Stackify build image.
    pub fn build_stackify_runtime_image(
        &self,
        build: BuildStackifyRuntimeImage,
    ) -> Result<impl Stream<Item = Result<BuildInfo>> + Unpin + '_> {
        let mut tar = tar::Builder::new(Vec::new());
        tar.append_data2("Dockerfile.runtime", build.stackify_runtime_dockerfile)?;
        let archive = tar.into_inner()?;

        let mut c = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        c.write_all(&archive).unwrap();
        let compressed = c.finish().unwrap();

        // docker build --tag stacks.local/runtime:latest --build-arg USER_ID=1000 --build-arg GROUP_ID=1000 --build-arg BITCOIN_VERSION=26.0 --target runtime .

        let image = "stackify-runtime".to_string();

        let build_args: HashMap<String, String> = [
            ("USER_ID".to_string(), build.user_id.to_string()),
            ("GROUP_ID".to_string(), build.group_id.to_string()),
        ]
        .iter()
        .cloned()
        .collect();

        let labels = vec![StackifyLabel(LabelKey::Stackify, String::new()).into()]
            .into_iter()
            .collect::<HashMap<_, _>>();

        let opts = BuildImageOptions {
            dockerfile: "Dockerfile.runtime".to_string(),
            t: image.clone(),
            session: Some(random_hex(8)),
            pull: true,
            buildargs: build_args,
            labels,
            version: BuilderVersion::BuilderV1,
            rm: true,
            ..Default::default()
        };

        let stream = self.docker.build_image(opts, None, Some(compressed.into()));

        return Ok(Box::pin(stream.map(|msg| match msg {
            Ok(info) => {
                Ok(BuildInfo {
                    message: info.stream.unwrap_or_else(|| "".to_string()),
                    error: info.error,
                    progress:
                        info.progress_detail.map(|p| {
                            Progress::new(p.current.unwrap() as u32, p.total.unwrap() as u32)
                        }),
                })
            }
            Err(e) => Err(e.into()),
        })));
    }

    /// Pulls a remote image.
    pub fn pull_image(&self, image: &str) {
        let ctx = StackifyDocker::new().unwrap();

        ctx.runtime.block_on(async {
            ctx.docker
                .create_image(
                    Some(bollard::image::CreateImageOptions {
                        from_image: image,
                        ..Default::default()
                    }),
                    None,
                    None,
                )
                .try_collect::<Vec<_>>()
                .await
                .expect("Failed to pull image");
        });
    }
}
