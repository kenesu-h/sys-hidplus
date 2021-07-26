use gilrs::{
  Gilrs,
};
use std::convert::TryInto;

#[derive(PartialEq, Debug)]
pub enum InputButton {
  North,
  South,
  East,
  West,
  LeftBumper,
  LeftTrigger,
  RightBumper,
  RightTrigger,
  Start,
  Select,
  DPadUp,
  DPadDown,
  DPadLeft,
  DPadRight
}

#[derive(Debug)]
pub enum InputAxis {
  LeftX,
  LeftY,
  RightX,
  RightY
}

#[derive(Debug)]
pub enum InputEvent {
  GamepadButton(usize, InputButton, f32),
  GamepadAxis(usize, InputAxis, f32)
}

impl InputEvent {
  pub fn get_gamepad_id(&self) -> &usize {
    return match self {
      Self::GamepadButton(gamepad_id, _, _) => gamepad_id,
      Self::GamepadAxis(gamepad_id, _, _) => gamepad_id
    }
  }
}

pub trait InputReader {
  fn read(&mut self) -> Result<Vec<InputEvent>, String>;

  fn is_connected(&mut self, gamepad_id: &usize) -> bool;
}

pub struct GilrsInputReader {
  gilrs: Gilrs
}

impl GilrsInputReader {
  pub fn new() -> GilrsInputReader {
    return GilrsInputReader {
      gilrs: Gilrs::new().unwrap()
    }
  }

  fn to_button(&self, button: &gilrs::Button) -> Result<InputButton, String> {
    return match button {
      gilrs::Button::South => Ok(InputButton::South),
      gilrs::Button::East => Ok(InputButton::East),
      gilrs::Button::North => Ok(InputButton::North),
      gilrs::Button::West => Ok(InputButton::West),
      gilrs::Button::LeftTrigger => Ok(InputButton::LeftBumper),
      gilrs::Button::LeftTrigger2 => Ok(InputButton::LeftTrigger),
      gilrs::Button::RightTrigger => Ok(InputButton::RightBumper),
      gilrs::Button::RightTrigger2 => Ok(InputButton::RightTrigger),
      gilrs::Button::Start => Ok(InputButton::Start),
      gilrs::Button::Select => Ok(InputButton::Select),
      gilrs::Button::DPadUp => Ok(InputButton::DPadUp),
      gilrs::Button::DPadDown => Ok(InputButton::DPadDown),
      gilrs::Button::DPadLeft => Ok(InputButton::DPadLeft),
      gilrs::Button::DPadRight => Ok(InputButton::DPadRight),
      _ => Err(format!("{:?} is currently an unmapped GilRs button.", button))
    }
  }

  fn to_axis(&self, axis: &gilrs::Axis) -> Result<InputAxis, String> {
    return match axis {
      gilrs::Axis::LeftStickX => Ok(InputAxis::LeftX),
      gilrs::Axis::LeftStickY => Ok(InputAxis::LeftY),
      gilrs::Axis::RightStickX => Ok(InputAxis::RightX),
      gilrs::Axis::RightStickY => Ok(InputAxis::RightY),
      _ => Err(format!("{:?} is currently an unmapped GilRs axis.", axis))
    }
  }
}

impl InputReader for GilrsInputReader {
  fn read(&mut self) -> Result<Vec<InputEvent>, String> {
    let mut events: Vec<InputEvent> = vec!();
    while let Some(gilrs::Event { id: gamepad_id, event: event_type, time: _ }) = self.gilrs.next_event() {
      events.push(
        match event_type {
          gilrs::EventType::ButtonChanged(button, value, _) => {
            InputEvent::GamepadButton(
              gamepad_id.try_into().unwrap(),
              self.to_button(&button).unwrap(),
              value as f32
            )
          },
          gilrs::EventType::AxisChanged(axis, value, _) => {
            InputEvent::GamepadAxis(
              gamepad_id.try_into().unwrap(),
              self.to_axis(&axis).unwrap(),
              value as f32
            )
          },
          _ => return Err(
            format!("{:?} is currently an unsupported Gilrs event type.", event_type)
          )
        }
      );
    }
    return Ok(events);
  }

  // The O(n) nature of this method makes its usage in client.rs O(n^2). Not great.
  // Granted, this is only done when someone wants to assign a controller, which means that its
  // usage isn't O(n^2) a large majority of the time. I still want to optimize this if we can,
  // though.

  // This could possibly be more efficient if we could turn a usize into a GamepadId, but we can't.
  fn is_connected(&mut self, gamepad_id: &usize) -> bool {
    for (id, _) in self.gilrs.gamepads() {
      if *gamepad_id == id.try_into().unwrap() {
        return true;
      }
    }
    return false;
  }
}

pub struct MultiInputReader {
  manager: multiinput::RawInputManager
}

impl MultiInputReader {
  pub fn new() -> MultiInputReader {
    let mut manager: multiinput::RawInputManager = multiinput::RawInputManager::new().unwrap();
    manager.register_devices(
      multiinput::DeviceType::Joysticks(
        // This was initially true, but it was way too hard to get controller types.
        multiinput::XInputInclude::False
      )
    );
    return MultiInputReader {
      manager: manager
    }
  }
  
  fn to_button(&self, button: &usize) -> Result<InputButton, String> {
    return match button {
      0 => Ok(InputButton::West),
      1 => Ok(InputButton::South),
      2 => Ok(InputButton::East),
      3 => Ok(InputButton::North),
      4 => Ok(InputButton::LeftBumper),
      5 => Ok(InputButton::RightBumper),
      6 => Ok(InputButton::LeftTrigger),
      7 => Ok(InputButton::RightTrigger),
      8 => Ok(InputButton::Select),
      9 => Ok(InputButton::Start),
      _ => Err(format!("{:?} is currently an unmapped multiinput button.", button))
    }
  }

  fn to_button_value(&self, state: &multiinput::State) -> f32 {
    return match state {
      multiinput::State::Pressed => 1.0,
      multiinput::State::Released => 0.0
    }
  }

  fn to_axis(&self, axis: &multiinput::Axis) -> Result<InputAxis, String> {
    return match axis {
      multiinput::Axis::X => Ok(InputAxis::LeftX),
      multiinput::Axis::Y => Ok(InputAxis::LeftY),
      multiinput::Axis::RX => Ok(InputAxis::RightX),
      multiinput::Axis::RY => Ok(InputAxis::RightY),
      _ => Err(format!("{:?} is currently an unmapped multiinput axis.", axis))
    }
  }

  fn to_dpad(&self, hat_switch: &multiinput::HatSwitch) -> Vec<(InputButton, f32)> {
    return match hat_switch {
      multiinput::HatSwitch::Center => vec!(),
      multiinput::HatSwitch::Up => vec!((InputButton::DPadUp, 1.0)),
      multiinput::HatSwitch::UpRight => vec!(
        (InputButton::DPadUp, 1.0),
        (InputButton::DPadRight, 1.0)
      ),
      multiinput::HatSwitch::Right => vec!((InputButton::DPadRight, 1.0)),
      multiinput::HatSwitch::DownRight => vec!(
        (InputButton::DPadDown, 1.0),
        (InputButton::DPadRight, 1.0)
      ),
      multiinput::HatSwitch::Down => vec!((InputButton::DPadDown, 1.0)),
      multiinput::HatSwitch::DownLeft => vec!(
        (InputButton::DPadDown, 1.0),
        (InputButton::DPadLeft, 1.0)
      ),
      multiinput::HatSwitch::Left => vec!((InputButton::DPadLeft, 1.0)),
      multiinput::HatSwitch::UpLeft => vec!(
        (InputButton::DPadUp, 1.0),
        (InputButton::DPadLeft, 1.0)
      )
    }
  }
}

impl InputReader for MultiInputReader {
  fn read(&mut self) -> Result<Vec<InputEvent>, String> {
    let mut events: Vec<InputEvent> = vec!();
    while let Some(raw_event) = self.manager.get_event() {
      match raw_event {
        multiinput::event::RawEvent::JoystickButtonEvent(device_id, button, state) => {
          match self.to_button(&button) {
            Ok(converted) => events.push(
              InputEvent::GamepadButton(
                device_id,
                converted,
                self.to_button_value(&state)
              )
            ),
            Err(e) => println!("{}", e)
          }
        },
        multiinput::event::RawEvent::JoystickAxisEvent(device_id, axis, value) => {
          match self.to_axis(&axis) {
            Ok(converted) => events.push(
              InputEvent::GamepadAxis(
                device_id,
                converted,
                value as f32
              )
            ),
            Err(_) => ()
          }
        },
        /* Fuck dpad inputs, do them later.
        multiinput::event::RawEvent::JoystickHatSwitchEvent(device_id, hat_switch) => {
          let pairs: Vec<(InputButton, f32)> = self.to_dpad(&hat_switch);
          // I'd consider this O(n^2), but the max size of pairs will only ever be 2.
          for (button, value) in pairs {
            events.push(
              InputEvent::GamepadButton(
                device_id,
                button,
                value
              )
            )
          }
        }
        */
        _ => return Err(
          format!("{:?} is currently an unsupported multiinput event type.", raw_event)
        )
      }
    }
    return Ok(events);
  }

  fn is_connected(&mut self, gamepad_id: &usize) -> bool {
    return self.manager.get_joystick_state(*gamepad_id).is_some();
  }
}
