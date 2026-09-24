// This file is auto-generated and managed by the flowlang build script.
use flowlang::rustcmd::{Transform};

// Each flowlang library within this crate will be added as a module here.

mod cmdinit;
pub use cmdinit::cmdinit;
mod api;
pub static API : crate::api::api = crate::api::new();

use std::sync::Once;
use flowlang::hotswap::Initializer;
pub mod nebula;

static START: Once = Once::new();

#[no_mangle]
pub unsafe extern "C" fn mirror_nebula(initializer: *mut Initializer) {
    if initializer.is_null() { return; }

    // Mirror the host's ndata heaps exactly once per loaded generation of
    // this library, however many times the host calls in.
    START.call_once(|| {
        flowlang::mirror(("data", (*initializer).ndata_config));
    });

    // Register this library's commands on every call, so a reload picks
    // up new and changed ones.
    cmdinit(&mut (*initializer).cmds);
}

// ABI guard: the host compares this against its own contract before
// calling mirror, and refuses the load when the two flowlang copies
// disagree about the handshake.
#[no_mangle]
pub extern "C" fn nb_ffi_contract_nebula() -> *const std::os::raw::c_char {
    flowlang::hotswap::contract_ptr()
}
