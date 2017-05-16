// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;

mod cli;
mod errors {
    error_chain!{}
}

use clap::ArgMatches;
use errors::*;

fn command_copy(_matches: &ArgMatches<'static>) -> Result<()> {
    unimplemented!()
}

fn command_delete(_matches: &ArgMatches<'static>) -> Result<()> {
    unimplemented!()
}

fn command_edit(_matches: &ArgMatches<'static>) -> Result<()> {
    unimplemented!()
}

fn command_list(_matches: &ArgMatches<'static>) -> Result<()> {
    unimplemented!()
}

fn command_local(_matches: &ArgMatches<'static>) -> Result<()> {
    unimplemented!()
}

fn command_new(_matches: &ArgMatches<'static>) -> Result<()> {
    unimplemented!()
}

fn command_start(_matches: &ArgMatches<'static>) -> Result<()> {
    unimplemented!()
}

fn run() -> Result<()> {
    let matches = cli::cli().get_matches();

    match matches.subcommand() {
        ("copy", Some(matches)) => command_copy(matches),
        ("delete", Some(matches)) => command_delete(matches),
        ("edit", Some(matches)) => command_edit(matches),
        ("list", Some(matches)) => command_list(matches),
        ("local", Some(matches)) => command_local(matches),
        ("new", Some(matches)) => command_new(matches),
        ("start", Some(matches)) => command_start(matches),
        ("", None) =>
            // No subcommand given. The clap `AppSettings` should be set to output the help by
            // default, so this is unreachable.
            unreachable!(),
        _ =>
            // All subcommands are defined above, this is unreachable.
            unreachable!(),
    }
}

quick_main!(run);
