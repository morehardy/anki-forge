use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

use crate::build::BuildOptions;
use crate::diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticDomain, DiagnosticStage, Severity, SourcePath,
};

/// All externally named files involved in a build. The candidate APKG lives in
/// a separate private directory; `package` is its eventual artifact destination.
pub(super) struct BuildPathPlan<'a> {
    options: &'a BuildOptions,
    package: Option<PathBuf>,
}

impl<'a> BuildPathPlan<'a> {
    pub(super) fn new(options: &'a BuildOptions) -> Self {
        Self {
            options,
            package: options
                .artifacts_dir
                .as_ref()
                .map(|dir| dir.join("package.apkg")),
        }
    }

    fn paths(&self) -> impl Iterator<Item = (&'static str, &Path)> {
        [
            ("output", self.options.output.as_deref()),
            ("package", self.package.as_deref()),
            ("report_json", self.options.report_json.as_deref()),
            (
                "identity_lockfile",
                self.options.identity_lockfile.as_deref(),
            ),
            ("compare_to", self.options.compare_to.as_deref()),
        ]
        .into_iter()
        .filter_map(|(name, path)| path.map(|path| (name, path)))
    }

    pub(super) fn validate(&self) -> Result<(), Box<Diagnostic>> {
        let paths: Vec<_> = self.paths().collect();
        for (index, &(left_name, left)) in paths.iter().enumerate() {
            for &(right_name, right) in &paths[index + 1..] {
                // An explicit output may name the retained package itself.
                if (left_name, right_name) == ("output", "package") {
                    continue;
                }
                // Two read-only baseline options do not perform any writes.
                if (left_name, right_name) == ("identity_lockfile", "compare_to")
                    && !self.options.write_identity_lockfile
                {
                    continue;
                }
                match paths_alias(left, right) {
                    Ok(false) => {}
                    Ok(true) => {
                        return Err(path_diagnostic(
                            "PROJECT.PATH_COLLISION",
                            right_name,
                            format!(
                                "{left_name} path collides with {right_name}: {} and {}",
                                left.display(),
                                right.display()
                            ),
                        ))
                    }
                    Err(error) => {
                        return Err(path_diagnostic(
                            "PROJECT.BUILD_IO",
                            right_name,
                            format!("cannot verify {left_name} and {right_name} paths: {error}"),
                        ))
                    }
                }
            }
        }
        // The writer also replaces its staging tree. A baseline must not live
        // there even if its filename is different from package.apkg.
        if let (Some(artifacts), Some(baseline)) = (
            self.options.artifacts_dir.as_ref(),
            self.options.compare_to.as_ref(),
        ) {
            let staging = artifacts.join("staging");
            let overlaps = resolved_destination(baseline)
                .and_then(|baseline| {
                    resolved_destination(&staging).map(|staging| baseline.starts_with(staging))
                })
                .map_err(|error| {
                    path_diagnostic("PROJECT.BUILD_IO", "compare_to", error.to_string())
                })?;
            if overlaps {
                return Err(path_diagnostic(
                    "PROJECT.PATH_COLLISION",
                    "compare_to",
                    "compare_to baseline must be outside the writable artifact staging directory"
                        .into(),
                ));
            }
        }
        Ok(())
    }

    /// Error reporting must never destroy one of the files the error describes.
    /// Recheck immediately before writing, including after a failed preflight.
    pub(super) fn report_is_safe(&self) -> bool {
        let Some(report) = self.options.report_json.as_deref() else {
            return false;
        };
        self.paths()
            .filter(|(name, _)| *name != "report_json")
            .all(|(_, protected)| matches!(paths_alias(report, protected), Ok(false)))
    }
}

fn path_diagnostic(code: &str, field: &str, message: String) -> Box<Diagnostic> {
    Box::new(Diagnostic {
        code: DiagnosticCode::new(code),
        severity: Severity::Error,
        domain: Some(DiagnosticDomain::new("project")),
        stage: Some(DiagnosticStage::new("validate")),
        source: Some(SourcePath::new(format!("build.{field}"))),
        message,
        help: Some("choose distinct, accessible paths for APKG output, artifact package, report_json, identity lockfile, and compare_to".into()),
    })
}

fn paths_alias(left: &Path, right: &Path) -> io::Result<bool> {
    if resolved_destination(left)? == resolved_destination(right)? {
        return Ok(true);
    }
    // Canonical paths handle symlinks and case folding on the actual filesystem;
    // file identity also detects hard links on Unix and Windows.
    match same_file::is_same_file(left, right) {
        Ok(same) => Ok(same),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
            ) =>
        {
            Ok(false)
        }
        Err(error) => Err(error),
    }
}

fn resolved_destination(path: &Path) -> io::Result<PathBuf> {
    let mut ancestor = std::path::absolute(path)?;
    let mut suffix = Vec::new();
    loop {
        match fs::canonicalize(&ancestor) {
            Ok(mut resolved) => {
                for component in suffix.iter().rev() {
                    if component == ".." {
                        resolved.pop();
                    } else {
                        resolved.push(component);
                    }
                }
                return Ok(resolved);
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) =>
            {
                // A dangling link cannot safely be treated as a new output file.
                if fs::symlink_metadata(&ancestor).is_ok_and(|meta| meta.file_type().is_symlink()) {
                    return Err(error);
                }
                match ancestor.components().next_back() {
                    Some(Component::Normal(name)) => suffix.push(name.to_os_string()),
                    Some(Component::ParentDir) => suffix.push("..".into()),
                    _ => return Err(error),
                }
                ancestor.pop();
            }
            Err(error) => return Err(error),
        }
    }
}
