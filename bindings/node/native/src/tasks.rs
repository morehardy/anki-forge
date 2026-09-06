use napi::bindgen_prelude::{spawn_blocking, Object};
use napi::{Env, Result, Task};

/// Keep task ownership until the JS callback runs or its environment is closed.
/// napi 3.12.2's AsyncTask completion tries to throw into a terminating Worker,
/// which aborts debug builds. JsDeferred drains its callback safely on teardown.
/// The built-in runtime also keeps the addon loaded while native work is alive.
pub fn spawn<'env, T: Task + 'static>(env: &'env Env, mut task: T) -> Result<Object<'env>> {
    let (deferred, promise) = env.create_deferred()?;
    spawn_blocking(move || {
        let result = task.compute();
        deferred.resolve(move |env| {
            let value = match result {
                Ok(output) => task.resolve(env, output),
                Err(error) => task.reject(env, error),
            };
            task.finally(env)?;
            value
        });
    });
    Ok(promise)
}
