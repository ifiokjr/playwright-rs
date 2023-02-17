use crate::imp::core::*;
use crate::imp::prelude::*;

#[derive(Debug)]
pub(crate) struct Selectors {
  channel: ChannelOwner,
}

impl Selectors {
  pub(crate) fn new(channel: ChannelOwner) -> Self {
    Self { channel }
  }

  pub(crate) async fn register(
    &self,
    name: &str,
    script: &str,
    content_script: bool,
  ) -> Result<(), Arc<Error>> {
    let args = RegisterArgs {
      name,
      source: script,
      content_script,
    };
    let _ = send_message!(self, "register", args);
    Ok(())
  }
}

impl RemoteObject for Selectors {
  fn channel(&self) -> &ChannelOwner {
    &self.channel
  }

  fn channel_mut(&mut self) -> &mut ChannelOwner {
    &mut self.channel
  }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterArgs<'a, 'b> {
  name: &'a str,
  source: &'b str,
  #[serde(skip_serializing_if = "std::ops::Not::not")]
  content_script: bool,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::imp::playwright::Playwright;

  crate::runtime_test!(register, {
    let driver = Driver::install().unwrap();
    let conn = Connection::run(&driver.executable()).unwrap();
    let p = Playwright::wait_initial_object(&conn).await.unwrap();
    let p = p.upgrade().unwrap();
    let s: Arc<Selectors> = p.selectors().upgrade().unwrap();
    let fut = s.register("foo", "()", false);
    log::trace!("fut");
    let res = fut.await;
    dbg!(&res);
    assert!(res.is_ok());
  });
}
