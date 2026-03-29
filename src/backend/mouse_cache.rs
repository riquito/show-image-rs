use std::collections::BTreeMap;
use winit::event::{DeviceId, ElementState, WindowEvent};

use crate::event::MouseButtonState;
use crate::WindowId;

#[derive(Default)]
pub struct MouseCache {
	mouse_buttons: BTreeMap<DeviceId, MouseButtonState>,
	mouse_position: BTreeMap<(WindowId, DeviceId), glam::Vec2>,
	mouse_prev_position: BTreeMap<(WindowId, DeviceId), glam::Vec2>,
}

impl MouseCache {
	pub fn get_position(&self, window_id: WindowId, device_id: Option<DeviceId>) -> Option<glam::Vec2> {
		let device_id = device_id?;
		self.mouse_position.get(&(window_id, device_id)).copied()
	}

	pub fn get_prev_position(&self, window_id: WindowId, device_id: Option<DeviceId>) -> Option<glam::Vec2> {
		let device_id = device_id?;
		self.mouse_prev_position.get(&(window_id, device_id)).copied()
	}

	pub fn get_buttons(&self, device_id: Option<DeviceId>) -> Option<&MouseButtonState> {
		let device_id = device_id?;
		self.mouse_buttons.get(&device_id)
	}

	pub fn handle_window_event(&mut self, window_id: WindowId, event: &WindowEvent) {
		match event {
			WindowEvent::PointerButton {
				device_id: Some(device_id),
				button,
				state,
				..
			} => {
				if let Some(mouse_button) = button.clone().mouse_button() {
					let buttons = self.mouse_buttons.entry(*device_id).or_default();
					buttons.set_pressed(mouse_button.into(), *state == ElementState::Pressed);
				}
			},
			WindowEvent::PointerMoved {
				device_id: Some(device_id),
				position,
				..
			} => {
				let cached_position = self
					.mouse_position
					.entry((window_id, *device_id))
					.or_insert_with(|| [0.0, 0.0].into());
				let cached_prev_position = self
					.mouse_prev_position
					.entry((window_id, *device_id))
					.or_insert_with(|| [0.0, 0.0].into());
				*cached_prev_position = *cached_position;
				*cached_position = glam::DVec2::new(position.x, position.y).as_vec2();
			},
			_ => {},
		}
	}
}
