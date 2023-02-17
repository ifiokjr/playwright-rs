use crate::api::JsHandle;
use crate::imp::core::*;
use crate::imp::prelude::*;
use crate::imp::worker::Evt;
use crate::imp::worker::Worker as Impl;

/// The Worker class represents a [WebWorker](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API). `worker`
/// event is emitted on the page object to signal a worker creation. `close`
/// event is emitted on the worker object when the worker is gone.
///
/// ```js
/// page.on('worker', worker => {
///  console.log('Worker created: ' + worker.url());
///  worker.on('close', worker => console.log('Worker destroyed: ' + worker.url()));
/// });
///
/// console.log('Current workers:');
/// for (const worker of page.workers())
///  console.log('  ' + worker.url());
/// ```
#[derive(Clone)]
pub struct Worker {
  inner: Weak<Impl>,
}

impl PartialEq for Worker {
  fn eq(&self, other: &Self) -> bool {
    let a = self.inner.upgrade();
    let b = other.inner.upgrade();
    a.and_then(|a| b.map(|b| (a, b)))
      .map(|(a, b)| a.guid() == b.guid())
      .unwrap_or_default()
  }
}

impl Worker {
  subscribe_event! {}

  pub(crate) fn new(inner: Weak<Impl>) -> Self {
    Self { inner }
  }

  pub fn url(&self) -> Result<String, Error> {
    Ok(upgrade(&self.inner)?.url().to_owned())
  }

  pub async fn eval_handle(&self, expression: &str) -> ArcResult<JsHandle> {
    upgrade(&self.inner)?
      .eval_handle(expression)
      .await
      .map(JsHandle::new)
  }

  pub async fn evaluate_handle<T>(&self, expression: &str, arg: Option<T>) -> ArcResult<JsHandle>
  where
    T: Serialize,
  {
    upgrade(&self.inner)?
      .evaluate_handle(expression, arg)
      .await
      .map(JsHandle::new)
  }

  pub async fn eval<U>(&self, expression: &str) -> ArcResult<U>
  where
    U: DeserializeOwned,
  {
    upgrade(&self.inner)?.eval(expression).await
  }

  pub async fn evaluate<T, U>(&self, expression: &str, arg: Option<T>) -> ArcResult<U>
  where
    T: Serialize,
    U: DeserializeOwned,
  {
    upgrade(&self.inner)?.evaluate(expression, arg).await
  }
}

#[derive(Debug)]
pub enum Event {
  Close,
}

impl From<Evt> for Event {
  fn from(e: Evt) -> Self {
    match e {
      Evt::Close => Self::Close,
    }
  }
}
