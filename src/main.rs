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
extern crate getch;
extern crate i3ipc;
extern crate i3nator;
#[macro_use]
extern crate lazy_static;
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
use getch::Getch;
use i3ipc::I3Connection;
use i3nator::projects;
use i3nator::projects::Project;
use std::ascii::AsciiExt;
use std::convert::Into;
use std::env;
use std::ffi::{OsStr, OsString};
use std::process::{Command, ExitStatus};

static PROJECT_TEMPLATE: &'static [u8] = include_bytes!("../resources/project_template.toml");

lazy_static! {
    static ref GETCH: Getch = Getch::new().expect("failed to create getch");
}

fn command_copy(matches: &ArgMatches<'static>) -> Result<()> {
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
        open_editor(&new_project)?;
        if !matches.is_present("no-verify") {
            verify_project(&new_project)?;
        }
    }

    Ok(())
}

fn command_delete(matches: &ArgMatches<'static>) -> Result<()> {
    // `PROJECT`s should not be empty, clap ensures this.
    let projects = matches.values_of_os("PROJECT").unwrap();

    for project_name in projects {
        Project::open(project_name)?.delete()?;
        println!("Deleted project '{}'", project_name.to_string_lossy());
    }

    Ok(())
}

fn command_edit(matches: &ArgMatches<'static>) -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of_os("PROJECT").unwrap();
    let project = Project::open(project_name)?;

    open_editor(&project)?;

    // Verify project contents
    if !matches.is_present("no-verify") {
        verify_project(&project)?;
    }

    Ok(())
}

fn command_list(matches: &ArgMatches<'static>) -> Result<()> {
    let projects = projects::list();
    let quiet = matches.is_present("quiet");

    if projects.is_empty() {
        // TODO: make error quiet as well?
        // We already exit with a non-zero exit code, so this might not be required in e.g. the
        // context of a shell script.
        Err(ErrorKind::NoProjectExist.into())
    } else {
        if !quiet {
            println!("i3nator projects:");
        }
        for project in projects {
            if quiet {
                println!("{}", project.to_string_lossy());
            } else {
                println!("  {}", project.to_string_lossy());
            }
        }

        Ok(())
    }
}

fn command_local(matches: &ArgMatches<'static>) -> Result<()> {
    // `FILE` should not be empty, clap ensures this.
    let project_path = matches.value_of_os("file").unwrap();
    let mut project = Project::from_path(project_path)?;
    let mut i3 = I3Connection::connect()?;

    println!("Starting project '{}'", project.name);
    project
        .start(&mut i3,
               matches.value_of_os("working-directory"),
               matches.value_of("workspace"))?;

    Ok(())
}

fn command_new(matches: &ArgMatches<'static>) -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of_os("PROJECT").unwrap();
    let project = Project::create_from_template(project_name, PROJECT_TEMPLATE)?;
    println!("Created project '{}'", project.name);

    // Open config file for editing
    if !matches.is_present("no-edit") {
        open_editor(&project)?;
        if !matches.is_present("no-verify") {
            verify_project(&project)?;
        }
    }

    Ok(())
}

fn command_rename(matches: &ArgMatches<'static>) -> Result<()> {
    // `CURRENT` and `NEW` should not be empty, clap ensures this.
    let current_project_name = matches.value_of_os("CURRENT").unwrap();
    let new_project_name = matches.value_of_os("NEW").unwrap();

    let current_project = Project::open(current_project_name)?;
    println!("Renaming project from '{}' to '{}'",
             current_project_name.to_string_lossy(),
             new_project_name.to_string_lossy());
    let new_project = current_project.rename(new_project_name)?;

    // Open editor for new project if desired
    if matches.is_present("edit") {
        open_editor(&new_project)?;
        if !matches.is_present("no-verify") {
            verify_project(&new_project)?;
        }
    }

    Ok(())
}

fn command_start(matches: &ArgMatches<'static>) -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of_os("PROJECT").unwrap();
    let mut project = Project::open(project_name)?;
    let mut i3 = I3Connection::connect()?;

    println!("Starting project '{}'", project.name);
    project
        .start(&mut i3,
               matches.value_of_os("working-directory"),
               matches.value_of("workspace"))?;

    Ok(())
}

fn command_verify(matches: &ArgMatches<'static>) -> Result<()> {
    // `PROJECT`s can be empty, if so, use the entire project list
    let projects: Vec<OsString> = matches
        .values_of_os("PROJECT")
        .map(|v| v.map(OsStr::to_os_string).collect::<Vec<_>>())
        .unwrap_or_else(|| projects::list());

    for project_name in projects {
        if let Err(e) = Project::open(&project_name)?.verify() {
            println!("Configuration INVALID: '{}'",
                     project_name.to_string_lossy());
            println!("Error:");
            println!("    {}", e);
            println!();
        } else {
            println!("Configuration   VALID: '{}'",
                     project_name.to_string_lossy());
        }
    }

    Ok(())
}

fn get_editor() -> Result<OsString> {
    env::var_os("VISUAL")
        .or_else(|| env::var_os("EDITOR"))
        .and_then(|s| if !s.is_empty() { Some(s) } else { None })
        .ok_or_else(|| ErrorKind::EditorNotFound.into())
}

fn open_editor(project: &Project) -> Result<ExitStatus> {
    println!("Opening your editor to edit project {}", project.name);
    Command::new(get_editor()?)
        .arg(&project.path)
        .status()
        .map_err(|e| e.into())
}

fn verify_project(project: &Project) -> Result<()> {
    while let Err(e) = project.verify() {
        println!();
        println!("PROJECT VERIFICATION FAILED!");
        println!("Error:");
        println!("  {}", e);
        println!();

        let mut ch: Option<char>;
        while {
                  println!("What do you want to do?");
                  println!("(R)eopen editor, (A)ccept anyway");

                  ch = GETCH
                      .getch()
                      .ok()
                      .map(|byte| byte.to_ascii_lowercase())
                      .map(|byte| byte as char);

                  if ch.is_none() {
                      true
                  } else {
                      match ch {
                          Some('a') | Some('r') => false,
                          _ => true,
                      }
                  }
              } {
            // Ugly do-while syntax:
            //   https://gist.github.com/huonw/8435502
        }

        match ch {
            Some('a') => break,
            Some('r') => open_editor(project)?,
            _ => continue,
        };
    }

    Ok(())
}

fn run() -> Result<()> {
    let matches = cli::cli().get_matches();

    match matches.subcommand() {
        ("copy", Some(sub_matches)) => command_copy(sub_matches),
        ("delete", Some(sub_matches)) => command_delete(sub_matches),
        ("edit", Some(sub_matches)) => command_edit(sub_matches),
        ("list", Some(sub_matches)) => command_list(sub_matches),
        ("local", Some(sub_matches)) => command_local(sub_matches),
        ("new", Some(sub_matches)) => command_new(sub_matches),
        ("rename", Some(sub_matches)) => command_rename(sub_matches),
        ("start", Some(sub_matches)) => command_start(sub_matches),
        ("verify", Some(sub_matches)) => command_verify(sub_matches),
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
