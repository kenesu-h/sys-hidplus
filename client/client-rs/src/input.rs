use gilrs::{
  Gilrs,
};
use multiinput::RawEvent;
use std::convert::TryInto;

// An enum representing the buttons that are universally available on gamepads; I'd hope so, anyway.
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

// An enum representing the axes that are universally available on gamepads.
#[derive(Debug)]
pub enum InputAxis {
  LeftX,
  LeftY,
  RightX,
  RightY
}

// An enum representing the different events possible on a gamepad.
#[derive(Debug)]
pub enum InputEvent {
  GamepadButton(usize, InputButton, f32),
  GamepadAxis(usize, InputAxis, f32)
}

impl InputEvent {
  // A method that returns that the gamepad ID of this event.
  pub fn get_gamepad_id(&self) -> &usize {
    return match self {
      Self::GamepadButton(gamepad_id, _, _) => gamepad_id,
      Self::GamepadAxis(gamepad_id, _, _) => gamepad_id
    }
  }
}

/**
 * A trait representing a input reader that reads from an gamepad input library of some kind,
 * from which an input event can be generated.
 */
pub trait InputReader {
  // A method that reads from an input library's buffer and returns the buffered events.
  fn read(&mut self) -> Vec<InputEvent>;

  // A method that checks the input library to verify if a gamepad of a given ID is connected.
  fn is_connected(&mut self, gamepad_id: &usize) -> bool;
}

/**
 * A struct representing a cross-platform input reader that will read from a GilRs instance.
 * 
 * As of the time of documentation, GilRs does not support any other gamepad APIs on Windows other
 * than XInput, and as a result will not support more than 4 gamepads. This has only been tested on
 * Windows as well, but should theoretically work with Unix OS's.
 */
pub struct GilrsInputReader {
  gilrs: Gilrs
}

impl GilrsInputReader {
  // Constructs a GilRs input reader with an accompanying GilRs instance.
  pub fn new() -> GilrsInputReader {
    return GilrsInputReader {
      gilrs: Gilrs::new().unwrap()
    }
  }

  // A helper method to convert GilRs buttons into InputButtons.
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

  // A helper method to convert GilRs axes into InputAxes.
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
  fn read(&mut self) -> Vec<InputEvent> {
    let mut events: Vec<InputEvent> = vec!();
    while let Some(gilrs::Event { id: gamepad_id, event: event_type, time: _ }) = self.gilrs.next_event() {
      match event_type {
        gilrs::EventType::ButtonChanged(button, value, _) => {
          events.push(InputEvent::GamepadButton(
            gamepad_id.try_into().unwrap(),
            // TODO: Change this (and the axis branch) to match that of the multiinput alternative.
            self.to_button(&button).unwrap(),
            value
          ))
        },
        gilrs::EventType::AxisChanged(axis, value, _) => {
          events.push(InputEvent::GamepadAxis(
            gamepad_id.try_into().unwrap(),
            self.to_axis(&axis).unwrap(),
            value
          ))
        },
        _ => ()
      }
    }
    return events;
  }

  /* This could possibly be more efficient if we could turn a usize into a GamepadId, but we can't.
   * Until then, this is gonna have to be O(n).
   */
  fn is_connected(&mut self, gamepad_id: &usize) -> bool {
    for (id, _) in self.gilrs.gamepads() {
      if *gamepad_id == id.try_into().unwrap() {
        return true;
      }
    }
    return false;
  }
}

/**
 * A struct representing a RawInput input reader that will read from the multiinput library using an
 * instance of an input manager.
 * 
 * This input reader is ONLY meant to be used for RawInput devices, and at the time of writing this,
 * has only been tested with DS4s (PS4 controllers). XInput support is poor right now and gamepads
 * other than the DS4 have not been tested. Do not expect an exquisite amount of support from this.
 */
pub struct MultiInputReader {
  manager: multiinput::RawInputManager
}

impl MultiInputReader {
  /**
   * Constructs a multiinput reader with an input manager instance.
   * 
   * This input manager instance will not read from XInput devices or mouse & keyboard, although
   * the options exist and may be implemented in a later update.
   */
  pub fn new() -> MultiInputReader {
    let mut manager: multiinput::RawInputManager = multiinput::RawInputManager::new().unwrap();
    manager.register_devices(
      multiinput::DeviceType::Joysticks(
        /*
         * This was initially true, but XInput controller support was poor and there was no way to
         * return the type of a controller.
         */
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
      multiinput::Axis::Z => Ok(InputAxis::RightX),
      multiinput::Axis::RZ => Ok(InputAxis::RightY),
      _ => Err(format!("{:?} is currently an unmapped multiinput axis.", axis))
    }
  }

  /**
   * A method that "corrects" a value for an axis, assuming the gamepad involved is a DS4.
   * 
   * For some reason, the right stick uses the Z and RZ axes; Z for horizontal and RZ for
   * vertical. Their values also happen to be inverted, unlike the left stick. We use this
   * method to invert the value back if it happens to be Z or RZ.
   */
  fn correct_axis_value(&self, axis: &multiinput::Axis, value: &f64) -> f32 {
    return match axis {
      multiinput::Axis::Z | multiinput::Axis::RZ => -(*value as f32),
      _ => *value as f32
    }
  }

  fn to_dpad(&self, hat_switch: &multiinput::HatSwitch) -> Vec<(InputButton, f32)> {
    return match hat_switch {
      // TODO: I might just make a struct for this since this is unnecessarily long.
      multiinput::HatSwitch::Center => vec!(
        (InputButton::DPadUp, 0.0),
        (InputButton::DPadDown, 0.0),
        (InputButton::DPadLeft, 0.0),
        (InputButton::DPadRight, 0.0)
      ),
      multiinput::HatSwitch::Up => vec!(
        (InputButton::DPadUp, 1.0),
        (InputButton::DPadDown, 0.0),
        (InputButton::DPadLeft, 0.0),
        (InputButton::DPadRight, 0.0)
      ),
      multiinput::HatSwitch::UpRight => vec!(
        (InputButton::DPadUp, 1.0),
        (InputButton::DPadDown, 0.0),
        (InputButton::DPadLeft, 0.0),
        (InputButton::DPadRight, 1.0)
      ),
      multiinput::HatSwitch::Right => vec!(
        (InputButton::DPadUp, 0.0),
        (InputButton::DPadDown, 0.0),
        (InputButton::DPadLeft, 0.0),
        (InputButton::DPadRight, 1.0)
      ),
      multiinput::HatSwitch::DownRight => vec!(
        (InputButton::DPadUp, 0.0),
        (InputButton::DPadDown, 1.0),
        (InputButton::DPadLeft, 0.0),
        (InputButton::DPadRight, 1.0)
      ),
      multiinput::HatSwitch::Down => vec!(
        (InputButton::DPadUp, 0.0),
        (InputButton::DPadDown, 1.0),
        (InputButton::DPadLeft, 0.0),
        (InputButton::DPadRight, 0.0)
      ),
      multiinput::HatSwitch::DownLeft => vec!(
        (InputButton::DPadUp, 0.0),
        (InputButton::DPadDown, 1.0),
        (InputButton::DPadLeft, 1.0),
        (InputButton::DPadRight, 0.0)
      ),
      multiinput::HatSwitch::Left => vec!(
        (InputButton::DPadUp, 0.0),
        (InputButton::DPadDown, 0.0),
        (InputButton::DPadLeft, 1.0),
        (InputButton::DPadRight, 0.0)
      ),
      multiinput::HatSwitch::UpLeft => vec!(
        (InputButton::DPadUp, 1.0),
        (InputButton::DPadDown, 0.0),
        (InputButton::DPadLeft, 1.0),
        (InputButton::DPadRight, 0.0)
      ),
    }
  }

  pub fn parse_buffered(&mut self, buffered: Vec<RawEvent>) -> Vec<InputEvent> {
    let mut events: Vec<InputEvent> = vec!();
    for event in buffered {
      match event {
        multiinput::event::RawEvent::JoystickButtonEvent(device_id, button, state) => {
          match self.to_button_event(&device_id, &button, &state) {
            Ok(mapped_event) => events.push(mapped_event),
            Err(_) => ()
          }
        },
        multiinput::event::RawEvent::JoystickAxisEvent(device_id, axis, value) => {
          match self.to_axis_event(&device_id, &axis, &value) {
            Ok(mapped_event) => events.push(mapped_event),
            Err(_) => ()
          }
        },
        multiinput::event::RawEvent::JoystickHatSwitchEvent(device_id, hat_switch) => {
          let pairs: Vec<(InputButton, f32)> = self.to_dpad(&hat_switch);
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
        _ => ()
      }
    }
    return events;
  }

  pub fn to_button_event(
    &self, device_id: &usize, button: &usize, state: &multiinput::State
  ) -> Result<InputEvent, String> {
    return match self.to_button(button) {
      Ok(mapped) => Ok(
        InputEvent::GamepadButton(
          *device_id,
          mapped,
          self.to_button_value(state)
        )
      ),
      Err(e) => Err(e)
    }
  }

  pub fn to_axis_event(
    &self, device_id: &usize, axis: &multiinput::Axis, value: &f64
  ) -> Result<InputEvent, String> {
    return match self.to_axis(axis) {
      Ok(mapped) => Ok(
        InputEvent::GamepadAxis(
          *device_id,
          mapped,
          self.correct_axis_value(axis, value)
        )
      ),
      Err(e) => Err(e)
    }
  }
}

impl InputReader for MultiInputReader {
  fn read(&mut self) -> Vec<InputEvent> {
    let mut buffered: Vec<RawEvent> = vec!();
    while let Some(event) = self.manager.get_event() {
      buffered.push(event); 
    }
    return self.parse_buffered(buffered);
  }

  fn is_connected(&mut self, gamepad_id: &usize) -> bool {
    return self.manager.get_joystick_state(*gamepad_id).is_some();
  }
}
