use ankiforge::build::{BuildOptions, BuildOutput, PreparedPublication};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::json;

use crate::artifact::{build_error, NativeBuildResult};

#[napi]
pub struct NativePreparedPublication {
    owner: Option<PreparedPublication>,
    report: String,
    reason: &'static str,
}

impl NativePreparedPublication {
    fn new(owner: PreparedPublication) -> Self {
        Self {
            report: serde_json::to_string(&owner.report().snapshot()).expect("serializable report"),
            owner: Some(owner),
            reason: "closed",
        }
    }
}

#[napi]
impl NativePreparedPublication {
    #[napi(getter)]
    pub fn report(&self) -> String {
        self.report.clone()
    }

    #[napi]
    pub fn publish<'env>(&mut self, env: &'env Env) -> Result<Object<'env>> {
        let owner = self.owner.take().ok_or_else(|| {
            Error::from_reason(json!({
            "domain":"prepared", "kind":"Unavailable", "code":"BUILD.PREPARED_UNAVAILABLE",
            "message":"prepared publication is unavailable", "details":{"reason":self.reason}
        }).to_string())
        })?;
        self.reason = "consumed";
        crate::tasks::spawn(env, PublishTask { owner: Some(owner) })
    }

    #[napi]
    pub fn close<'env>(&mut self, env: &'env Env) -> Result<Object<'env>> {
        // Taking the owner is synchronous; deletion happens on the same worker
        // mechanism as artifact cleanup. A publish task already owns its file.
        crate::tasks::spawn(env, CloseTask(self.owner.take()))
    }
}

pub struct PrepareTask {
    pub project: Option<std::sync::Arc<ankiforge::Project>>,
    pub options: BuildOptions,
}
impl Task for PrepareTask {
    type Output = PreparedPublication;
    type JsValue = NativePreparedPublication;
    fn compute(&mut self) -> Result<Self::Output> {
        self.project
            .take()
            .expect("prepare task owns snapshot")
            .prepare_publication(self.options.clone())
            .map_err(build_error)
    }
    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(NativePreparedPublication::new(output))
    }
}

struct PublishTask {
    owner: Option<PreparedPublication>,
}
impl Task for PublishTask {
    type Output = BuildOutput;
    type JsValue = NativeBuildResult;
    fn compute(&mut self) -> Result<Self::Output> {
        self.owner
            .take()
            .expect("publication task owns candidate")
            .publish()
            .map_err(build_error)
    }
    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output.into())
    }
}

struct CloseTask(Option<PreparedPublication>);
impl Task for CloseTask {
    type Output = ();
    type JsValue = ();
    fn compute(&mut self) -> Result<()> {
        drop(self.0.take());
        Ok(())
    }
    fn resolve(&mut self, _env: Env, _: ()) -> Result<()> {
        Ok(())
    }
}
