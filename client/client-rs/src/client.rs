use crate::{
  config::Config,
  input::{
    InputButton,
    InputEvent,
    InputReader,
    GilrsInputReader,
    MultiInputReader
  },
  pad::{SwitchPad, EmulatedPad}
};

use gilrs::{
  Gilrs,
  Event,
  GamepadId,
  Button
};
use std::{
  collections::HashMap,
  net::UdpSocket,
  time
};

/**
 * A struct representing the main input client.
 * 
 * An input client should have:
 * - A config that dictates what Switch pad types each slot should be assigned.
 * - A UDP socket to transmit emulated pad info to a Switch.
 * - The server IP of the Switch.
 * - A way to access gamepad events (a GilRs instance in this case).
 * - A list of emulated pads.
 */
pub struct Client {
  config: Config,
  sock: UdpSocket,
  server_ip: String,

  input_reader: Box<dyn InputReader>,
  input_map: HashMap<usize, usize>,

  rawinput_reader: Option<Box<dyn InputReader>>,
  rawinput_map: HashMap<usize, usize>,

  pads: Vec<EmulatedPad>,
}

impl Client {
  /**
   * Constructs a client from a configuration.
   * 
   * By default, it initializes a UDP socket, no server IP, a GilRs instance, and emulated pads
   * each with a switch pad type of None.
   */
  /**
   * Constructs a client from a configuration.
   *
   * It initializes a UDP socket, no server IP, and each of the emulated pads are constructed with a
   * type of None.
   *
   * However, the input readers (for both general gamepad API and RawInput support) must be passed
   * through the constructor. This trusts that you pass in input readers that correspond with the
   * respective APIs.
   *
   * HashMaps are also initialized for both input readers in order to have a O(1) lookup for gamepad
   * ID to slots for emulated pads, instead of a O(n) lookup.
   */
  pub fn new(
    config: Config,
    input_reader: Box<dyn InputReader>, rawinput_reader: Option<Box<dyn InputReader>>
  ) -> Client {
    return Client {
      config: config,
      // Unwrapping here might not be the best thing
      sock: UdpSocket::bind("0.0.0.0:8000").unwrap(),
      server_ip: "".to_string(),

      input_reader: input_reader,
      input_map: HashMap::new(),
      
      rawinput_reader: rawinput_reader,
      rawinput_map: HashMap::new(),

      pads: c![EmulatedPad::new(), for _i in 0..4]
    }
  }
 
  /**
   * A method that attempts to assign the given gamepad id and switch pad type to an open slot.
   * Slots are open so as long as they are not equal to None.
   * If there's no open slot, we return an error.
   */
  fn assign_pad(&mut self, gamepad_id: &usize, raw_input: bool) -> Result<String, String> {
    let mut i: usize = 0;
    for pad in &mut self.pads {
      if pad.get_gamepad_id().is_none() {
      // if !self.input_reader.is_connected(pad.get_gamepad_id()) {
      // || !self.rawinput_reader.is_connected(gamepad_id) {
        match self.config.pads_to_vec()[i] {
          Some(switch_pad) => {
            if raw_input {
              self.rawinput_map.insert(*gamepad_id, i);
            } else {
              self.input_map.insert(*gamepad_id, i);
            }
            pad.connect(gamepad_id, switch_pad);
            return Ok(format!("Gamepad (id: {}) connected to slot {}.", &gamepad_id, i + 1));
          },
          None => ()
        }
      }
      i = i + 1;
    }
    return Err("Couldn't assign gamepad since there were no slots available.".to_string());
  }

  // A method that sets the target server IP of this client.
  pub fn set_server_ip(&mut self, server_ip: &str) -> () {
    self.server_ip = server_ip.to_string();
  }

  /**
   * A method that updates this client by parsing gamepad events and updating their respective
   * emulated pads.
   * 
   * This was originally in a start() method in an endless loop, but was moved into this so main()
   * could simultaneously update the client and respond to Ctrl-C events; Rust would be picky
   * about thread-safety and references being moved otherwise, which is a whole other can of
   * worms that would be way too much to handle - in particular, GilRs structs are not thread-safe
   * and I really would not like to make thread-safe GilRs wrappers (or directly edit it) just for
   * this.
   * 
   * Either way, this method should be called at a fixed time interval, ideally matching that of
   * (1 / <the framerate>). Any more than that and input delay might get bad in certain scenarios.
   * - I've experienced such delay in Smash's stage and character selection screens.
   * - Normal Smash gameplay is surprisingly fine and doesn't seem to be affected by this.
   * - POSSIBLY corresponds to lag in demanding games like Mario Odyssey, but I don't have it so I
   *   can't test this.
   * 
   * TODO: This could potentially be alleviated if the Switch sent packets back telling us its
   * current framerate so we could adjust the loop time interval, but:
   * 1. I don't know if libnx has such a function to return the current framerate (a quick search
   *    suggests otherwise).
   * 2. I'm not sure if Rust allows you to dynamically adjust the time interval/tickrate, but in
   *    theory, this should be possible; check main() for this.
   */
  pub fn update_all_pads(&mut self) -> () {
    self.update_pads(false);
    if self.config.get_rawinput_fallback() {
      self.update_pads(true);
    }
  }

  pub fn update_pads(&mut self, rawinput: bool) -> () {
    let input_reader: &mut Box<dyn InputReader>; 
    if rawinput {
      // input_reader = &mut self.rawinput_reader;
    } else {
      input_reader = &mut self.input_reader;
    }
    match self.input_reader.read() {
      Ok(events) => for event in events {
        let input_map: &mut HashMap<usize, usize>;
        if rawinput {
          input_map = &mut self.rawinput_map;
        } else {
          input_map = &mut self.input_map;
        }
        if let Some(i) = self.input_map.get(event.get_gamepad_id()) {
          if *self.pads[*i].get_gamepad_id() == Some(*event.get_gamepad_id()) {
            self.pads[*i].update(&event);
            break;
          }
        } else {
          if let InputEvent::GamepadButton(gamepad_id, button, value) = event {
            // This used to check for both triggers, but multiinput doesn't have an is_pressed()
            // method of any kind, I think. Either way, I changed it so you just have to press your
            // controller's equivalent of the start button. 
            if button == InputButton::Start && value == 1.0 {
              match self.assign_pad(&gamepad_id, rawinput) {
                Err(e) => println!("{}", e),
                Ok(msg) => println!("{}", msg)
              }
            }
          }
        } 
      },
      Err(e) => println!("{}", e)
    }
  }

  // A method that sends the current emulated pad states to the Switch (the input server).
  // Like update_pads(), this should be called at a fixed time interval too.
  pub fn update_server(&self) -> Result<(), String> {
    match self.sock.send_to(
      &PackedData::new(&self.pads, 4).to_bytes(),
      // &PackedData::new(&self.pads, self.get_connected()).to_bytes(),
      format!("{}:8000", self.server_ip)
    ) {
      Err(e) => return Err(
        format!("The following error occurred: {}. The given IP is either invalid or improperly formatted.", e)
      ),
      Ok(_) => Ok(())
    }
  }

  // A method that's SUPPOSED to cleanup all the controllers so that they're gone.
  /*
  The big problem is that we're using a UDP socket to tell the Switch "controllers are gone".

  Unfortunately, UDP is liable to drops and I imagine that's what's happening when you use reset
  just once. Because of this, you need to send many packets so that at least one will HOPEFULLY tell
  the Switch all the controllers are gone, which is (I think) why the original implementation
  required that you change all the controller types to 0, then rerun the client (then close) to
  accomplish this. However, this is really clunky, which is why I'm trying to make this easier.

  While the obvious solution is to use a TCP socket or invent our own protocol where the Switch
  sends back a "disconnected" message, I'm not particularly thrilled about installing a Switch
  development environment, working with C++ (I might be a Rust user, but idk C++ and any low-level
  stuff involved that well), and dealing with the already-multi-threaded nature of the sysmodule.
  Though I understand a large portion of the client, I don't understand a whole lot of the
  server/sysmodule portion of this at the time of writing this and I hate working with multi-
  threaded code (even just college examples of this were difficult to debug).

  Because of this, I'm implementing this in a way that we attempt to reset many, many times over the
  course of 3 seconds. In other words, it's a brute force method of cleaning up the controllers
  where we throw shit at the wall until it sticks, but it seems like the best way that doesn't
  involve me digging into the sysmodule portion of the code (yet).

  I'd love to make this method server-agnostic, but I don't see a good way yet.
  */
  pub fn cleanup(&mut self) -> Result<String, String> {
    println!("Cleaning up connected gamepads... This will take a moment.");
    self.pads = c![EmulatedPad::new(), for _i in 0..4];
    let start: time::Instant = time::Instant::now();
    while start.elapsed().as_millis() < 3000 {
      match self.sock.send_to(
        &PackedData::new(&self.pads, 4).to_bytes(),
        format!("{}:8000", self.server_ip)
      ) {
        Err(e) => return Err(e.to_string()),
        Ok(_) => ()
      }
    }
    return Ok("Gamepads should now be cleaned up.".to_string());
  }

  // A method that returns the number of pads connected to this client.
  /*
  fn get_connected(&self) -> i8 {
    let mut connected: i8 = 0;
    for pad in &self.pads {
      if let Some(gamepad_id) = pad.get_gamepad_id() {
        if self.input_reader.is_connected(&gamepad_id) {
          connected = connected + 1;
        }
      }
    }
    return connected;
  }
  */
}

/**
 * A struct representing packed data to be sent to a Switch.
 * 
 * This isn't the cleanest or most dynamic thing by any means, but I wanted it to be consistent
 * with the original data structure.
 */
pub struct PackedData {
  magic: u16,
  connected: u16,

  con_type: u16,
  keys: u64,
  joy_l_x: i32,
  joy_l_y: i32,
  joy_r_x: i32,
  joy_r_y: i32,

  con_type2: u16,
  keys2: u64,
  joy_l_x2: i32,
  joy_l_y2: i32,
  joy_r_x2: i32,
  joy_r_y2: i32,

  con_type3: u16,
  keys3: u64,
  joy_l_x3: i32,
  joy_l_y3: i32,
  joy_r_x3: i32,
  joy_r_y3: i32,

  con_type4: u16,
  keys4: u64,
  joy_l_x4: i32,
  joy_l_y4: i32,
  joy_r_x4: i32,
  joy_r_y4: i32,
}

// Maps a switch pad (or lack thereof) to its integer counterpart.
fn switch_pad_to_int(switch_pad: &Option<SwitchPad>) -> i8 {
  match switch_pad {
    Some(pad) => return pad.value(),
    None => return 0
  }
}

impl PackedData {
  // Constructs a packed data struct just from a list of pads.
  pub fn new(pads: &Vec<EmulatedPad>, connected: i8) -> PackedData {
    return PackedData {
      magic: 0x3276,
      connected: connected as u16,

      con_type: switch_pad_to_int(pads[0].get_switch_pad()) as u16,
      keys: *pads[0].get_keyout() as u64,
      joy_l_x: pads[0].get_left().0,
      joy_l_y: pads[0].get_left().1,
      joy_r_x: pads[0].get_right().0,
      joy_r_y: pads[0].get_right().1,

      con_type2: switch_pad_to_int(pads[1].get_switch_pad()) as u16,
      keys2: *pads[1].get_keyout() as u64,
      joy_l_x2: pads[1].get_left().0,
      joy_l_y2: pads[1].get_left().1,
      joy_r_x2: pads[1].get_right().0,
      joy_r_y2: pads[1].get_right().1,

      con_type3: switch_pad_to_int(pads[2].get_switch_pad()) as u16,
      keys3: *pads[2].get_keyout() as u64,
      joy_l_x3: pads[2].get_left().0,
      joy_l_y3: pads[2].get_left().1,
      joy_r_x3: pads[2].get_right().0,
      joy_r_y3: pads[2].get_right().1,

      con_type4: switch_pad_to_int(pads[3].get_switch_pad()) as u16,
      keys4: *pads[3].get_keyout() as u64,
      joy_l_x4: pads[3].get_left().0,
      joy_l_y4: pads[3].get_left().1,
      joy_r_x4: pads[3].get_right().0,
      joy_r_y4: pads[3].get_right().1,
    }
  }

  // Converts this packed data to structured bytes.
  pub fn to_bytes(&self) -> Vec<u8> {
    /* 
     * H - SwitchPad (Controller Type)
     * Q - Keyout
     * i - Stick Info 
     */
    structure!("<HHHQiiiiHQiiiiHQiiiiHQiiii").pack(
      self.magic,
      self.connected,

      self.con_type,
      self.keys,
      self.joy_l_x,
      self.joy_l_y,
      self.joy_r_x,
      self.joy_r_y,

      self.con_type2,
      self.keys2,
      self.joy_l_x2,
      self.joy_l_y2,
      self.joy_r_x2,
      self.joy_r_y2,

      self.con_type3,
      self.keys3,
      self.joy_l_x3,
      self.joy_l_y3,
      self.joy_r_x3,
      self.joy_r_y3,

      self.con_type4,
      self.keys4,
      self.joy_l_x4,
      self.joy_l_y4,
      self.joy_r_x4,
      self.joy_r_y4,
    ).unwrap()
  }
}
