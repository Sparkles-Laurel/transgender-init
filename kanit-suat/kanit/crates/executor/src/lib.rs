use std::future::Future;
use std::sync::OnceLock;

use async_executor::{LocalExecutor, Task};
use futures_lite::stream::iter;
use futures_lite::StreamExt;
use send_wrapper::SendWrapper;

static GLOBAL_EXECUTOR: OnceLock<SendWrapper<LocalExecutor<'static>>> = OnceLock::new();

pub fn spawn<T: 'static>(future: impl Future<Output = T> + 'static) -> Task<T> {
    GLOBAL_EXECUTOR
        .get_or_init(|| SendWrapper::new(LocalExecutor::new()))
        .spawn(future)
}

pub fn block<T: 'static>(future: impl Future<Output = T> + 'static) -> T {
    async_io::block_on(
        GLOBAL_EXECUTOR
            .get_or_init(|| SendWrapper::new(LocalExecutor::new()))
            .run(future),
    )
}

pub async fn join_all<I, F, R: 'static>(futures: I) -> Vec<R>
where
    I: IntoIterator<Item = F>,
    F: Future<Output = R> + 'static,
{
    let handles: Vec<_> = futures.into_iter().map(spawn).collect();

    iter(handles).then(|f| f).collect().await
}
