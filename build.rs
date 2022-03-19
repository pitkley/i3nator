// Copyright Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

use clap::{crate_name, CommandFactory};
use clap_complete::{generate_to, shells};
use std::{env, io::Error};

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = Cli::command();
    generate_to(shells::Bash, &mut cmd, crate_name!(), outdir.clone())?;
    generate_to(shells::Zsh, &mut cmd, crate_name!(), outdir.clone())?;
    generate_to(shells::Fish, &mut cmd, crate_name!(), outdir.clone())?;

    Ok(())
}
