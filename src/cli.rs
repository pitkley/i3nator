// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

use clap::{Arg, App, AppSettings, SubCommand};

pub fn cli() -> App<'static, 'static> {
    let working_directory = Arg::with_name("working-directory")
        .help("Directory used as context for starting the applications")
        .long_help("Directory used as context for starting the applications. This overrides any \
                    specified working-directory in the projects configuration.")
        .short("d")
        .long("working-directory")
        .takes_value(true)
        .value_name("PATH")
        .required(false);
    let workspace = Arg::with_name("workspace")
        .help("Workspace to apply the layout to")
        .long_help("Workspace to apply the layout to. This overrides the specified workspace in \
                    the projects configuration.")
        .short("w")
        .long("workspace")
        .takes_value(true)
        .value_name("WORKSPACE")
        .required(false);

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
                                 .required(false))
                        .arg(Arg::with_name("no-verify")
                                 .help("Don't verify the contents of the new project after the \
                                        editor closes")
                                 .long("no-verify")
                                 .required(false)
                                 .conflicts_with("no-edit")))
        .subcommand(SubCommand::with_name("delete")
                        .about("delete existing projects")
                        .arg(Arg::with_name("PROJECT").multiple(true).required(true)))
        .subcommand(SubCommand::with_name("edit")
                        .alias("open")
                        .about("open an existing project in your editor")
                        .arg(Arg::with_name("PROJECT").required(true))
                        .arg(Arg::with_name("no-verify")
                                 .help("Don't verify the contents of the new project after the \
                                        editor closes")
                                 .long("no-verify")
                                 .required(false)))
        // TODO: decide if we want to add `implode?`
        .subcommand(SubCommand::with_name("list")
                        .about("list all projects")
                        .arg(Arg::with_name("quiet")
                                .short("q")
                                .long("quiet")
                                .takes_value(false)
                                .required(false)))
        .subcommand(SubCommand::with_name("local")
                        .about("run a project from a local TOML-file")
                        .arg(Arg::with_name("file")
                             .short("f")
                             .long("file")
                             .value_name("FILE")
                             .default_value("i3nator.toml"))
                        .arg(working_directory.clone())
                        .arg(workspace.clone()))
        .subcommand(SubCommand::with_name("new")
                        .about("create a new project and open it in your editor")
                        .arg(Arg::with_name("PROJECT").required(true))
                        .arg(Arg::with_name("no-edit")
                                 .long("no-edit")
                                 .required(false))
                        .arg(Arg::with_name("no-verify")
                                 .help("Don't verify the contents of the new project after the \
                                        editor closes")
                                 .long("no-verify")
                                 .required(false)
                                 .conflicts_with("no-edit")))
        .subcommand(SubCommand::with_name("rename")
                        .about("rename a project")
                        .arg(Arg::with_name("CURRENT").required(true))
                        .arg(Arg::with_name("NEW").required(true))
                        .arg(Arg::with_name("edit")
                                 .long("edit")
                                 .required(false))
                        .arg(Arg::with_name("no-verify")
                                 .help("Don't verify the contents of the new project after the \
                                        editor closes")
                                 .long("no-verify")
                                 .required(false)
                                 .requires("edit")))
        .subcommand(SubCommand::with_name("start")
                        .alias("run")
                        .about("start a project according to it's configuration")
                        .arg(Arg::with_name("PROJECT").required(true))
                        .arg(working_directory)
                        .arg(workspace))
    // TODO: determine if we can implement `stop`.
    // This would probably require keeping track of PIDs and workspaces and such, so my
    // immediate thought is "no".
}
