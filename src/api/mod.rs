use crate::domain::package_spec::PackageSpec;
use crate::package_builder::PackageBuilder;

#[derive(Debug, Default, Clone, Copy)]
pub struct Facade;

impl Facade {
    #[must_use]
    pub fn builder() -> PackageBuilder {
        PackageBuilder::new()
    }

    #[must_use]
    pub fn from_spec(spec: PackageSpec) -> PackageBuilder {
        PackageBuilder::new().with_spec(spec)
    }
}
