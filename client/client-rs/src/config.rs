use crate::input::SwitchPad;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
  switch_pad_1: Option<SwitchPad>,
  switch_pad_2: Option<SwitchPad>,
  switch_pad_3: Option<SwitchPad>,
  switch_pad_4: Option<SwitchPad>
}

impl Default for Config {
  fn default() -> Config {
    return Config {
      switch_pad_1: Some(SwitchPad::ProController),
      switch_pad_2: Some(SwitchPad::ProController),
      switch_pad_3: Some(SwitchPad::ProController),
      switch_pad_4: Some(SwitchPad::ProController)
    }
  }
}

impl Config { 
  pub fn to_vec(&self) -> Vec<Option<SwitchPad>> {
    return vec!(
      self.switch_pad_1,
      self.switch_pad_2,
      self.switch_pad_3,
      self.switch_pad_4
    );
  }
}