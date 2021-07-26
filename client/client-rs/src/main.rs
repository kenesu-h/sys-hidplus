pub mod client;
pub mod config;
pub mod input;
pub mod pad;

use crate::{
  client::Client,
  config::Config,
  input::{
    GilrsInputReader,
    MultiInputReader
  },
};
use clap::{Arg, App, ArgMatches};
use crossbeam_channel::{bounded, tick, Receiver, select};
use std::time;

#[macro_use(c)]
extern crate cute;

#[macro_use]
extern crate structure;

// Opens a channel to receive Ctrl-C signals.
fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
  let (sender, receiver) = bounded(100);
  ctrlc::set_handler(move || {
      let _ = sender.send(());
  })?;

  return Ok(receiver);
}

fn main() -> Result<(), ctrlc::Error> {
  let matches: ArgMatches = App::new("sys-hidplus client")
    .version("1.0")
    .about("An input client for sys-hidplus.")
    .arg(
      Arg::with_name("server_ip")
      .help("The IP for the target Nintendo Switch.")
      .required(true)
      .takes_value(true)
    )
    .get_matches();

  let server_ip: &str = matches.value_of("server_ip").unwrap();
  let config: Config = confy::load_path("./config.toml")
    .expect("Expected a config to be generated from a file.");

  // TODO: Change later, we should be checking for the current OS before deciding on an input reader.
  let mut client: Client = Client::new(
    config,
    Box::new(GilrsInputReader::new()),
    // Some(Box::new(MultiInputReader::new())),
    None
  );
  client.set_server_ip(server_ip);

  // Everything below here is pretty much thanks to the following link:
  // https://rust-cli.github.io/book/in-depth/signals.html
  let ctrl_c_events = ctrl_channel()?;

  // If we ever consider dynamically adjusting tickrate according to the Switch's framerate, change
  // this to be mutable.
  let ticks = tick(time::Duration::from_secs_f32(1.0 / 60.0));

  loop {
    select! {
      recv(ticks) -> _ => {
        client.update_all_pads();
        match client.update_server() {
          Err(e) => {
            println!("{}", e);
            // This isn't actually ok but it's a way to end when we get a socket error
            return Ok(());
          },
          Ok(_) => ()
        }
      }
      recv(ctrl_c_events) -> _ => {
        match client.cleanup() {
          Ok(msg) => println!("{}", msg),
          Err(e) => println!("{}", e)
        }
        return Ok(());
      }
    }
  }
}
