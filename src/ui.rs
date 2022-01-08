// Ignore the warnings this will inevitably create :E
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]

// Import from the file generated during the build stage in `../build.rs`
include!(concat!(env!("OUT_DIR"), "/nbt-edit-ui.rs"));
