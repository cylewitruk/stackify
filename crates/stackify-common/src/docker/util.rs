use std::{collections::HashMap, path::Path};

use bytes::Bytes;
use color_eyre::Result;
use futures_util::{Stream, TryStreamExt};
use rand::{thread_rng, Rng};
use tokio::runtime::Runtime;

use crate::EnvironmentName;

use super::LabelKey;

// =============================================================================
// Name helpers
// =============================================================================

#[allow(dead_code)]
pub fn get_new_name(environment_name: &EnvironmentName) -> String {
    let random = thread_rng()
        .gen::<[u8; 32]>()
        .iter()
        .take(4)
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    format!(
        "stx-{}-{}",
        environment_name.as_ref()[0..5].to_string(),
        random.to_lowercase()
    )
}

// =============================================================================
// Stream helpers
// =============================================================================

pub fn concat_byte_stream<S>(rt: &Runtime, s: S) -> Result<Vec<u8>>
where
    S: Stream<Item = std::result::Result<Bytes, bollard::errors::Error>>,
{
    rt.block_on(async {
        let result = s
            .try_fold(Vec::new(), |mut acc, chunk| async move {
                acc.extend_from_slice(&chunk[..]);
                Ok(acc)
            })
            .await?;
        Ok(result)
    })
}

// =============================================================================
// Bollard filters
// =============================================================================

pub fn make_filters() -> HashMap<String, Vec<String>> {
    return HashMap::new();
}

pub trait AddLabelFilter {
    fn add_label_filter(&mut self, label: LabelKey, value: &str) -> &mut Self;
}

impl AddLabelFilter for HashMap<String, Vec<String>> {
    fn add_label_filter(&mut self, label: LabelKey, value: &str) -> &mut Self {
        self.insert("label".into(), vec![format!("{}={}", label, value)]);
        self
    }
}

// =============================================================================
// TAR archive helpers
// =============================================================================

pub trait TarAppend {
    fn append_data2<P: AsRef<Path>>(&mut self, path: P, data: &[u8]) -> Result<()>;
}

impl TarAppend for tar::Builder<Vec<u8>> {
    fn append_data2<P: AsRef<Path>>(&mut self, path: P, data: &[u8]) -> Result<()> {
        let mut header = tar::Header::new_gnu();
        header.set_path(path)?;
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        self.append(&header, data)?;
        Ok(())
    }
}
