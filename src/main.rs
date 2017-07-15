mod webhook;
mod commands;

use std::env;
use commands::{init, new, show, delete, start};
fn main() {
  let command = env::args().nth(1).unwrap_or("start".to_string());
  match &*command {
    "init" => init(),
    "new" => new(env::args().skip(2)),
    "show" => show(),
    "delete" => delete(env::args().skip(2)),
    "start" | _ => start()
  }
}
