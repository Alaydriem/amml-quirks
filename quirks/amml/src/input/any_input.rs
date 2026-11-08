use std::ffi::c_void;

use amml_game::{EntryId, GameEntries, RawObject};

use crate::input::{InputHandlerLayout, MinecraftInputHandlerLayout, NonOwnerPointer, StdFunctionLayout};

// The game's any-input handler: `client->getMinecraftInput()->mInputHandler->mAnyInputHandler(*client)`, the code the game
// runs for each input event. No key, button or pointer event is made.
#[derive(Debug, Clone, Copy)]
pub struct AnyInput {
    get_minecraft_input: u64,
}

// the hidden return pointer after `this`; the buffer comes back in rax
type GetMinecraftInputFn = unsafe extern "C" fn(*mut c_void, *mut NonOwnerPointer) -> *mut NonOwnerPointer;

// `std::function<void(ClientInstance&)>::_Do_call`: the callable, then the client
type DoCallFn = unsafe extern "C" fn(*mut c_void, *mut c_void);

impl AnyInput {
    pub const GET_MINECRAFT_INPUT: EntryId = EntryId::CLIENT_INSTANCE_GET_MINECRAFT_INPUT;

    // `None` when the entry is off
    pub fn new(entries: &GameEntries) -> Option<Self> {
        Some(Self { get_minecraft_input: entries.get(Self::GET_MINECRAFT_INPUT)? })
    }

    /// Runs the any-input handler of `client`. False when the client has no input handler, the handler has no
    /// `InputHandler`, or no handler is registered yet; nothing is called then. The `NonOwnerPointer` the game hands out is
    /// released once. Game thread.
    ///
    /// # Safety
    /// `client` must be the live `ClientInstance`
    #[allow(unsafe_code)]
    pub unsafe fn fire(&self, client: *mut c_void) -> bool {
        // SAFETY: the caller guarantees a live client whose vtable holds getMinecraftInput at the slot
        let Some(function) = (unsafe { RawObject::slot(client, self.get_minecraft_input) }) else {
            return false;
        };
        // SAFETY: the slot holds `NonOwnerPointer<MinecraftInputHandler> getMinecraftInput() const`, a member that
        // returns a class through a hidden pointer
        let call: GetMinecraftInputFn = unsafe { std::mem::transmute(function) };
        let mut out = std::mem::MaybeUninit::<NonOwnerPointer>::zeroed();
        // SAFETY: `out` is a buffer for the 24-byte NonOwnerPointer
        unsafe { call(client, out.as_mut_ptr()) };
        // SAFETY: the game constructed the pointer in `out` (or left it zeroed, empty)
        let mut input = unsafe { out.assume_init() };
        let valid = input.control.get().is_some_and(|flag| *flag == 1);
        let fired = valid && !input.pointer.is_null() && {
            // SAFETY: a valid NonOwnerPointer points to the live MinecraftInputHandler
            unsafe { Self::run(input.pointer, client) }
        };
        // SAFETY: the copy the game handed out holds one use of the control block, given back here
        unsafe { input.control.release() };
        fired
    }

    // walks mInputHandler, mAnyInputHandler and the callable, and calls it
    #[allow(unsafe_code)]
    unsafe fn run(handler: *mut c_void, client: *mut c_void) -> bool {
        let handler = handler.cast::<MinecraftInputHandlerLayout>();
        // SAFETY: a live MinecraftInputHandler holds its InputHandler pointer at the mirror offset
        let input = unsafe { (&raw const (*handler).input_handler).read_unaligned() };
        if input.is_null() {
            return false;
        }
        let input = input.cast::<InputHandlerLayout>();
        // SAFETY: a live InputHandler holds the std::function at the mirror offset
        let callable = unsafe { (&raw const (*input).any_input.callable).read_unaligned() };
        if callable.is_null() {
            return false;
        }
        // SAFETY: a non-empty std::function points to a callable object whose vtable holds `_Do_call` at the slot
        let Some(do_call) = (unsafe { RawObject::slot(callable, StdFunctionLayout::DO_CALL_SLOT) }) else {
            return false;
        };
        // SAFETY: the slot holds `_Do_call` of a `void(ClientInstance&)` callable
        let do_call: DoCallFn = unsafe { std::mem::transmute(do_call) };
        // SAFETY: the callable and the client are live; the game calls it the same way in `InputHandler::tick`
        unsafe { do_call(callable, client) };
        true
    }
}
