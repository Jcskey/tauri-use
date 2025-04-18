use std::path::PathBuf;
use serde::Serialize;

use crate::{use_invoke_with_args, use_invoke_with_options, UseTauriWithReturn};

#[derive(Clone, Serialize)]
#[serde(rename = "camelCase")]
pub struct ExistsOptions {
    pub base_dir: Option<PathBuf>,
}

pub fn use_exists() -> UseTauriWithReturn<ExistsOptions, bool> {
    use_invoke_with_options::<ExistsOptions, bool>("plugin:fs|exists")
}

pub fn use_size() -> UseTauriWithReturn<PathBuf, u64> {
    use_invoke_with_args::<PathBuf, u64>("plugin:fs|size")
}
