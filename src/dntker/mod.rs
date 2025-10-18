mod bc;
mod util;

use atty::Stream;
use std::io::Write;

#[cfg(target_os = "windows")]
use winconsole::console as wconsole;

use ansi_term;
use std::io::{stdout, BufWriter};

include!("dntker.rs");
include!("tests.rs");
