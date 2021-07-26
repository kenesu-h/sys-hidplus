use crate::pad::SwitchPad;
use serde::{Serialize, Deserialize};

/**
 * A struct representing a configuration for a client.
 *
 * - rawinput_fallback decides whether the client should attempt to use a RawInput library for
 *   controllers that aren't recognized by GilRs.
 * - The switch_pads represent what Switch controller type each slot will emulate.
 */
#[derive(Serialize, Deserialize)]
pub struct Config {
  rawinput_fallback: bool,
  switch_pad_1: Option<SwitchPad>,
  switch_pad_2: Option<SwitchPad>,
  switch_pad_3: Option<SwitchPad>,
  switch_pad_4: Option<SwitchPad>
}

impl Default for Config {
  fn default() -> Config {
    return Config {
      rawinput_fallback: true,
      switch_pad_1: Some(SwitchPad::ProController),
      switch_pad_2: Some(SwitchPad::ProController),
      switch_pad_3: Some(SwitchPad::ProController),
      switch_pad_4: Some(SwitchPad::ProController)
    }
  }
}

impl Config { 
  pub fn get_rawinput_fallback(&self) -> bool {
    return self.rawinput_fallback;
  }

  pub fn pads_to_vec(&self) -> Vec<Option<SwitchPad>> {
    return vec!(
      self.switch_pad_1,
      self.switch_pad_2,
      self.switch_pad_3,
      self.switch_pad_4
    );
  }
}
