use crate::{
  config::Config,
  input::{
    common::reader::{
      InputButton,
      InputEvent,
      InputReader,
    },
    switch::{
      SwitchPad,
      EmulatedPad
    }
  }
};
use std::{
  collections::HashMap,
  net::UdpSocket,
  time
};

/**
 * A struct representing the main input client.
 * 
 * There's a lot that goes into a client, but the bare minimum is:
 * - A socket to communicate with the input server.
 * - The IP of the input server.
 *   - This must be preserved between update calls.
 * - A way to read inputs from a general gamepad API.
 * - A list of the emulated gamepads.
 *
 * We also need these, although the reasoning behind them might be more obscure:
 * - A way to read inputs from RawInput.
 *   - This is needed for XInput-incompatible gamepads and to possibly support
 *     4+ players.
 * - HashMaps mapping gamepad IDs to the index of their corresponding emulated
 *   gamepad.
 *   - This allows controller updates to be O(n) as opposed to O(n^2).
 */
pub struct Client {
  config: Config,
  sock: UdpSocket,
  server_ip: String,

  input_reader: Box<dyn InputReader>,
  input_map: HashMap<usize, usize>,

  rawinput_reader: Box<dyn InputReader>,
  rawinput_map: HashMap<usize, usize>,

  pads: Vec<EmulatedPad>,
}

impl Client {
  /**
   * Constructs a new client from a config, and two input readers respectively
   * corresponding to general input APIs and RawInput.
   *
   * The socket itself is bound to port 8000, but no server IP is specified.
   * Empty input maps are initialized, as well as emulated gamepads with types
   * of None.
   */
  pub fn new(
    config: Config,
    input_reader: Box<dyn InputReader>, rawinput_reader: Box<dyn InputReader>
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

  // A method that sets the target server IP of this client.
  pub fn set_server_ip(&mut self, server_ip: &str) -> () {
    self.server_ip = server_ip.to_string();
  }

  /**
   * A method that updates all emulated gamepads. If RawInput fallback is
   * enabled, the client will also attempt to update RawInput gamepads.
   */
  pub fn update_all_pads(&mut self) -> () {
    self.update_pads(false);
    if self.config.get_rawinput_fallback() {
      self.update_pads(true);
    }
  }

  /**
   * A helper method that reads events from an input reader and updates
   * corresponding gamepads.
   *
   * The method will use the general gamepad API reader if RawInput fallback is
   * disabled; otherwise, the RawInput reader. Corresponding maps will be used
   * as well.
   *
   * This should be called at a fixed time interval alongside update_server().
   */
  fn update_pads(&mut self, rawinput: bool) -> () {
    let events: Vec<InputEvent>;
    if rawinput {
      events = self.rawinput_reader.read();
    } else {
      events = self.input_reader.read();
    }
    for event in events {
      let input_map: &HashMap<usize, usize>;
      if rawinput {
        input_map = &self.rawinput_map;
      } else {
        input_map = &self.input_map;
      }
      if let Some(i) = input_map.get(event.get_gamepad_id()) {
        if *self.pads[*i].get_gamepad_id() == Some(*event.get_gamepad_id()) {
          self.pads[*i].update(&event);
        }
      } else {
        if let InputEvent::GamepadButton(gamepad_id, button, value) = event {
          if button == InputButton::RightBumper && value == 1.0 {
            match self.assign_pad(&gamepad_id, rawinput) {
              Ok(msg) => println!("{}", msg),
              Err(e) => println!("{}", e)
            }
          }
        }
      }
    }
  }

  /**
   * A helper method that attempts to assign the given gamepad ID and switch pad
   * type to an open slot, while mapping said ID the corresponding index. Slots
   * are open so as long as they are not equal to None, or if the associated
   * controller is reported by the respective input reader as disconnected.
   */
  fn assign_pad(
    &mut self, gamepad_id: &usize, rawinput: bool
  ) -> Result<String, String> {
    let mut i: usize = 0;
    for pad in &mut self.pads {
      if match pad.get_gamepad_id() {
        Some(gamepad_id) => {
          !self.input_reader.is_connected(gamepad_id)
          || !self.rawinput_reader.is_connected(gamepad_id)
        },
        None => true
      } {
        match self.config.pads_to_vec()[i] {
          Some(switch_pad) => {
            if rawinput {
              self.rawinput_map.insert(*gamepad_id, i);
            } else {
              self.input_map.insert(*gamepad_id, i);
            }
            pad.connect(gamepad_id, switch_pad);
            return Ok(
              format!(
                "Gamepad (id: {}) connected to slot {}. RawInput: {}",
                &gamepad_id,
                i + 1,
                rawinput
              )
            );
          },
          None => ()
        }
      }
      i = i + 1;
    }
    return Err(
      "Couldn't assign gamepad since there were no slots available.".to_string()
    );
  }

  /**
   * A method that sends the current emulated pad states to the Switch.
   *
   * Like update_pads(), this should be called at a fixed time interval.
   */
  pub fn update_server(&self) -> Result<(), String> {
    match self.sock.send_to(
      &PackedData::new(&self.pads, 4).to_bytes(),
      format!("{}:8000", self.server_ip)
    ) {
      Err(e) => return Err(
        format!("The following error occurred: {}.", e)
      ),
      Ok(_) => Ok(())
    }
  }

  /**
   * A method disconnects all connected gamepads.
   *
   * This unfortunately uses a brute-force approach of disconnecting all the
   * gamepads, but there's no other way that doesn't involve modifying the
   * server. For now, a list of gamepads (all set to None) will be spammed over
   * the course of 3 seconds in order for shit to somehow stick onto the wall.
   * This hasn't failed so far, but this may change if a network happens to be
   * unstable.
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
}

/**
 * A struct representing packed data to be sent to a Switch.
 * 
 * This isn't the cleanest or most dynamic thing by any means, but I wanted it
 * to be consistent with the original data structure.
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
