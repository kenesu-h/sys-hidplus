extern crate sdl2;

use crate::input::common::reader::{
  InputButton,
  InputAxis,
  InputEvent,
  InputReader
};

use sdl2::{
  Sdl,
  GameControllerSubsystem,
  EventPump,
  event::Event,
  controller::{
    GameController,
    Button,
    Axis
  },
  VideoSubsystem
};

use std::collections::HashMap;

pub struct SdlAdapter {
  gamepads: HashMap<u32, GameController>,
  game_controller: GameControllerSubsystem,
  event_pump: EventPump,
  // I literally have no clue why, but you need this for hotplugging.
  _video: VideoSubsystem
}

impl SdlAdapter {
  pub fn new() -> SdlAdapter {
    let sdl_context: Sdl = sdl2::init().unwrap();

    let game_controller: GameControllerSubsystem =
      sdl_context.game_controller().unwrap();
    let event_pump: EventPump = sdl_context.event_pump().unwrap();
    let video: VideoSubsystem = sdl_context.video().unwrap();
    return SdlAdapter {
      gamepads: HashMap::new(),
      game_controller: game_controller,
      event_pump: event_pump,
      _video: video
    }
  }

  fn to_button_event(
    &self, which: &u32, button: &Button, pressed: bool
  ) -> Result<InputEvent, String> {
    return match self.to_button(button) {
      Ok(mapped) => Ok(
        InputEvent::GamepadButton(
          *which as usize,
          mapped,
          self.to_button_value(pressed)
        )
      ),
      Err(e) => Err(e)
    }
  }

  fn to_button_value(&self, pressed: bool) -> f32 {
    return match pressed {
      true => 1.0,
      false => 0.0
    }
  }

  fn to_button(&self, button: &Button) -> Result<InputButton, String> {
    return match button {
      Button::A => Ok(InputButton::South),
      Button::B => Ok(InputButton::East),
      Button::X => Ok(InputButton::North),
      Button::Y => Ok(InputButton::West),
      Button::Back => Ok(InputButton::Select),
      Button::Start => Ok(InputButton::Start),
      Button::LeftShoulder => Ok(InputButton::LeftBumper),
      Button::RightShoulder => Ok(InputButton::RightBumper),
      Button::DPadUp => Ok(InputButton::DPadUp),
      Button::DPadDown => Ok(InputButton::DPadDown),
      Button::DPadLeft => Ok(InputButton::DPadLeft),
      Button::DPadRight => Ok(InputButton::DPadRight),
      _ => Err(
        format!("{:?} is currently an unmapped SDL button.", button)
      )
    }
  }

  fn to_axis_event(
    &self, which: &u32, axis: &Axis, value: &i16
  ) -> Result<InputEvent, String> {
    return match self.to_axis(axis) {
      Ok(mapped) => Ok(
        InputEvent::GamepadAxis(
          *which as usize,
          mapped,
          self.to_axis_value(&axis, value)
        )
      ),
      Err(e) => Err(e)
    }
  }

  fn to_axis(&self, axis: &Axis) -> Result<InputAxis, String> {
    return match axis {
      Axis::LeftX => Ok(InputAxis::LeftX),
      Axis::LeftY => Ok(InputAxis::LeftY),
      Axis::RightX => Ok(InputAxis::RightX),
      Axis::RightY => Ok(InputAxis::RightY),
      _ => Err(
        format!("{:?} is currently an unmapped SDL axis.", axis)
      )
    }
  } 

  fn to_axis_value(&self, axis: &Axis, value: &i16) -> f32 {
    let calculated: f32 = (*value as f32) / 32767.0;
    return match axis {
      Axis::LeftY | Axis::RightY => -calculated,
      _ => calculated
    }
  }

  fn is_trigger(&self, axis: &Axis) -> bool {
    return match axis {
      Axis::TriggerLeft | Axis::TriggerRight => true,
      _ => false
    }
  }

  fn to_trigger(&self, axis: &Axis) -> Result<InputButton, String> {
    return match axis {
      Axis::TriggerLeft => Ok(InputButton::LeftTrigger),
      Axis::TriggerRight => Ok(InputButton::RightTrigger),
      _ => Err(
        format!("{:?} are not triggers.", axis)
      )
    }
  }

  fn to_trigger_event(
    &self, which: &u32, axis: &Axis, value: &i16
  ) -> Result<InputEvent, String> {
    return match self.to_trigger(axis) {
      Ok(mapped) => Ok(
        InputEvent::GamepadButton(
          *which as usize,
          mapped,
          self.to_button_value(value > &0)
        )
      ),
      Err(e) => Err(e)
    }
  }
}

impl InputReader for SdlAdapter {
  fn read(&mut self) -> Vec<InputEvent> {
    let mut events: Vec<InputEvent> = vec!();
    while let Some(event) = self.event_pump.poll_event() {
      match event { 
        Event::ControllerDeviceAdded { which, .. } => {
          // We need to store the gamepad somewhere to receive button events.
          let gamepad: GameController = self.game_controller.open(which)
            .unwrap();
          self.gamepads.insert(which, gamepad);
        },
        Event::ControllerDeviceRemoved { which, .. } => {
          self.gamepads.remove(&which);
        },
        Event::ControllerAxisMotion { timestamp: _, which, axis, value } => {
          if self.is_trigger(&axis) {
            match self.to_trigger_event(&which, &axis, &value) {
              Ok(adapted) => events.push(adapted),
              Err(_) => ()
            }
          } else {
            match self.to_axis_event(&which, &axis, &value) {
              Ok(adapted) => events.push(adapted),
              Err(_) => ()
            }
          }
        },
        Event::ControllerButtonDown { timestamp: _, which, button } => {
          match self.to_button_event(&which, &button, true) {
            Ok(adapted) => events.push(adapted),
            Err(_) => ()
          }
        },
        Event::ControllerButtonUp {timestamp: _, which, button } => {
          match self.to_button_event(&which, &button, false) {
            Ok(adapted) => events.push(adapted),
            Err(_) => ()
          }
        },
        _ => ()
      }
    }
    return events;
  }
  
  fn is_connected(&mut self, gamepad_id: &usize) -> bool {
    return (&mut self.game_controller).open(*gamepad_id as u32).is_ok();
  }
}
