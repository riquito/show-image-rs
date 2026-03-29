use super::mouse_cache::MouseCache;

pub fn convert_winit_window_event(
	window_id: winit::window::WindowId,
	event: winit::event::WindowEvent,
	mouse_cache: &MouseCache,
	modifiers: winit::keyboard::ModifiersState,
) -> Option<crate::event::WindowEvent> {
	use crate::event;
	use winit::event::WindowEvent as W;

	match event {
		W::Ime(_) => None,
		W::Occluded(_) => None,
		W::SurfaceResized(size) => Some(
			event::WindowResizedEvent {
				window_id,
				size: glam::UVec2::new(size.width, size.height),
			}
			.into(),
		),
		W::Moved(position) => Some(
			event::WindowMovedEvent {
				window_id,
				position: glam::IVec2::new(position.x, position.y),
			}
			.into(),
		),
		W::CloseRequested => Some(event::WindowCloseRequestedEvent { window_id }.into()),
		W::Destroyed => Some(event::WindowDestroyedEvent { window_id }.into()),
		W::DragDropped { paths, .. } => Some(event::WindowDroppedFileEvent { window_id, files: paths }.into()),
		W::DragEntered { paths, .. } => Some(event::WindowHoveredFileEvent { window_id, files: paths }.into()),
		W::DragMoved { .. } => None,
		W::DragLeft { .. } => Some(event::WindowHoveredFileCancelledEvent { window_id }.into()),
		W::Focused(true) => Some(event::WindowFocusGainedEvent { window_id }.into()),
		W::Focused(false) => Some(event::WindowFocusLostEvent { window_id }.into()),
		W::KeyboardInput {
			device_id,
			event,
			is_synthetic,
		} => Some(
			event::WindowKeyboardInputEvent {
				window_id,
				device_id,
				input: event,
				is_synthetic,
				modifiers,
			}
			.into(),
		),
		W::ModifiersChanged(_) => None,
		W::PointerMoved { device_id, position, .. } => {
			let position = glam::DVec2::new(position.x, position.y).as_vec2();
			Some(
				event::WindowMouseMoveEvent {
					window_id,
					device_id,
					position,
					prev_position: mouse_cache.get_prev_position(window_id, device_id).unwrap_or(position),
					buttons: mouse_cache.get_buttons(device_id).cloned().unwrap_or_default(),
				}
				.into(),
			)
		},
		W::PointerEntered { device_id, .. } => Some(
			event::WindowMouseEnterEvent {
				window_id,
				device_id,
				buttons: mouse_cache.get_buttons(device_id).cloned().unwrap_or_default(),
			}
			.into(),
		),
		W::PointerLeft { device_id, .. } => Some(
			event::WindowMouseLeaveEvent {
				window_id,
				device_id,
				buttons: mouse_cache.get_buttons(device_id).cloned().unwrap_or_default(),
			}
			.into(),
		),
		W::MouseWheel { device_id, delta, phase } => Some(
			event::WindowMouseWheelEvent {
				window_id,
				device_id,
				delta,
				phase,
				position: mouse_cache.get_position(window_id, device_id),
				buttons: mouse_cache.get_buttons(device_id).cloned().unwrap_or_default(),
			}
			.into(),
		),
		W::PointerButton {
			device_id,
			state,
			button,
			position,
			..
		} => {
			let mouse_button = button.mouse_button()?;
			let pos = glam::DVec2::new(position.x, position.y).as_vec2();
			let prev_position = mouse_cache.get_prev_position(window_id, device_id).unwrap_or(pos);
			Some(
				event::WindowMouseButtonEvent {
					window_id,
					device_id,
					button: mouse_button.into(),
					state: state.into(),
					position: pos,
					prev_position,
					buttons: mouse_cache.get_buttons(device_id).cloned().unwrap_or_default(),
				}
				.into(),
			)
		},
		W::TouchpadPressure {
			device_id,
			pressure,
			stage,
		} => Some(
			event::WindowTouchpadPressureEvent {
				window_id,
				device_id,
				pressure,
				stage,
			}
			.into(),
		),
		W::ThemeChanged(theme) => Some(
			event::WindowThemeChangedEvent {
				window_id,
				theme: theme.into(),
			}
			.into(),
		),
		W::ScaleFactorChanged { scale_factor, .. } => Some(event::WindowScaleFactorChangedEvent { window_id, scale_factor }.into()),

		W::ActivationTokenDone { .. } => None,
		W::PinchGesture { device_id, delta, phase } => Some(
			event::WindowTouchpadMagnifyEvent {
				window_id,
				device_id,
				scale: 1.0 + delta,
				phase,
			}
			.into(),
		),
		W::PanGesture { .. } => None,
		W::DoubleTapGesture { .. } => None,
		W::RotationGesture { .. } => None,
		W::RedrawRequested => Some(event::WindowRedrawRequestedEvent { window_id }.into()),
	}
}

pub fn convert_winit_device_event(
	device_id: Option<winit::event::DeviceId>,
	event: winit::event::DeviceEvent,
) -> crate::event::DeviceEvent {
	use crate::event;
	use winit::event::DeviceEvent as W;

	let device_id_or_default = device_id.unwrap_or(winit::event::DeviceId::from_raw(0));

	match event {
		W::PointerMotion { delta } => event::DevicePointerMotionEvent {
			device_id: device_id_or_default,
			delta: glam::DVec2::new(delta.0, delta.1).as_vec2(),
		}
		.into(),
		W::MouseWheel { delta } => event::DeviceMouseWheelEvent {
			device_id: device_id_or_default,
			delta,
		}
		.into(),
		W::Button { button, state } => event::DeviceButtonEvent {
			device_id: device_id_or_default,
			button,
			state: state.into(),
		}
		.into(),
		W::Key(event) => event::DeviceKeyboardInputEvent {
			device_id: device_id_or_default,
			input: event,
		}
		.into(),
	}
}

impl From<winit::event::ElementState> for crate::event::ElementState {
	fn from(other: winit::event::ElementState) -> Self {
		match other {
			winit::event::ElementState::Pressed => Self::Pressed,
			winit::event::ElementState::Released => Self::Released,
		}
	}
}

impl From<winit::event::MouseButton> for crate::event::MouseButton {
	fn from(other: winit::event::MouseButton) -> Self {
		match other {
			winit::event::MouseButton::Left => Self::Left,
			winit::event::MouseButton::Right => Self::Right,
			winit::event::MouseButton::Middle => Self::Middle,
			winit::event::MouseButton::Back => Self::Back,
			winit::event::MouseButton::Forward => Self::Forward,
			_ => Self::Other(0),
		}
	}
}

impl From<winit::window::Theme> for crate::event::Theme {
	fn from(other: winit::window::Theme) -> Self {
		match other {
			winit::window::Theme::Light => Self::Light,
			winit::window::Theme::Dark => Self::Dark,
		}
	}
}
