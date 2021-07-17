use crate::input::{SwitchPad, EmulatedPad};

use gilrs::{
  Gilrs,
  Event,
  GamepadId,
  Button
};
use std::{
  net::UdpSocket,
  time,
  thread
};

/**
 * A struct representing the main input client.
 * 
 * An input client should have:
 * - A way to access gamepad events (a GilRs instance in this case).
 * - A list of emulated pads.
 */
pub struct Client {
  gilrs: Gilrs,
  pads: Vec<EmulatedPad>,
}

impl Client {
  // Constructs a client with a GilRs instance and a fixed amount of disconnected emulated pads.
  pub fn new() -> Client {
    return Client {
      gilrs: Gilrs::new().unwrap(),
      pads: c![EmulatedPad::new(), for i in 0..4]
    }
  }
 
  /**
   * A method that attempts to assign the given gamepad id and switch pad type to an open slot.
   * If there's no open slot, we return an error.
   */
  fn assign_pad(&mut self, gamepad_id: &GamepadId, switch_pad: SwitchPad) -> Result<&str, &str> {
    let mut slot: i8 = 1;
    for pad in &mut self.pads {
      if !pad.is_connected(&mut self.gilrs) {
        pad.connect(gamepad_id, switch_pad);
        return Ok(format!("Gamepad (id: {}) connected to slot {}", gamepad_id, slot));
      }
      slot = slot + 1;
    }
    return Err("Couldn't assign controller since there were no slots available.")
  }

  /**
   * A method that starts the primary loop of this client.
   * 
   * At a fixed time interval (1/60s, but may differ for different games), the loop should:
   * - Poll gamepad inputs and assigning unassigned gamepads to a slot.
   * - If we're sending inputs to a Switch, send inputs.
   */
  pub fn start(&mut self, ip: &str, online: bool) -> () {
    // 0.0.0.0 will be bound to localhost, don't worry
    let sock: UdpSocket = UdpSocket::bind("0.0.0.0:8000").unwrap(); 
    loop {
      while let Some(Event { id: gamepad_id, event, time: _ }) = self.gilrs.next_event() {
        let mut gamepad_mapped: bool = false; 
        for pad in &mut self.pads {
          if pad.is_connected(&mut self.gilrs)
          && pad.get_gamepad_id().unwrap() == gamepad_id {
            gamepad_mapped = true;
            pad.update(&event);
          }
        }
        if !gamepad_mapped {
          if let Some(gamepad) = Some(gamepad_id).map(|id| self.gilrs.gamepad(id)) {
            if gamepad.is_pressed(Button::LeftTrigger2)
            && gamepad.is_pressed(Button::RightTrigger2) {
              // TODO: Properly reset controllers if they've been disconnected by the Switch
              // Might be able to do this by temporarily setting a pad's switch_pad to None for one
              // tick, then setting it to the original value the next tick. This hasn't worked for
              // me when I tried it though.
              match self.assign_pad(&gamepad_id, SwitchPad::ProController) {
                Err(e) => println!("{}", e),
                Ok(msg) => println!("{}", msg)
              }
            }
          } 
        }
      }
      if online {
        let connected: i8 = self.get_connected();
        match sock.send_to(
          &PackedData::new(&self.pads, connected).to_bytes(),
          format!("{}:8000", ip)
        ) {
          Err(e) => println!("{}", e),
          Ok(_) => ()
        }
      }
      thread::sleep(time::Duration::from_secs_f32(1.0 / 60.0));
    }
  }

  // A method that returns the number of pads connected to this client.
  fn get_connected(&mut self) -> i8 {
    let mut connected: i8 = 0;
    for pad in &self.pads {
      if pad.is_connected(&mut self.gilrs) {
        connected = connected + 1;
      }
    }
    return connected;
  }
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