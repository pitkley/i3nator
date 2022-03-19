// Copyright Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

#![deny(clippy::missing_docs_in_private_items)]

//! CLI module

use clap::{
    crate_authors, crate_description, crate_name, crate_version, CommandFactory, Parser, Subcommand,
};
use clap_complete::Shell;
use std::{ffi::OsString, io};

/// Main CLI entry type
#[derive(Parser)]
#[clap(
    author = crate_authors!(),
    version = crate_version!(),
    about = crate_description!(),
    infer_subcommands = true,
)]
pub(crate) struct Cli {
    /// Some docs on subcommands
    #[clap(subcommand)]
    pub(crate) command: Commands,
}

/// Main level subcommands
#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Manage projects
    #[clap(subcommand)]
    Project(ProjectCommands),
    /// Manage projects
    #[clap(flatten)]
    FlattenedProject(ProjectCommands),
    /// Manage layouts which can be used in projects
    #[clap(subcommand)]
    Layout(LayoutCommands),
    /// Generate shell completions for i3nator
    GenerateShellCompletions {
        /// Shell to generate the completions for
        #[clap(long = "shell", arg_enum)]
        generator: Shell,
        /// Path to save the completions into.
        ///
        /// If the directory in question does not exist, it will not be created. Don't specify this parameter if you
        /// want to output the completions to stdout.
        output_path: Option<OsString>,
    },
}

/// Project-specific subcommands
#[derive(Subcommand)]
pub(crate) enum ProjectCommands {
    /// Copy an existing project to a new project
    Copy {
        /// Name of the existing project
        existing: OsString,
        /// Name of the new, destination project
        new: OsString,
        /// Don't open new project for editing after copying
        #[clap(long = "no-edit")]
        no_edit: bool,
        /// Don't verify the contents of the new project after the editor closes
        #[clap(long = "no-verify")]
        no_verify: bool,
    },
    /// Delete existing projects
    Delete {
        /// Names of the projects to delete
        #[clap(required = true)]
        names: Vec<OsString>,
    },
    /// Open an existing project in your editor
    #[clap(alias = "open")]
    Edit {
        /// Name of the project to edit
        name: OsString,
        /// Don't verify the contents of the new project after the editor closes
        #[clap(long = "no-verify")]
        no_verify: bool,
    },
    /// Show information for the specified project
    Info {
        /// Name of the project to show informaiton for
        name: OsString,
    },
    /// List all projects
    List {
        /// List one project per line, no other output
        #[clap(short = 'q', long = "quiet")]
        quiet: bool,
    },
    /// Run a project from a local TOML-file
    Local {
        /// File to load the project from
        #[clap(short = 'f', long = "file", default_value = "i3nator.toml")]
        file: OsString,
        /// Directory used as context for starting the applications. This overrides any specified working-directory in
        /// the project's configuration.
        #[clap(short = 'd', long = "working-directory", value_name = "PATH")]
        working_directory: Option<OsString>,
        /// Workspace to apply the layout to. This overrides the specified workspace in the project's configuration.
        #[clap(short = 'w', long = "workspace", value_name = "WORKSPACE")]
        workspace: Option<String>,
    },
    /// Create a new project and open it in your editor
    New {
        /// Name of the project to create
        name: OsString,
        /// Don't open new project for editing after copying
        #[clap(long = "no-edit")]
        no_edit: bool,
        /// Don't verify the contents of the new project after the editor closes
        #[clap(long = "no-verify")]
        no_verify: bool,
    },
    /// Rename a project
    Rename {
        /// Name of the existing project to rename
        existing: OsString,
        /// New name for the existing project
        new: OsString,
        /// Open the renamed project for editing
        #[clap(long = "edit")]
        edit: bool,
        /// Don't verify the contents of the new project after the editor closes
        #[clap(long = "no-verify")]
        no_verify: bool,
    },
    /// Start a project according to it's configuration
    #[clap(alias = "run")]
    Start {
        /// Name of the project to start
        name: OsString,
        /// Directory used as context for starting the applications. This overrides any specified working-directory in
        /// the project's configuration.
        #[clap(short = 'd', long = "working-directory", value_name = "PATH")]
        working_directory: Option<OsString>,
        /// Workspace to apply the layout to. This overrides the specified workspace in the project's configuration.
        #[clap(short = 'w', long = "workspace", value_name = "WORKSPACE")]
        workspace: Option<String>,
    },
    /// Verify the configuration of the existing projects
    Verify {
        /// Names of the project to verify.
        ///
        /// If not specified, all projects will be checked.
        names: Vec<String>,
    },
}

/// Layout-specific subcommands
#[derive(Subcommand)]
pub(crate) enum LayoutCommands {
    /// Copy an existing layout to a new layout
    Copy {
        /// Name of the existing layout
        existing: OsString,
        /// Name of the new layout
        new: OsString,
        /// Don't open the new layout for editing after copying
        #[clap(long = "no-edit")]
        no_edit: bool,
    },
    /// Delete existing layouts
    #[clap(alias = "remove")]
    Delete {
        /// Names of the layouts to delete
        #[clap(required = true)]
        names: Vec<OsString>,
    },
    /// Open an existing layout in your editor
    Edit {
        /// Name of the layout to edit
        name: OsString,
    },
    /// Show information for the specified layout
    Info {
        /// Name of the layout to show information for
        name: OsString,
    },
    /// List all layouts
    List {
        /// List one layout per line, no other output
        #[clap(short = 'q', long = "quiet")]
        quiet: bool,
    },
    /// Create a new layout and open it in your editor
    New {
        /// Name of the layout to create
        name: OsString,
        /// Don't open the new layout for editing
        #[clap(long = "no-edit")]
        no_edit: bool,
        /// Prepopulate the layout from the given path. Use '-' to read from stdin.
        #[clap(short = 't', long = "template")]
        template: Option<OsString>,
    },
    /// Rename a layout
    Rename {
        /// Name of the existing layout to rename
        existing: OsString,
        /// New name for the existing layout
        new: OsString,
        /// Open the renamed layout for editing
        #[clap(long = "edit")]
        edit: bool,
    },
}

/// Generate shell completions
pub(crate) fn generate_completions<S: Into<OsString>>(
    generator: Shell,
    output_path: Option<S>,
) -> Result<(), io::Error> {
    let mut cmd = Cli::command();

    if let Some(output_path) = output_path {
        clap_complete::generate_to(generator, &mut cmd, crate_name!(), output_path)?;
    } else {
        clap_complete::generate(generator, &mut cmd, crate_name!(), &mut io::stdout());
    }

    Ok(())
}
