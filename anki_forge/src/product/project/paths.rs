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
    staging_manifest: Option<PathBuf>,
}

impl<'a> BuildPathPlan<'a> {
    pub(super) fn new(options: &'a BuildOptions) -> Self {
        Self {
            options,
            package: options
                .artifacts_dir
                .as_ref()
                .map(|dir| dir.join("package.apkg")),
            staging_manifest: options
                .artifacts_dir
                .as_ref()
                .map(|dir| dir.join("staging/manifest.json")),
        }
    }

    fn paths(&self) -> impl Iterator<Item = (&'static str, &Path)> {
        [
            ("output", self.options.output.as_deref()),
            ("package", self.package.as_deref()),
            ("staging_manifest", self.staging_manifest.as_deref()),
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
                validate_distinct_paths(left_name, left, right_name, right)?;
            }
        }
        // Materialization runs before the policy gate. Every file preserved on
        // rejection must stay outside staging, including read-only lockfiles.
        // Check media separately because that directory can link outside staging;
        // the actual manifest destination is protected by the pairwise check.
        if let Some(artifacts) = self.options.artifacts_dir.as_ref() {
            let staging = artifacts.join("staging");
            let media = staging.join("media");
            for (name, path) in self.paths().filter(|(name, _)| {
                matches!(
                    *name,
                    "output" | "package" | "identity_lockfile" | "compare_to"
                )
            }) {
                for directory in [&staging, &media] {
                    let overlaps = path_enters_directory(path, directory).map_err(|error| {
                        path_diagnostic("PROJECT.BUILD_IO", name, error.to_string())
                    })?;
                    if overlaps {
                        return Err(path_diagnostic(
                            "PROJECT.PATH_COLLISION",
                            name,
                            format!(
                                "{name} must be outside writable artifact staging directories: {}",
                                directory.display()
                            ),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Error reporting must never destroy one of the files the error describes.
    /// Recheck immediately before writing, including after a failed preflight.
    pub(super) fn validate_report(&self) -> Result<(), Box<Diagnostic>> {
        let Some(report) = self.options.report_json.as_deref() else {
            return Ok(());
        };
        for (name, protected) in self.paths().filter(|(name, _)| *name != "report_json") {
            validate_distinct_paths(name, protected, "report_json", report)?;
        }
        Ok(())
    }
}

fn validate_distinct_paths(
    left_name: &str,
    left: &Path,
    right_name: &str,
    right: &Path,
) -> Result<(), Box<Diagnostic>> {
    match paths_alias(left, right) {
        Ok(false) => Ok(()),
        Ok(true) => Err(path_diagnostic(
            "PROJECT.PATH_COLLISION",
            right_name,
            format!(
                "{left_name} path collides with {right_name}: {} and {}",
                left.display(),
                right.display()
            ),
        )),
        Err(error) => Err(path_diagnostic(
            "PROJECT.BUILD_IO",
            right_name,
            format!("cannot verify {left_name} and {right_name} paths: {error}"),
        )),
    }
}

fn path_enters_directory(path: &Path, directory: &Path) -> io::Result<bool> {
    let directory = resolved_destination(directory)?;
    if resolved_destination(path)?.starts_with(&directory) {
        return Ok(true);
    }

    // A path can enter staging and then follow a link out of it. Replacing that
    // link would still change the caller's baseline path. Resolve each ancestor
    // to account for aliased artifact roots, but normalize parent components so
    // ordinary paths such as staging/../previous.apkg remain valid.
    let mut lexical = PathBuf::new();
    for component in std::path::absolute(path)?.components() {
        match component {
            Component::ParentDir => {
                lexical.pop();
            }
            Component::CurDir => {}
            _ => lexical.push(component.as_os_str()),
        }
    }
    for ancestor in lexical.ancestors() {
        if resolved_destination(ancestor)?.starts_with(&directory) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn path_diagnostic(code: &str, field: &str, message: String) -> Box<Diagnostic> {
    Box::new(Diagnostic {
        code: DiagnosticCode::new(code),
        severity: Severity::Error,
        domain: Some(DiagnosticDomain::new("project")),
        stage: Some(DiagnosticStage::new("validate")),
        source: Some(SourcePath::new(format!("build.{field}"))),
        message,
        help: Some("choose distinct, accessible paths for APKG output, artifact package, staging manifest, report_json, identity lockfile, and compare_to".into()),
    })
}

fn paths_alias(left: &Path, right: &Path) -> io::Result<bool> {
    if resolved_destination(left)? == resolved_destination(right)? {
        return Ok(true);
    }
    // Existing paths account for the actual filesystem's case folding, and
    // file identity also detects hard links on Unix and Windows. New suffixes
    // remain unverified until creation, so callers must recheck after publishing
    // and before subsequent lockfile/report writes.
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
