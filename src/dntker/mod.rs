mod util;
mod bc;

use std::io::Write;
use atty::Stream;
//use crate::DNTK_OPT;

#[cfg(target_os = "windows")]
use winconsole::console as wconsole;

use ansi_term;

include!("dntker.rs");
include!("tests.rs");