// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

use clap::{Arg, App, AppSettings, SubCommand};

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
                        .about("copy an existing project to a new project")
                        .arg(Arg::with_name("EXISTING").required(true))
                        .arg(Arg::with_name("NEW").required(true))
                        .arg(Arg::with_name("no-edit")
                                 .long("no-edit")
                                 .required(false)))
        .subcommand(SubCommand::with_name("delete")
                        .about("delete existing projects")
                        .arg(Arg::with_name("PROJECT").multiple(true).required(true)))
        .subcommand(SubCommand::with_name("edit")
                        .alias("open")
                        .about("open an existing project in your editor")
                        .arg(Arg::with_name("PROJECT").required(true)))
        // TODO: decide if we want to add `implode?`
        .subcommand(SubCommand::with_name("list")
                        .about("list all projects"))
        .subcommand(SubCommand::with_name("local")
                        .about("run a project from a local TOML-file")
                        .arg(Arg::with_name("file")
                             .short("f")
                             .long("file")
                             .value_name("FILE")
                             .default_value("i3nator.toml")))
        .subcommand(SubCommand::with_name("new")
                        .about("create a new project and open it in your editor")
                        .arg(Arg::with_name("PROJECT").required(true))
                        .arg(Arg::with_name("no-edit")
                                 .long("no-edit")
                                 .required(false)))
        .subcommand(SubCommand::with_name("start")
                        .alias("run")
                        .about("start a project according to it's configuration"))
    // TODO: determine if we can implement `stop`.
    // This would probably require keeping track of PIDs and workspaces and such, so my
    // immediate thought is "no".
}
