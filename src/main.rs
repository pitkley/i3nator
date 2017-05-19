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
extern crate i3ipc;
extern crate i3nator;
extern crate tempfile;

mod cli;
mod errors {
    error_chain! {
        foreign_links {
            I3EstablishError(::i3ipc::EstablishError);
            I3MessageError(::i3ipc::MessageError);
            IoError(::std::io::Error);
        }

        links {
            Lib(::i3nator::errors::Error, ::i3nator::errors::ErrorKind);
        }

        errors {
            EditorNotFound {
                description("cannot find an editor")
                display("cannot find an editor. Please specify $VISUAL or $EDITOR")
            }

            InvalidUtF8Path(t: String) {
                description("path is invalid UTF8")
                display("path is invalid UTF8: '{}'", t)
            }

            LayoutNotSpecified {
                description("layout and not specified")
                display("both `layout` and `layout_path` not specified")
            }

            NoProjectExist {
                description("no projects exist")
                display("no projects exist. Feel free to create one")
            }
        }
    }
}

use clap::ArgMatches;
use errors::*;
use i3ipc::I3Connection;
use i3nator::projects;
use i3nator::projects::Project;
use std::convert::Into;
use std::env;
use std::ffi::{OsStr, OsString};
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use tempfile::NamedTempFile;

static PROJECT_TEMPLATE: &'static [u8] = include_bytes!("../resources/project_template.toml");

fn command_copy(matches: &ArgMatches<'static>,
                _global_matches: &ArgMatches<'static>)
                -> Result<()> {
    // `EXISTING` and `NEW` should not be empty, clap ensures this.
    let existing_project_name = matches.value_of_os("EXISTING").unwrap();
    let new_project_name = matches.value_of_os("NEW").unwrap();

    let existing_project = Project::open(existing_project_name)?;
    let new_project = existing_project.copy(new_project_name)?;

    println!("Copied existing project '{}' to new project '{}'",
             existing_project.name,
             new_project.name);

    // Open config file for editing
    if !matches.is_present("no-edit") {
        println!("Opening your editor to edit project {}", new_project.name);
        open_editor(&new_project.path).map(|_| ())
    } else {
        Ok(())
    }
}

fn command_delete(matches: &ArgMatches<'static>,
                  _global_matches: &ArgMatches<'static>)
                  -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of_os("PROJECT").unwrap();

    let result = Project::open(project_name)?
        .delete()
        .map_err(|e| e.into());
    println!("Deleted project '{}'", project_name.to_string_lossy());
    result
}

fn command_edit(matches: &ArgMatches<'static>,
                _global_matches: &ArgMatches<'static>)
                -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of_os("PROJECT").unwrap();
    let project = Project::open(project_name)?;

    println!("Opening your editor to edit project {}", project.name);
    open_editor(&project.path).map(|_| ())
}

fn command_list(_matches: &ArgMatches<'static>,
                _global_matches: &ArgMatches<'static>)
                -> Result<()> {
    let projects = projects::list();

    if projects.is_empty() {
        Err(ErrorKind::NoProjectExist.into())
    } else {
        println!("i3nator projects:");
        for project in projects {
            println!("  {}", project);
        }

        Ok(())
    }
}

fn command_local(_matches: &ArgMatches<'static>,
                 _global_matches: &ArgMatches<'static>)
                 -> Result<()> {
    unimplemented!()
}

fn command_new(matches: &ArgMatches<'static>, _global_matches: &ArgMatches<'static>) -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of_os("PROJECT").unwrap();
    let project = Project::create_from_template(project_name, PROJECT_TEMPLATE)?;
    println!("Created project '{}'", project.name);

    // Open config file for editing
    if !matches.is_present("no-edit") {
        println!("Opening your editor to edit project {}", project.name);
        open_editor(&project.path).map(|_| ())
    } else {
        Ok(())
    }
}

fn command_start(matches: &ArgMatches<'static>,
                 global_matches: &ArgMatches<'static>)
                 -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of_os("PROJECT").unwrap();
    let mut project = Project::open(project_name)?;
    let config = project.config()?;

    // Establish connection to i3
    let mut i3 = I3Connection::connect()?;

    let general = &config.general;

    // Get working directory (`--working-directory` takes precedence if it exists)
    let working_directory = global_matches
        .value_of_os("working-directory")
        .map(OsStr::to_os_string)
        .or(general.working_directory.as_ref().map(OsString::from));

    // Create temporary file if required
    let mut tempfile = if general.layout.is_some() {
        Some(NamedTempFile::new()?)
    } else {
        None
    };

    // Get the provided layout-path or the path of the temporary file
    let path: &Path = if let Some(ref path) = general.layout_path {
        path
    } else if let Some(ref layout) = general.layout {
        // The layout has been provided directly, save into the temporary file.
        let mut tempfile = tempfile.as_mut().unwrap();
        tempfile.write_all(layout.as_bytes())?;
        tempfile.flush()?;
        tempfile.path()
    } else {
        // Neither `layout` nor `layout_path` has been specified
        bail!(ErrorKind::LayoutNotSpecified)
    };

    // Change workspace if provided
    if let Some(ref workspace) = general.workspace {
        i3.command(&format!("workspace {}", workspace))?;
    }

    // Append the layout to the workspace
    i3.command(&format!("append_layout {}",
                          path.to_str()
                              .ok_or_else(|| {
                                              ErrorKind::InvalidUtF8Path(path.to_string_lossy()
                                                                             .into_owned())
                                          })?))?;

    // Start the applications
    let applications = &config.applications;
    for application in applications {
        let mut cmd = Command::new(&application.command.program);
        // Set args if available
        if let Some(ref args) = application.command.args {
            cmd.args(args);
        }

        // Get working directory. Precedence is as follows:
        // 1. `--working-directory` command-line parameter
        // 2. `working_directory` option in config for application
        // 3. `working_directory` option in the general section of the config
        let working_directory = global_matches
            .value_of_os("working-directory")
            .map(OsStr::to_os_string)
            .or(application
                    .working_directory
                    .as_ref()
                    .map(OsString::from))
            .or(general.working_directory.as_ref().map(OsString::from));

        if let Some(working_directory) = working_directory {
            cmd.current_dir(working_directory);
        }

        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
    }

    Ok(())
}

fn get_editor() -> Result<OsString> {
    env::var_os("VISUAL")
        .or_else(|| env::var_os("EDITOR"))
        .ok_or_else(|| ErrorKind::EditorNotFound.into())
}

fn open_editor<S: AsRef<OsStr> + ?Sized>(path: &S) -> Result<ExitStatus> {
    Command::new(get_editor()?)
        .arg(path)
        .status()
        .map_err(|e| e.into())
}

fn run() -> Result<()> {
    let matches = cli::cli().get_matches();

    match matches.subcommand() {
        ("copy", Some(sub_matches)) => command_copy(sub_matches, &matches),
        ("delete", Some(sub_matches)) => command_delete(sub_matches, &matches),
        ("edit", Some(sub_matches)) => command_edit(sub_matches, &matches),
        ("list", Some(sub_matches)) => command_list(sub_matches, &matches),
        ("local", Some(sub_matches)) => command_local(sub_matches, &matches),
        ("new", Some(sub_matches)) => command_new(sub_matches, &matches),
        ("start", Some(sub_matches)) => command_start(sub_matches, &matches),
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
