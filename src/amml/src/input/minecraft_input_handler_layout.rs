use std::ffi::c_void;
use std::mem::offset_of;

// The start of the game's `MinecraftInputHandler`, up to and with `mInputHandler`. Only a prefix: use it through a pointer
// to a game object.
#[repr(C)]
pub struct MinecraftInputHandlerLayout {
    _head: [u8; 0x20],
    // 0x20 mInputHandler (a unique_ptr to the InputHandler)
    pub input_handler: *mut c_void,
}

const _: () = assert!(offset_of!(MinecraftInputHandlerLayout, input_handler) == 0x20);
