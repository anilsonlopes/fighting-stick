use fighting_stick_core::PhysicalInput;
use sdl2::{
    event::Event,
    joystick::{HatState, Joystick},
    EventPump, JoystickSubsystem, Sdl,
};
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum InputEvent {
    Connected {
        index: u32,
        guid: String,
        name: String,
    },
    Disconnected,
    Changed {
        input: PhysicalInput,
        pressed: bool,
    },
    Diagnostic(String),
}

#[derive(Debug, Error)]
pub enum InputError {
    #[error("SDL: {0}")]
    Sdl(String),
}

/// Raw joystick events ensure buttons omitted from GameControllerDB and
/// trigger-style axes remain available to Learn.
pub struct InputManager {
    _sdl: Sdl,
    joysticks: JoystickSubsystem,
    pump: EventPump,
    current: Option<Joystick>,
    current_instance: Option<u32>,
    axis_baselines: Vec<i16>,
    axis_states: Vec<i8>,
    hat_inputs: BTreeSet<PhysicalInput>,
    pending: Vec<InputEvent>,
}

impl InputManager {
    pub fn new() -> Result<Self, InputError> {
        let sdl = sdl2::init().map_err(InputError::Sdl)?;
        let joysticks = sdl.joystick().map_err(InputError::Sdl)?;
        let pump = sdl.event_pump().map_err(InputError::Sdl)?;
        let mut manager = Self {
            _sdl: sdl,
            joysticks,
            pump,
            current: None,
            current_instance: None,
            axis_baselines: vec![],
            axis_states: vec![],
            hat_inputs: BTreeSet::new(),
            pending: vec![],
        };
        manager.open_first();
        Ok(manager)
    }

    fn open_first(&mut self) {
        let Ok(count) = self.joysticks.num_joysticks() else {
            return;
        };
        for index in 0..count {
            if self.open(index).is_ok() {
                break;
            }
        }
    }

    fn open(&mut self, index: u32) -> Result<(), String> {
        let joystick = self.joysticks.open(index).map_err(|e| e.to_string())?;
        let name = joystick.name();
        let guid = joystick.guid().string();
        let instance = joystick.instance_id();
        let axes = joystick.num_axes();
        self.axis_baselines = (0..axes)
            .map(|axis| joystick.axis(axis).unwrap_or(0))
            .collect();
        self.axis_states = vec![0; axes as usize];
        self.current_instance = Some(instance);
        self.current = Some(joystick);
        self.pending
            .push(InputEvent::Connected { index, guid, name });
        self.pending.push(InputEvent::Diagnostic(format!(
            "Joystick bruto: {axes} eixos detectados"
        )));
        Ok(())
    }

    pub fn scan(&mut self) -> Vec<InputEvent> {
        let mut out = std::mem::take(&mut self.pending);
        let events: Vec<_> = self.pump.poll_iter().collect();
        for event in events {
            match event {
                Event::JoyDeviceAdded { which, .. } if self.current.is_none() => {
                    if let Err(error) = self.open(which) {
                        out.push(InputEvent::Diagnostic(format!(
                            "Falha ao abrir joystick: {error}"
                        )));
                    } else {
                        out.append(&mut self.pending);
                    }
                }
                Event::JoyDeviceRemoved { which, .. } if self.current_instance == Some(which) => {
                    self.current = None;
                    self.current_instance = None;
                    self.axis_baselines.clear();
                    self.axis_states.clear();
                    self.hat_inputs.clear();
                    out.push(InputEvent::Disconnected);
                }
                Event::JoyButtonDown {
                    which, button_idx, ..
                } if self.current_instance == Some(which) => out.push(InputEvent::Changed {
                    input: PhysicalInput::Button(button_idx),
                    pressed: true,
                }),
                Event::JoyButtonUp {
                    which, button_idx, ..
                } if self.current_instance == Some(which) => out.push(InputEvent::Changed {
                    input: PhysicalInput::Button(button_idx),
                    pressed: false,
                }),
                Event::JoyAxisMotion {
                    which,
                    axis_idx,
                    value,
                    ..
                } if self.current_instance == Some(which) => {
                    self.axis_event(axis_idx, value, &mut out)
                }
                Event::JoyHatMotion {
                    which,
                    hat_idx,
                    state,
                    ..
                } if self.current_instance == Some(which) => {
                    self.hat_event(hat_idx, state, &mut out)
                }
                _ => {}
            }
        }
        out
    }

    fn axis_event(&mut self, axis: u8, value: i16, out: &mut Vec<InputEvent>) {
        const THRESHOLD: i32 = 16_000;
        let Some(&baseline) = self.axis_baselines.get(axis as usize) else {
            return;
        };
        let delta = value as i32 - baseline as i32;
        let new_state = if delta < -THRESHOLD {
            -1
        } else if delta > THRESHOLD {
            1
        } else {
            0
        };
        let Some(old_state) = self.axis_states.get_mut(axis as usize) else {
            return;
        };
        if *old_state == new_state {
            return;
        }
        if *old_state != 0 {
            out.push(InputEvent::Changed {
                input: axis_input(axis, *old_state),
                pressed: false,
            });
        }
        if new_state != 0 {
            out.push(InputEvent::Changed {
                input: axis_input(axis, new_state),
                pressed: true,
            });
        }
        *old_state = new_state;
    }

    fn hat_event(&mut self, hat: u8, state: HatState, out: &mut Vec<InputEvent>) {
        let next = hat_directions(hat, state);
        for input in self.hat_inputs.difference(&next) {
            out.push(InputEvent::Changed {
                input: input.clone(),
                pressed: false,
            });
        }
        for input in next.difference(&self.hat_inputs) {
            out.push(InputEvent::Changed {
                input: input.clone(),
                pressed: true,
            });
        }
        self.hat_inputs = next;
    }
}

fn axis_input(axis: u8, direction: i8) -> PhysicalInput {
    match (axis, direction) {
        (0, -1) => PhysicalInput::DPadLeft,
        (0, 1) => PhysicalInput::DPadRight,
        (1, -1) => PhysicalInput::DPadUp,
        (1, 1) => PhysicalInput::DPadDown,
        (_, -1) => PhysicalInput::AxisNegative(axis),
        _ => PhysicalInput::AxisPositive(axis),
    }
}

fn hat_directions(hat: u8, state: HatState) -> BTreeSet<PhysicalInput> {
    let mut inputs = BTreeSet::new();
    let (up, down, left, right) = if hat == 0 {
        (
            PhysicalInput::DPadUp,
            PhysicalInput::DPadDown,
            PhysicalInput::DPadLeft,
            PhysicalInput::DPadRight,
        )
    } else {
        (
            PhysicalInput::AxisNegative(hat.saturating_mul(2).saturating_add(32)),
            PhysicalInput::AxisPositive(hat.saturating_mul(2).saturating_add(32)),
            PhysicalInput::AxisNegative(hat.saturating_mul(2).saturating_add(33)),
            PhysicalInput::AxisPositive(hat.saturating_mul(2).saturating_add(33)),
        )
    };
    if matches!(state, HatState::Up | HatState::RightUp | HatState::LeftUp) {
        inputs.insert(up);
    }
    if matches!(
        state,
        HatState::Down | HatState::RightDown | HatState::LeftDown
    ) {
        inputs.insert(down);
    }
    if matches!(
        state,
        HatState::Left | HatState::LeftUp | HatState::LeftDown
    ) {
        inputs.insert(left);
    }
    if matches!(
        state,
        HatState::Right | HatState::RightUp | HatState::RightDown
    ) {
        inputs.insert(right);
    }
    inputs
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn axes_have_independent_inputs() {
        assert_eq!(axis_input(0, -1), PhysicalInput::DPadLeft);
        assert_eq!(axis_input(3, 1), PhysicalInput::AxisPositive(3));
    }
    #[test]
    fn diagonal_hat_contains_two_directions() {
        let i = hat_directions(0, HatState::RightUp);
        assert!(i.contains(&PhysicalInput::DPadUp));
        assert!(i.contains(&PhysicalInput::DPadRight));
    }
}
