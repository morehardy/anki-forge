use crate::domain::package_spec::PackageSpec;
use crate::validate::diagnostic::Diagnostic;

#[must_use]
pub fn check(_spec: &PackageSpec) -> Vec<Diagnostic> {
    Vec::new()
}
