#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op, clippy::cargo_common_metadata)]
mod buffer;
mod error;
mod panic;
mod read;
mod v1;
mod wave;
mod write;

use anyhow::{bail, Result};
use std::ffi::CStr;
use std::os::raw::c_char;

fn filename_to_string(ptr: *const c_char) -> Result<String> {
    if ptr.is_null() {
        bail!("filename is null");
    }
    let cstr = unsafe { CStr::from_ptr(ptr) };
    Ok(cstr.to_str()?.to_string())
}
