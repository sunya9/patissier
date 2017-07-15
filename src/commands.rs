extern crate iron;
extern crate time;
extern crate crypto;
extern crate rusqlite;
extern crate params;
extern crate rand;

use std::iter;
use std::env;
use std::process::{Command};
use self::rand::Rng;
use self::iron::prelude::*;
use self::iron::status;
use self::rusqlite::Connection;
use self::crypto::sha2::Sha256;
use self::crypto::digest::Digest;
use self::params::{Params, Value};
use webhook::WebHook;

pub fn new(iter: iter::Skip<env::Args>) {
  let con = get_connection();
  let mut sha256 = Sha256::new();
  let salt = rand::thread_rng().gen_ascii_chars().take(40).collect::<String>();
  sha256.input_str(&salt);
  let hash = sha256.result_str();
  let command = iter.map(|arg| arg.to_string()).collect::<Vec<String>>().join(" ");
  if command.len() > 0 {
    let mut stmt = con.prepare("INSERT INTO hooks (hash, command) VALUES (?1, ?2)").unwrap();
    let id = stmt.insert(&[&hash, &command]).unwrap();
    let hook = WebHook {
      id: id,
      hash: hash,
      command: command
    };
    println!("{}", hook);

  } else {
    panic!("Error! You have to set command to second parameter.");
  }
}

pub fn show() {
  let con = get_connection();
  let mut stmt = con.prepare("SELECT id, hash, command FROM hooks").unwrap();
  let iter = stmt.query_map(&[], |row| {
    WebHook {
      id: row.get(0),
      hash: row.get(1),
      command: row.get(2)
    }
  }).unwrap();
  for hook in iter {
    println!("{}", hook.unwrap());
  }
}

pub fn delete(iter: iter::Skip<env::Args>) {
  let mut success: i32 = 0;
  let con = get_connection();
  for argstr in iter {
    let _ = con.execute("DELETE FROM hooks where id = ?1", &[&argstr]).unwrap();
    success += 1;
  }
  println!("delete {} rows", success);
}

pub fn start() {
  let host = format!("localhost:{}", env::var("PORT").unwrap_or("5123".to_string()));
  println!("Listening on http://{}", host);
  Iron::new(handler).http(&host).unwrap();
}

fn handler(req: &mut Request) -> IronResult<Response> {
  let con = get_connection();
  let map = req.get_ref::<Params>().unwrap();
  if let Some(id) = map.get("id").and_then(convert_id) {
    let mut stmt = con.prepare("SELECT command FROM hooks where hash = ?1").unwrap();
    let row = stmt.query_row(&[&id], |row| row.get::<i32, String>(0));
    if let Ok(command) = row {
      let mut iter = command.split_whitespace();
      let output = Command::new(iter.next().unwrap())
        .args(iter)
        .output();
      match output {
        Ok(output_str) => {
          let (status_code, out) = if output_str.status.success() {
            (status::Ok, output_str.stdout)
          } else {
            (status::InternalServerError, output_str.stderr)
          };
          return Ok(Response::with((status_code, out)))
        },
        Err(err) => {
          return Ok(Response::with((status::InternalServerError, err.to_string())))
        }
      }
    }
  }
  Ok(Response::with((status::NotFound, "ID not found.")))
}

fn convert_id(val: &Value) -> Option<String> {
  match val {
    &Value::String(ref v) => Some(v.to_string()),
    _ => None
  }
}

pub fn init() {
  let _ = get_connection().execute("CREATE TABLE hooks (
    id  INTEGER PRIMARY KEY,
    hash TEXT NOT NULL,
    command TEXT NOT NULL)", &[]);
}

fn get_connection() -> Connection {
  Connection::open("./db.sqlite").unwrap()
}