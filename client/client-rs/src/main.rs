pub mod client;
pub mod config;
pub mod input;

use crate::{
  input::adapter::sdl::SdlAdapter,
  client::Client, 
};
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
  match confy::load_path("./config.toml") {
    Ok(config) => match Client::new(config, Box::new(SdlAdapter::new())) {
      Ok(mut client) => {
        /* 
         * Everything below here is pretty much thanks to the following link:
         * https://rust-cli.github.io/book/in-depth/signals.html
         */
        let ctrl_c_events = ctrl_channel()?;
        let ticks = tick(time::Duration::from_secs_f32(1.0 / 60.0));

        loop {
          select! {
            recv(ticks) -> _ => {
              client.update_pads();
              match client.update_server() {
                Err(e) => {
                  println!("An error occurred while attempting to update the
                    input server:");
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
      },
      Err(e) => {
        println!("{}", e);
        return Ok(());
      }
    },
    Err(e) => panic!("{}", e)
  }
}
