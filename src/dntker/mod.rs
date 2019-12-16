mod util;
mod bc;

use std::io::Write;
use atty::Stream;

#[cfg(target_os = "windows")]
use winconsole::console as wconsole;

use ansi_term;

#[cfg(not(target_os = "freebsd"))]
use std::io::{stdout, BufWriter};

include!("dntker.rs");
include!("tests.rs");