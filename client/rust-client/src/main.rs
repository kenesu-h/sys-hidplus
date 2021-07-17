pub mod client;
pub mod input;

use crate::client::Client;

use clap::{Arg, App, ArgMatches};

#[macro_use(c)]
extern crate cute;

#[macro_use]
extern crate structure;

fn main() {
  let matches: ArgMatches = App::new("sys-hidplus client")
    .version("1.0")
    .about("An input client for sys-hidplus.")
    .arg(
      Arg::with_name("ip")
      .help("The IP for the target Nintendo Switch.")
      .required(true)
      .takes_value(true)
    )
    .get_matches();
  let ip: &str = matches.value_of("ip").unwrap();
  let mut client: Client = Client::new();
  client.start(ip, true);
}
