use std::mem::offset_of;

use crate::input::StdFunctionLayout;

// The start of the game's `InputHandler`, up to and with `mAnyInputHandler`. Only a prefix: use it through a pointer to a
// game object.
#[repr(C)]
pub struct InputHandlerLayout {
    _head: [u8; 0xC0],
    // 0xC0 mAnyInputHandler: runs for each input event
    pub any_input: StdFunctionLayout,
}

const _: () = assert!(offset_of!(InputHandlerLayout, any_input) == 0xC0);
// the callable pointer is at InputHandler + 0xF8
const _: () = assert!(offset_of!(InputHandlerLayout, any_input) + 0x38 == 0xF8);
