use crate::api::JsHandle;
use crate::imp::console_message::ConsoleMessage as Impl;
use crate::imp::core::*;
use crate::imp::prelude::*;
use crate::imp::utils::SourceLocation;

/// `ConsoleMessage` objects are dispatched by page via the
/// [page::Event::Console](crate::api::page::Event::Console) event.
#[derive(Clone)]
pub struct ConsoleMessage {
  inner: Weak<Impl>,
}

impl ConsoleMessage {
  pub(crate) fn new(inner: Weak<Impl>) -> Self {
    Self { inner }
  }

  /// One of the following values: `'log'`, `'debug'`, `'info'`, `'error'`,
  /// `'warning'`, `'dir'`, `'dirxml'`, `'table'`, `'trace'`, `'clear'`,
  /// `'startGroup'`, `'startGroupCollapsed'`, `'endGroup'`, `'assert'`,
  /// `'profile'`, `'profileEnd'`, `'count'`, `'timeEnd'`.
  pub fn r#type(&self) -> Result<String, Error> {
    Ok(upgrade(&self.inner)?.r#type().into())
  }

  /// The text of the console message.
  pub fn text(&self) -> Result<String, Error> {
    Ok(upgrade(&self.inner)?.text().into())
  }

  /// URL of the resource followed by 0-based line and column numbers in the
  /// resource formatted as `URL:line:column`.
  pub fn location(&self) -> Result<SourceLocation, Error> {
    Ok(upgrade(&self.inner)?.location().to_owned())
  }

  /// List of arguments passed to a `console` function call.
  pub fn args(&self) -> Result<Vec<JsHandle>, Error> {
    Ok(
      upgrade(&self.inner)?
        .args()
        .iter()
        .map(|x| JsHandle::new(x.clone()))
        .collect(),
    )
  }
}
