use super::ButtonId;
use super::DeviceId;
use super::ElementState;
use super::MouseScrollDelta;

/// Raw hardware events that are not associated with any particular window.
///
/// Useful for interactions that diverge significantly from a conventional 2D GUI, such as 3D camera or first-person game controls.
/// Many physical actions, such as mouse movement, can produce both device and window events.
/// Because window events typically arise from virtual devices (corresponding to GUI cursors and keyboard focus) the device IDs may not match.
///
/// Note that these events are delivered regardless of input focus.
#[derive(Debug, Clone)]
pub enum DeviceEvent {
	/// Change in physical position of a pointing device.
	PointerMotion(DevicePointerMotionEvent),

	/// The scroll-wheel of a mouse was moved.
	MouseWheel(DeviceMouseWheelEvent),

	/// A button on a device was pressed or released.
	Button(DeviceButtonEvent),

	/// A device generated keyboard input.
	KeyboardInput(DeviceKeyboardInputEvent),
}

/// The physical position of a pointing device was moved.
///
/// This represents raw, unfiltered physical motion.
/// Not to be confused with [`WindowMouseMoveEvent`][super::WindowMouseMoveEvent].
#[derive(Debug, Clone)]
pub struct DevicePointerMotionEvent {
	/// The ID of the device.
	pub device_id: DeviceId,

	/// The relative motion.
	pub delta: glam::Vec2,
}

/// The scroll-wheel of a mouse was moved.
#[derive(Debug, Clone)]
pub struct DeviceMouseWheelEvent {
	/// The ID of the device.
	pub device_id: DeviceId,

	/// The scroll delta.
	pub delta: MouseScrollDelta,
}

/// A button on a device was pressed or released.
#[derive(Debug, Clone)]
pub struct DeviceButtonEvent {
	/// The ID of the device.
	pub device_id: DeviceId,

	/// The button that was pressed or released.
	pub button: ButtonId,

	/// The new state of the button (pressed or released).
	pub state: ElementState,
}

/// A device generated keyboard input.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DeviceKeyboardInputEvent {
	/// The ID of the device.
	pub device_id: DeviceId,

	/// The event that occured.
	pub input: super::RawKeyEvent,
}

impl_from_variant!(DeviceEvent::PointerMotion(DevicePointerMotionEvent));
impl_from_variant!(DeviceEvent::MouseWheel(DeviceMouseWheelEvent));
impl_from_variant!(DeviceEvent::Button(DeviceButtonEvent));
impl_from_variant!(DeviceEvent::KeyboardInput(DeviceKeyboardInputEvent));
