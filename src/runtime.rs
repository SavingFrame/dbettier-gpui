use std::{fmt::Display, future::Future};

use gpui::{App, AppContext, Global, Task};
use tokio::{runtime::Runtime, task::JoinError};

struct TokioRuntime {
    runtime: Option<Runtime>,
    handle: tokio::runtime::Handle,
}

impl Global for TokioRuntime {}

impl Drop for TokioRuntime {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}

pub(crate) fn init(cx: &mut App) -> Result<(), std::io::Error> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;
    cx.set_global(TokioRuntime {
        handle: runtime.handle().clone(),
        runtime: Some(runtime),
    });
    Ok(())
}

pub(crate) fn spawn_result<C, Fut, Output, Error>(
    cx: &C,
    operation: &'static str,
    future: Fut,
) -> Task<Result<Output, String>>
where
    C: AppContext,
    Fut: Future<Output = Result<Output, Error>> + Send + 'static,
    Output: Send + 'static,
    Error: Display + Send + 'static,
{
    let task = spawn(cx, future);
    cx.background_spawn(async move {
        task.await
            .map_err(|error| format!("{operation} task failed: {error}"))?
            .map_err(|error| error.to_string())
    })
}

struct AbortOnDrop(tokio::task::AbortHandle);

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

pub(crate) fn spawn<C, Fut, Output>(cx: &C, future: Fut) -> Task<Result<Output, JoinError>>
where
    C: AppContext,
    Fut: Future<Output = Output> + Send + 'static,
    Output: Send + 'static,
{
    cx.read_global(|runtime: &TokioRuntime, cx| {
        let join = runtime.handle.spawn(future);
        let abort = AbortOnDrop(join.abort_handle());
        cx.background_spawn(async move {
            let result = join.await;
            drop(abort);
            result
        })
    })
}
