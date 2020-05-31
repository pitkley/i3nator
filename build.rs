// Copyright Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

#[macro_use]
extern crate clap;

use clap::Shell;
use std::env;

include!("src/cli.rs");

fn main() {
    let outdir = match env::var_os("OUT_DIR") {
        None => return,
        Some(outdir) => outdir,
    };

    let mut app = cli();
    app.gen_completions(crate_name!(), Shell::Bash, outdir.clone());
    app.gen_completions(crate_name!(), Shell::Zsh, outdir.clone());
    app.gen_completions(crate_name!(), Shell::Fish, outdir);
}
