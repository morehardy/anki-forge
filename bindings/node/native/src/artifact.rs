use ankiforge::build::{ApkgArtifact, BuildOptions, BuildOutput};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::json;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::Mutex;

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
                .map_err(|error| {
                    crate::domain(
                        "persist",
                        error.kind(),
                        error.code(),
                        &error,
                        json!({"publication":error.publication()}),
                    )
                }),
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

/// Build on an owned project snapshot so later JS edits cannot race this operation.
pub struct BuildTask {
    pub project: ankiforge::Project,
    pub options: BuildOptions,
}
impl Task for BuildTask {
    type Output = BuildOutput;
    type JsValue = NativeBuildResult;
    fn compute(&mut self) -> Result<BuildOutput> {
        self.project.build(self.options.clone()).map_err(|e| crate::domain("build", e.kind(), e.code(), &e, json!({"snapshot": e.snapshot(), "limitExceeded":e.limit_exceeded().map(|l|json!({"resource":l.resource,"entry":l.entry,"limit":l.limit,"observed":l.observed}))})))
    }
    fn resolve(&mut self, _env: Env, output: BuildOutput) -> Result<NativeBuildResult> {
        Ok(NativeBuildResult {
            snapshot: serde_json::to_string(&output.snapshot()).expect("serializable snapshot"),
            artifact: NativeApkgArtifact::new(output.artifact().clone()),
        })
    }
}
#[napi(object, object_from_js = false)]
pub struct NativeBuildResult {
    pub snapshot: String,
    pub artifact: NativeApkgArtifact,
}
