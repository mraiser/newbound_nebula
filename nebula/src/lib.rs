// This file is auto-generated and managed by the flowlang build script.
use flowlang::rustcmd::{Transform};

// Each flowlang library within this crate will be added as a module here.

mod cmdinit;
pub use cmdinit::cmdinit;
mod api;
pub static API : crate::api::api = crate::api::new();

use std::sync::Once;
pub mod nebula;

// THIS IS THE FFI-SAFE INITIALIZER STRUCT.
// ITS DEFINITION MUST EXACTLY MATCH THE ONE IN THE MAIN BINARY.
#[repr(C)]
#[derive(Debug)]
pub struct Initializer {
    pub ndata_config: ndata::NDataConfig,
    pub cmds: Vec<(String, Transform, String)>,
}

static START: Once = Once::new();

#[no_mangle]
pub unsafe extern "C" fn mirror_nebula(initializer: *mut Initializer) {
    if initializer.is_null() { return; }

    // Use Once to ensure ndata::mirror is only ever called one time,
    // even across multiple hot-reloads of this library.
    START.call_once(|| {
        flowlang::mirror(("data", (*initializer).ndata_config));
    });

    // Then, call this library's internal cmdinit to populate the cmds vector.
    // We want this to run on every reload to register any new commands.
    cmdinit(&mut (*initializer).cmds);
}
