use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProjectMediaPolicy {
    #[default]
    Strict,
    Advanced {
        unused_binding_behavior: ProjectMediaDiagnosticBehavior,
        unknown_mime_behavior: ProjectMediaDiagnosticBehavior,
        declared_mime_mismatch_behavior: ProjectDeclaredMimeMismatchBehavior,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectMediaDiagnosticBehavior {
    Ignore,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectDeclaredMimeMismatchBehavior {
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectMediaPolicyError {
    behavior: ProjectMediaDiagnosticBehavior,
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
            media_policy: ProjectMediaPolicy::strict(),
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

    pub fn media_policy(mut self, policy: ProjectMediaPolicy) -> Self {
        self.media_policy = policy;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn to_authoring_media_policy(&self) -> authoring_core::MediaPolicy {
        self.media_policy.to_authoring_media_policy()
    }
}

impl ProjectMediaPolicy {
    pub fn strict() -> Self {
        Self::Strict
    }

    pub fn unused_binding_behavior(self, behavior: ProjectMediaDiagnosticBehavior) -> Self {
        let (_, unknown_mime_behavior, declared_mime_mismatch_behavior) = self.behaviors();
        Self::from_behaviors(
            behavior,
            unknown_mime_behavior,
            declared_mime_mismatch_behavior,
        )
    }

    pub fn unknown_mime_behavior(self, behavior: ProjectMediaDiagnosticBehavior) -> Self {
        let (unused_binding_behavior, _, declared_mime_mismatch_behavior) = self.behaviors();
        Self::from_behaviors(
            unused_binding_behavior,
            behavior,
            declared_mime_mismatch_behavior,
        )
    }

    pub fn declared_mime_mismatch_behavior(
        self,
        behavior: ProjectDeclaredMimeMismatchBehavior,
    ) -> Self {
        let (unused_binding_behavior, unknown_mime_behavior, _) = self.behaviors();
        Self::from_behaviors(unused_binding_behavior, unknown_mime_behavior, behavior)
    }

    fn to_authoring_media_policy(self) -> authoring_core::MediaPolicy {
        let mut policy = authoring_core::MediaPolicy::default_strict();
        let (unused_binding_behavior, unknown_mime_behavior, declared_mime_mismatch_behavior) =
            self.behaviors();
        policy.unused_binding_behavior = unused_binding_behavior.to_authoring_behavior();
        policy.unknown_mime_behavior = unknown_mime_behavior.to_authoring_behavior();
        policy.declared_mime_mismatch_behavior =
            declared_mime_mismatch_behavior.to_authoring_behavior();
        policy
    }

    fn behaviors(
        self,
    ) -> (
        ProjectMediaDiagnosticBehavior,
        ProjectMediaDiagnosticBehavior,
        ProjectDeclaredMimeMismatchBehavior,
    ) {
        match self {
            ProjectMediaPolicy::Strict => (
                ProjectMediaDiagnosticBehavior::Warning,
                ProjectMediaDiagnosticBehavior::Warning,
                ProjectDeclaredMimeMismatchBehavior::Error,
            ),
            ProjectMediaPolicy::Advanced {
                unused_binding_behavior,
                unknown_mime_behavior,
                declared_mime_mismatch_behavior,
            } => (
                unused_binding_behavior,
                unknown_mime_behavior,
                declared_mime_mismatch_behavior,
            ),
        }
    }

    fn from_behaviors(
        unused_binding_behavior: ProjectMediaDiagnosticBehavior,
        unknown_mime_behavior: ProjectMediaDiagnosticBehavior,
        declared_mime_mismatch_behavior: ProjectDeclaredMimeMismatchBehavior,
    ) -> Self {
        if unused_binding_behavior == ProjectMediaDiagnosticBehavior::Warning
            && unknown_mime_behavior == ProjectMediaDiagnosticBehavior::Warning
            && declared_mime_mismatch_behavior == ProjectDeclaredMimeMismatchBehavior::Error
        {
            Self::Strict
        } else {
            Self::Advanced {
                unused_binding_behavior,
                unknown_mime_behavior,
                declared_mime_mismatch_behavior,
            }
        }
    }
}

impl ProjectMediaDiagnosticBehavior {
    fn to_authoring_behavior(self) -> authoring_core::DiagnosticBehavior {
        match self {
            ProjectMediaDiagnosticBehavior::Ignore => authoring_core::DiagnosticBehavior::Ignore,
            ProjectMediaDiagnosticBehavior::Info => authoring_core::DiagnosticBehavior::Info,
            ProjectMediaDiagnosticBehavior::Warning => authoring_core::DiagnosticBehavior::Warning,
            ProjectMediaDiagnosticBehavior::Error => authoring_core::DiagnosticBehavior::Error,
        }
    }
}

impl ProjectDeclaredMimeMismatchBehavior {
    fn to_authoring_behavior(self) -> authoring_core::DiagnosticBehavior {
        match self {
            ProjectDeclaredMimeMismatchBehavior::Warning => {
                authoring_core::DiagnosticBehavior::Warning
            }
            ProjectDeclaredMimeMismatchBehavior::Error => authoring_core::DiagnosticBehavior::Error,
        }
    }
}

impl TryFrom<ProjectMediaDiagnosticBehavior> for ProjectDeclaredMimeMismatchBehavior {
    type Error = ProjectMediaPolicyError;

    fn try_from(
        behavior: ProjectMediaDiagnosticBehavior,
    ) -> Result<Self, <Self as TryFrom<ProjectMediaDiagnosticBehavior>>::Error> {
        match behavior {
            ProjectMediaDiagnosticBehavior::Warning => Ok(Self::Warning),
            ProjectMediaDiagnosticBehavior::Error => Ok(Self::Error),
            ProjectMediaDiagnosticBehavior::Ignore | ProjectMediaDiagnosticBehavior::Info => {
                Err(ProjectMediaPolicyError { behavior })
            }
        }
    }
}

impl std::fmt::Display for ProjectMediaPolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "declared MIME mismatch behavior must be warning or error, got {:?}",
            self.behavior
        )
    }
}

impl std::error::Error for ProjectMediaPolicyError {}

impl Default for ProjectMediaDiagnosticBehavior {
    fn default() -> Self {
        Self::Warning
    }
}

impl Default for ProjectDeclaredMimeMismatchBehavior {
    fn default() -> Self {
        Self::Error
    }
}
