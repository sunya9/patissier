use std::fmt;

pub struct WebHook {
  pub id: i64,
  pub hash: String,
  pub command: String
}

impl fmt::Display for WebHook {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "id: {}, url: /?id={}, command: {}", self.id, self.hash, self.command)
  }
}

