use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProjectMediaPolicy {
    #[default]
    Strict,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectNormalizeOptions {
    pub base_dir: Option<PathBuf>,
    pub media_store_dir: Option<PathBuf>,
    pub media_policy: ProjectMediaPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOptions {
    pub output: Option<PathBuf>,
    pub artifacts_dir: Option<PathBuf>,
    pub normalize_options: Option<ProjectNormalizeOptions>,
    pub inspect: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            output: None,
            artifacts_dir: None,
            normalize_options: None,
            inspect: true,
        }
    }
}

impl BuildOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn output(mut self, path: impl Into<PathBuf>) -> Self {
        self.output = Some(path.into());
        self
    }

    pub fn artifacts_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.artifacts_dir = Some(path.into());
        self
    }

    pub fn normalize_options(mut self, options: ProjectNormalizeOptions) -> Self {
        self.normalize_options = Some(options);
        self
    }

    pub fn inspect(mut self, inspect: bool) -> Self {
        self.inspect = inspect;
        self
    }
}

impl ProjectNormalizeOptions {
    pub fn strict() -> Self {
        Self {
            base_dir: None,
            media_store_dir: None,
            media_policy: ProjectMediaPolicy::Strict,
        }
    }

    pub fn base_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.base_dir = Some(path.into());
        self
    }

    pub fn media_store_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.media_store_dir = Some(path.into());
        self
    }

    #[allow(dead_code)]
    pub(crate) fn to_authoring_media_policy(&self) -> authoring_core::MediaPolicy {
        match self.media_policy {
            ProjectMediaPolicy::Strict => authoring_core::MediaPolicy::default_strict(),
        }
    }
}
