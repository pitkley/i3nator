// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

use clap::{App, AppSettings, Arg, SubCommand};

pub fn cli() -> App<'static, 'static> {
    let working_directory = Arg::with_name("working-directory")
        .help("Directory used as context for starting the applications")
        .long_help(
            "Directory used as context for starting the applications. This overrides any \
             specified working-directory in the projects configuration.",
        )
        .short("d")
        .long("working-directory")
        .takes_value(true)
        .value_name("PATH")
        .required(false);
    let workspace = Arg::with_name("workspace")
        .help("Workspace to apply the layout to")
        .long_help(
            "Workspace to apply the layout to. This overrides the specified workspace in the \
             projects configuration.",
        )
        .short("w")
        .long("workspace")
        .takes_value(true)
        .value_name("WORKSPACE")
        .required(false);

    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .global_settings(&[
            AppSettings::ColoredHelp,
            AppSettings::GlobalVersion,
            AppSettings::InferSubcommands,
            AppSettings::VersionlessSubcommands,
        ])
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("copy")
                .about("Copy an existing project to a new project")
                .arg(
                    Arg::with_name("EXISTING")
                        .help("Name of the existing project")
                        .required(true),
                )
                .arg(
                    Arg::with_name("NEW")
                        .help("Name of the new, destination project")
                        .required(true),
                )
                .arg(
                    Arg::with_name("no-edit")
                        .help("Don't open the new project for editing after copying")
                        .long("no-edit")
                        .required(false),
                )
                .arg(
                    Arg::with_name("no-verify")
                        .help(
                            "Don't verify the contents of the new project after the editor closes",
                        )
                        .long("no-verify")
                        .required(false)
                        .conflicts_with("no-edit"),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .alias("remove")
                .about("Delete existing projects")
                .arg(
                    Arg::with_name("NAME")
                        .help("Names of the projects to delete")
                        .multiple(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("edit")
                .alias("open")
                .about("Open an existing project in your editor")
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the project to edit")
                        .required(true),
                )
                .arg(
                    Arg::with_name("no-verify")
                        .help(
                            "Don't verify the contents of the new project after the editor closes",
                        )
                        .long("no-verify")
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("info")
                .about("Show information for the specified project")
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the project to show information for")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("List all projects")
                .arg(
                    Arg::with_name("quiet")
                        .help("List one project per line, no other output")
                        .short("q")
                        .long("quiet")
                        .takes_value(false)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("local")
                .about("Run a project from a local TOML-file")
                .arg(
                    Arg::with_name("file")
                        .help("File to load the project from")
                        .short("f")
                        .long("file")
                        .value_name("FILE")
                        .default_value("i3nator.toml"),
                )
                .arg(working_directory.clone())
                .arg(workspace.clone()),
        )
        .subcommand(
            SubCommand::with_name("new")
                .about("Create a new project and open it in your editor")
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the project to create")
                        .required(true),
                )
                .arg(
                    Arg::with_name("no-edit")
                        .help("Don't open the new project for editing")
                        .long("no-edit")
                        .required(false),
                )
                .arg(
                    Arg::with_name("no-verify")
                        .help(
                            "Don't verify the contents of the new project after the editor closes",
                        )
                        .long("no-verify")
                        .required(false)
                        .conflicts_with("no-edit"),
                ),
        )
        .subcommand(
            SubCommand::with_name("rename")
                .about("Rename a project")
                .arg(
                    Arg::with_name("CURRENT")
                        .help("Name of the existing project to rename")
                        .required(true),
                )
                .arg(
                    Arg::with_name("NEW")
                        .help("New name for the existing project")
                        .required(true),
                )
                .arg(
                    Arg::with_name("edit")
                        .help("Open the renamed project for editing")
                        .long("edit")
                        .required(false),
                )
                .arg(
                    Arg::with_name("no-verify")
                        .help(
                            "Don't verify the contents of the new project after the editor closes",
                        )
                        .long("no-verify")
                        .required(false)
                        .requires("edit"),
                ),
        )
        .subcommand(
            SubCommand::with_name("start")
                .alias("run")
                .about("Start a project according to it's configuration")
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the project to start")
                        .required(true),
                )
                .arg(working_directory)
                .arg(workspace),
        )
        .subcommand(
            SubCommand::with_name("verify")
                .about("Verify the configuration of the existing projects")
                .arg(
                    Arg::with_name("NAME")
                        .help("Names of the projects to verify")
                        .long_help(
                            "Name of the projects to verify. If not specified, all projects will \
                             be checked.",
                        )
                        .multiple(true)
                        .required(false),
                ),
        )
        .subcommand(cli_layout())
}

pub fn cli_layout() -> App<'static, 'static> {
    SubCommand::with_name("layout")
        .about("Manage layouts which can used in projects")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("copy")
                .about("Copy an existing layout to a new layout")
                .arg(
                    Arg::with_name("EXISTING")
                        .help("Name of the existing layout")
                        .required(true),
                )
                .arg(
                    Arg::with_name("NEW")
                        .help("Name of the new, destination layout")
                        .required(true),
                )
                .arg(
                    Arg::with_name("no-edit")
                        .help("Don't open the new layout for editing after copying")
                        .long("no-edit")
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .alias("remove")
                .about("Delete existing layouts")
                .arg(
                    Arg::with_name("NAME")
                        .help("Names of the layouts to delete")
                        .multiple(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("edit")
                .alias("open")
                .about("Open an existing layout in your editor")
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the layout to edit")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("info")
                .about("Show information for the specified layout")
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the layout to show information for")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("list").about("List all layouts").arg(
                Arg::with_name("quiet")
                    .help("List one layout per line, no other output")
                    .short("q")
                    .long("quiet")
                    .takes_value(false)
                    .required(false),
            ),
        )
        .subcommand(
            SubCommand::with_name("new")
                .about("Create a new layout and open it in your editor")
                .arg(
                    Arg::with_name("NAME")
                        .help("Name of the layout to create")
                        .required(true),
                )
                .arg(
                    Arg::with_name("no-edit")
                        .help("Don't open the new layout for editing")
                        .long("no-edit")
                        .required(false),
                )
                .arg(
                    Arg::with_name("template")
                        .help(
                            "Prepopulate the layout from the given path. Use '-' to read from \
                             stdin.",
                        )
                        .short("t")
                        .long("template")
                        .takes_value(true)
                        .value_name("TEMPLATE")
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("rename")
                .about("Rename a layout")
                .arg(
                    Arg::with_name("CURRENT")
                        .help("Name of the existing layout to rename")
                        .required(true),
                )
                .arg(
                    Arg::with_name("NEW")
                        .help("New name for the existing layout")
                        .required(true),
                )
                .arg(
                    Arg::with_name("edit")
                        .help("Open the renamed layout for editing")
                        .long("edit")
                        .required(false),
                ),
        )
}
