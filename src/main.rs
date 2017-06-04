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

            NoLayoutExist {
                description("no layouts exist")
                display("no layouts exist. Feel free to create one")
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
use i3nator::configfile::ConfigFile;
use i3nator::layouts::{self, Layout};
use i3nator::projects::{self, Project};
use std::ascii::AsciiExt;
use std::convert::Into;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{BufReader, Read, stdin};
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

fn command_info(matches: &ArgMatches<'static>) -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of_os("PROJECT").unwrap();
    let project = Project::open(project_name)?;

    println!("Name: {}", project.name);
    println!("Configuration path: {}", project.path.to_string_lossy());
    println!("Configuration valid: {}",
             if project.verify().is_ok() {
                 "yes"
             } else {
                 "NO"
             });

    Ok(())
}

fn command_layout(matches: &ArgMatches<'static>) -> Result<()> {
    match matches.subcommand() {
        ("copy", Some(sub_matches)) => layout_copy(sub_matches),
        ("delete", Some(sub_matches)) => layout_delete(sub_matches),
        ("edit", Some(sub_matches)) => layout_edit(sub_matches),
        ("info", Some(sub_matches)) => layout_info(sub_matches),
        ("list", Some(sub_matches)) => layout_list(sub_matches),
        ("new", Some(sub_matches)) => layout_new(sub_matches),
        ("rename", Some(sub_matches)) => layout_rename(sub_matches),
        ("", None) =>
            // No subcommand given. The clap `AppSettings` should be set to output the help by
            // default, so this is unreachable.
            unreachable!(),
        _ =>
            // If all subcommands are defined above, this should be unreachable.
            unreachable!(),
    }
}

fn command_list(matches: &ArgMatches<'static>) -> Result<()> {
    let projects = projects::list();
    let quiet = matches.is_present("quiet");

    if projects.is_empty() {
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
        .unwrap_or_else(projects::list);

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

fn layout_copy(matches: &ArgMatches<'static>) -> Result<()> {
    // `EXISTING` and `NEW` should not be empty, clap ensures this.
    let existing_layout_name = matches.value_of_os("EXISTING").unwrap();
    let new_layout_name = matches.value_of_os("NEW").unwrap();

    let existing_layout = Layout::open(existing_layout_name)?;
    let new_layout = existing_layout.copy(new_layout_name)?;

    println!("Copied existing layout '{}' to new layout '{}'",
             existing_layout.name,
             new_layout.name);

    // Open config file for editing
    if !matches.is_present("no-edit") {
        open_editor(&new_layout)?;
    }

    Ok(())
}

fn layout_delete(matches: &ArgMatches<'static>) -> Result<()> {
    // `LAYOUT`s should not be empty, clap ensures this.
    let layouts = matches.values_of_os("LAYOUT").unwrap();

    for layout_name in layouts {
        Layout::open(layout_name)?.delete()?;
        println!("Deleted layout '{}'", layout_name.to_string_lossy());
    }

    Ok(())
}

fn layout_edit(matches: &ArgMatches<'static>) -> Result<()> {
    // `LAYOUT` should not be empty, clap ensures this.
    let layout_name = matches.value_of_os("LAYOUT").unwrap();
    let layout = Layout::open(layout_name)?;

    open_editor(&layout)?;

    Ok(())
}

fn layout_info(matches: &ArgMatches<'static>) -> Result<()> {
    // `LAYOUT` should not be empty, clap ensures this.
    let layout_name = matches.value_of_os("LAYOUT").unwrap();
    let layout = Layout::open(layout_name)?;

    println!("Name: {}", layout.name);
    println!("Configuration path: {}", layout.path.to_string_lossy());

    Ok(())
}

fn layout_list(matches: &ArgMatches<'static>) -> Result<()> {
    let layouts = layouts::list();
    let quiet = matches.is_present("quiet");

    if layouts.is_empty() {
        Err(ErrorKind::NoLayoutExist.into())
    } else {
        if !quiet {
            println!("i3nator layouts:");
        }
        for layout in layouts {
            if quiet {
                println!("{}", layout.to_string_lossy());
            } else {
                println!("  {}", layout.to_string_lossy());
            }
        }

        Ok(())
    }
}

fn layout_new(matches: &ArgMatches<'static>) -> Result<()> {
    // `LAYOUT` should not be empty, clap ensures this.
    let layout_name = matches.value_of_os("LAYOUT").unwrap();

    let layout = if !matches.is_present("template") {
        Layout::create(layout_name)?
    } else {
        let template = matches.value_of_os("template").unwrap();

        // Open appropriate reader
        let stdin_;
        let reader: Box<Read> = if template == "-" {
            stdin_ = stdin();
            Box::new(stdin_.lock())
        } else {
            Box::new(File::open(template)?)
        };
        let mut reader = BufReader::new(reader);

        // Load bytes from reader
        let mut bytes: Vec<u8> = Vec::new();
        reader.read_to_end(&mut bytes)?;

        // Create layout from template
        Layout::create_from_template(layout_name, &bytes)?
    };
    println!("Created layout '{}'", layout.name);

    // Open config file for editing
    if !matches.is_present("no-edit") {
        open_editor(&layout)?;
    }

    Ok(())
}

fn layout_rename(matches: &ArgMatches<'static>) -> Result<()> {
    // `CURRENT` and `NEW` should not be empty, clap ensures this.
    let current_layout_name = matches.value_of_os("CURRENT").unwrap();
    let new_layout_name = matches.value_of_os("NEW").unwrap();

    let current_layout = Layout::open(current_layout_name)?;
    println!("Renaming layout from '{}' to '{}'",
             current_layout_name.to_string_lossy(),
             new_layout_name.to_string_lossy());
    let new_layout = current_layout.rename(new_layout_name)?;

    // Open editor for new layout if desired
    if matches.is_present("edit") {
        open_editor(&new_layout)?;
    }

    Ok(())
}

fn get_editor() -> Result<OsString> {
    env::var_os("VISUAL")
        .or_else(|| env::var_os("EDITOR"))
        .and_then(|s| if !s.is_empty() { Some(s) } else { None })
        .ok_or_else(|| ErrorKind::EditorNotFound.into())
}

fn open_editor(configfile: &ConfigFile) -> Result<ExitStatus> {
    println!("Opening your editor to {}", configfile.name);
    Command::new(get_editor()?)
        .arg(configfile.path.as_os_str())
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
        ("info", Some(sub_matches)) => command_info(sub_matches),
        ("layout", Some(sub_matches)) => command_layout(sub_matches),
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
            // If all subcommands are defined above, this should be unreachable.
            unreachable!(),
    }
}

#[cfg(unix)]
quick_main!(run);
