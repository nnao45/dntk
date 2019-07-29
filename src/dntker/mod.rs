mod util;
mod bc;

use atty::Stream;

#[cfg(target_os = "windows")]
use winconsole::console as wconsole;

use ansi_term;
use std::io::{stdout, Write, BufWriter};

include!("dntker.rs");
include!("tests.rs");