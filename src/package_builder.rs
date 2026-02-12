use crate::domain::package_spec::PackageSpec;
use crate::options::BuildOptions;

#[derive(Debug, Clone)]
pub struct PackageBuilder {
    options: BuildOptions,
    spec: PackageSpec,
}

impl PackageBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: BuildOptions::default(),
            spec: PackageSpec::default(),
        }
    }

    #[must_use]
    pub const fn options(&self) -> BuildOptions {
        self.options
    }

    #[must_use]
    pub fn with_options(mut self, options: BuildOptions) -> Self {
        self.options = options;
        self
    }

    #[must_use]
    pub fn with_spec(mut self, spec: PackageSpec) -> Self {
        self.spec = spec;
        self
    }

    #[must_use]
    pub const fn spec(&self) -> &PackageSpec {
        &self.spec
    }

    pub fn build(self) -> crate::Result<PackageSpec> {
        Ok(self.spec)
    }
}

impl Default for PackageBuilder {
    fn default() -> Self {
        Self::new()
    }
}
