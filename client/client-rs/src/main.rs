pub mod client;
pub mod config;
pub mod input;

use crate::{
  input::adapter::{
    gilrs::GilrsAdapter,
    multiinput::MultiInputAdapter,
    sdl::SdlAdapter
  },
  client::Client,
  config::Config, 
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

  let matches: ArgMatches = App::new("sys-hidplus client-rs")
    .version("1.0.0")
    .about("An input client for sys-hidplus that is written in Rust.")
    .arg(
      Arg::with_name("server_ip")
      .help("The IP of the Nintendo Switch that is hosting the input server.")
      .required(true)
      .takes_value(true)
    )
    .get_matches();

  let server_ip: &str = matches.value_of("server_ip").unwrap();
  let config: Config = confy::load_path("./config.toml")
    .expect("Expected a config to be generated from a file.");

  let mut client: Client = Client::new(
    config,
    // Box::new(GilrsAdapter::new()),
    Box::new(SdlAdapter::new())
  );
  client.set_server_ip(server_ip);

  /* Everything below here is pretty much thanks to the following link:
   * https://rust-cli.github.io/book/in-depth/signals.html
   */
  let ctrl_c_events = ctrl_channel()?;

  /* If we ever consider dynamically adjusting tickrate according to the Switch's framerate, change
   * this to be a mutable variable rather than immutable.
   */
  let ticks = tick(time::Duration::from_secs_f32(1.0 / 60.0));

  loop {
    select! {
      recv(ticks) -> _ => {
        client.update_all_pads();
        match client.update_server() {
          Err(e) => {
            println!("An error occurred while attempting to update the input server:");
            println!("{}", e);
            match client.cleanup() {
              Ok(msg) => println!("{}", msg),
              Err(e) => println!("{}", e)
            }
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
