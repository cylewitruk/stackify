use std::path::Path;

use bollard::container::{DownloadFromContainerOptions, UploadToContainerOptions};
use color_eyre::{eyre::eyre, Result};

use super::{stackify_docker::StackifyDocker, util::concat_byte_stream};

impl StackifyDocker {
    pub fn download_file_from_container(
        &self,
        container_name: &str,
        file_path: &Path,
    ) -> Result<Vec<u8>> {
        let opts = DownloadFromContainerOptions {
            path: file_path.to_string_lossy().to_string(),
        };

        self.runtime.block_on(async {
            let stream = self
                .docker
                .download_from_container(container_name, Some(opts));

            let result = concat_byte_stream(&self.runtime, stream)?;
            Ok(result)
        })
    }

    pub fn upload_ephemeral_file_to_container(
        &self,
        container_name: &str,
        destination_path: &Path,
        data: &[u8],
    ) -> Result<()> {
        let file_name = destination_path
            .file_name()
            .ok_or_else(|| eyre!("Failed to get file name."))?;
        let dir = destination_path
            .parent()
            .ok_or_else(|| eyre!("Failed to get parent directory."))?;

        let mut tar = tar::Builder::new(Vec::new());
        let mut tar_header = tar::Header::new_gnu();
        tar_header.set_mode(644);
        tar_header.set_size(data.len() as u64);

        tar.append_data(&mut tar_header, Path::new(file_name), data)?;
        tar.finish()?;

        let bytes = tar.into_inner().map_err(|e| eyre!(e))?;

        let opts = UploadToContainerOptions {
            path: format!("{}", dir.display()),
            ..Default::default()
        };

        self.runtime.block_on(async {
            self.docker
                .upload_to_container(container_name, Some(opts), bytes.into())
                .await?;
            Ok(())
        })
    }

    pub fn upload_ephemeral_files_to_container(
        &self,
        container_name: &str,
        destination_dir: &Path,
        files: Vec<(&str, &[u8])>,
    ) -> Result<()> {
        let mut tar = tar::Builder::new(Vec::new());

        for (filename, data) in files {
            let mut tar_header = tar::Header::new_gnu();
            tar_header.set_mode(644);
            tar_header.set_size(data.len() as u64);
            tar.append_data(&mut tar_header, destination_dir.join(filename), data)?;
        }
        tar.finish()?;

        let bytes = tar.into_inner().map_err(|e| eyre!(e))?;

        let opts = UploadToContainerOptions {
            path: format!("{}", destination_dir.display()),
            ..Default::default()
        };

        self.runtime.block_on(async {
            self.docker
                .upload_to_container(container_name, Some(opts), bytes.into())
                .await?;
            Ok(())
        })
    }
}
