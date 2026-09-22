use anki_forge::build::{ApkgArtifact, BuildError, BuildReport};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::json;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::Mutex;

use crate::reports;

/// A single independently releasable owner, backed by the core's shared artifact.
#[napi]
pub struct NativeApkgArtifact {
    path: String,
    inner: Mutex<Option<ApkgArtifact>>,
}

impl NativeApkgArtifact {
    pub fn new(artifact: ApkgArtifact) -> Self {
        Self {
            path: artifact.path().to_string_lossy().into_owned(),
            inner: Mutex::new(Some(artifact)),
        }
    }

    fn snapshot(&self) -> Result<ApkgArtifact> {
        self.inner
            .lock()
            .map_err(|_| Error::from_reason("BINDING.ARTIFACT_FAILED"))?
            .clone()
            .ok_or_else(|| Error::from_reason("BINDING.ARTIFACT_CLOSED"))
    }
}

#[napi]
impl NativeApkgArtifact {
    #[napi(getter)]
    pub fn path(&self) -> String {
        self.path.clone()
    }

    #[napi]
    pub fn clone_handle(&self) -> Result<Self> {
        self.snapshot().map(Self::new)
    }

    #[napi]
    pub fn persist_to<'env>(&self, env: &'env Env, path: String) -> Result<Object<'env>> {
        crate::tasks::spawn(
            env,
            ArtifactTask {
                artifact: Some(self.snapshot()?),
                destination: Some(path.into()),
            },
        )
    }

    #[napi]
    pub fn close<'env>(&self, env: &'env Env) -> Result<Object<'env>> {
        let artifact = self
            .inner
            .lock()
            .map_err(|_| Error::from_reason("BINDING.ARTIFACT_FAILED"))?
            .take();
        crate::tasks::spawn(
            env,
            ArtifactTask {
                artifact,
                destination: None,
            },
        )
    }
}

struct ArtifactTask {
    artifact: Option<ApkgArtifact>,
    destination: Option<PathBuf>,
}

impl Task for ArtifactTask {
    type Output = Option<ApkgArtifact>;
    type JsValue = Option<NativeApkgArtifact>;

    fn compute(&mut self) -> Result<Self::Output> {
        catch_unwind(AssertUnwindSafe(|| match self.destination.take() {
            Some(path) => self
                .artifact
                .as_ref()
                .expect("persist task owns an artifact")
                .persist_to(&path)
                .map(Some)
                .map_err(|error| Error::from_reason(format!("BINDING.ARTIFACT_IO: {error}"))),
            None => {
                drop(self.artifact.take());
                Ok(None)
            }
        }))
        .map_err(|_| Error::from_reason("BINDING.ARTIFACT_FAILED"))?
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output.map(NativeApkgArtifact::new))
    }
}

/// The ownership-bearing part travels beside JSON, including on domain failure.
pub struct BuildOutcome {
    pub result: String,
    pub artifact: Option<ApkgArtifact>,
}

impl BuildOutcome {
    pub fn from_result(result: std::result::Result<BuildReport, BuildError>) -> Self {
        match result {
            Ok(report) => Self {
                result: reports::success(reports::build_report(&report)),
                artifact: report.artifact,
            },
            Err(error) => {
                let mut details = reports::build_report(&error.report);
                details["cause"] = json!(format!("{:?}", error.cause));
                Self {
                    result: reports::failure(
                        "build",
                        error.code().as_str(),
                        &error.to_string(),
                        details,
                    ),
                    artifact: error.report.artifact,
                }
            }
        }
    }

    pub fn into_native(self) -> NativeBuildResult {
        NativeBuildResult {
            result: self.result,
            artifact: self.artifact.map(NativeApkgArtifact::new),
        }
    }
}

#[napi(object, object_from_js = false)]
pub struct NativeBuildResult {
    pub result: String,
    pub artifact: Option<NativeApkgArtifact>,
}
