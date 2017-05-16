// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

use clap::{Arg, App, AppSettings, SubCommand};

// usage:
//
// i3nator commands # list commands
//         copy <EXISTING> <NEW>
//         edit|e|open|o <PROJECT>
//         delete <PROJECT>
//         list
//         new <PROJECT>
//         start|s|run|r <PROJECT>
//         version

pub fn cli() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .settings(&[AppSettings::ColoredHelp,
                    AppSettings::GlobalVersion,
                    AppSettings::InferSubcommands,
                    AppSettings::SubcommandRequiredElseHelp,
                    AppSettings::VersionlessSubcommands,])
        .subcommand(SubCommand::with_name("copy")
                    .arg(Arg::with_name("EXISTING").required(true))
                    .arg(Arg::with_name("NEW").required(true)))
        .subcommand(SubCommand::with_name("delete")
                    .arg(Arg::with_name("PROJECT").multiple(true).required(true)))
        .subcommand(SubCommand::with_name("edit")
                        .alias("open")
                        .arg(Arg::with_name("PROJECT").required(true)))
        // TODO: decide if we want to add `implode?`
        .subcommand(SubCommand::with_name("list"))
        .subcommand(SubCommand::with_name("local")
                    .arg(Arg::with_name("file")
                         .short("f")
                         .long("file")
                         .value_name("FILE")
                         .default_value("tmuxinator.yml")))
        .subcommand(SubCommand::with_name("new")
                    .arg(Arg::with_name("PROJECT")
                         .required(true)))
        .subcommand(SubCommand::with_name("start")
                    .alias("run"))
    // TODO: determine if we can implement `stop`.
    // This would probably require keeping track of PIDs and workspaces and such, so my
    // immediate thought is "no".
}
