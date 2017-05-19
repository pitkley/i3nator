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
use std::process::{Command, ExitStatus};

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

fn command_local(matches: &ArgMatches<'static>,
                 global_matches: &ArgMatches<'static>)
                 -> Result<()> {
    // `FILE` should not be empty, clap ensures this.
    let project_path = matches.value_of_os("file").unwrap();
    let mut project = Project::from_path(project_path)?;
    let mut i3 = I3Connection::connect()?;

    println!("Starting project '{}'", project.name);
    project
        .start(&mut i3, global_matches.value_of_os("working-directory"))?;

    Ok(())
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
    let mut i3 = I3Connection::connect()?;

    println!("Starting project '{}'", project.name);
    project
        .start(&mut i3, global_matches.value_of_os("working-directory"))?;

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
