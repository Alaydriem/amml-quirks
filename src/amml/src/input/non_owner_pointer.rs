use std::ffi::c_void;

use amml_msvc::SharedPtr;

// `Bedrock::NonOwnerPointer<MinecraftInputHandler>` (24 bytes): the shared_ptr to the control block (whose first byte is
// the valid flag), then the pointer
#[repr(C)]
pub(crate) struct NonOwnerPointer {
    pub(crate) control: SharedPtr<u8>,
    pub(crate) pointer: *mut c_void,
}

// the game writes 24 bytes into this type
const _: () = assert!(core::mem::size_of::<NonOwnerPointer>() == 24);
