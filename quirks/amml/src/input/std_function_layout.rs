use std::ffi::c_void;
use std::mem::offset_of;

// The start of the MSVC `std::function` (64 bytes): the small-buffer storage, then the pointer to the callable object. The
// callable's `_Do_call` is slot 2 of its vtable. Only a prefix: use it through a pointer into a game object.
#[repr(C)]
pub struct StdFunctionLayout {
    _storage: [u8; 0x38],
    // 0x38 the `_Impl` pointer (null when the function is empty)
    pub callable: *mut c_void,
}

impl StdFunctionLayout {
    // the slot of `_Do_call` in the callable's vtable
    pub const DO_CALL_SLOT: u64 = 2;
}

const _: () = assert!(offset_of!(StdFunctionLayout, callable) == 0x38);
