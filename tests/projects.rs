// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

#![cfg(feature = "sequential-tests")]

extern crate i3nator;
#[macro_use]
extern crate lazy_static;
extern crate tempdir;
extern crate tempfile;

use i3nator::projects::{self, Project};
use i3nator::types::*;
use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::prelude::*;
use std::panic::{self, UnwindSafe};
use std::path::{Path, PathBuf};
use tempdir::TempDir;
use tempfile::NamedTempFile;

lazy_static! {
    static ref TMP_DIR: TempDir = TempDir::new("i3nator-tests").unwrap();
    static ref PROJECTS_DIR: PathBuf = TMP_DIR.path().join("i3nator/projects");
}

fn with_projects_dir<F: FnOnce(&Path) -> ()>(body: F)
    where F: UnwindSafe
{
    // Create the temporary directories if they do not exist
    if !PROJECTS_DIR.exists() {
        fs::create_dir_all(&*PROJECTS_DIR).expect("couldn't create temporary directories");
    }

    // Set up temporary XDG config directory
    env::set_var("XDG_CONFIG_HOME", TMP_DIR.path());

    // Run body
    let panic_result = panic::catch_unwind(|| body(PROJECTS_DIR.as_ref()));

    // Remove the temporary directories
    fs::remove_dir_all(&*TMP_DIR).expect("couldn't delete temporary directories");

    if let Err(err) = panic_result {
        panic::resume_unwind(err);
    }
}

#[test]
fn empty_list() {
    with_projects_dir(|_| {
                          assert!(projects::list().is_empty());
                      })
}

#[test]
fn create() {
    with_projects_dir(|projects_dir| {
        let project = Project::create("project-one").unwrap();
        assert_eq!(project.name, "project-one");
        assert_eq!(project.path, projects_dir.join("project-one.toml"));
        assert!(project.verify().is_err());

        // File does not get created by default, list should still be empty
        assert!(projects::list().is_empty());
    })
}

#[test]
#[should_panic(expected = "ProjectExists")]
fn create_exists() {
    with_projects_dir(|projects_dir| {
        let project = Project::create("project-one").unwrap();
        assert_eq!(project.name, "project-one");
        assert_eq!(project.path, projects_dir.join("project-one.toml"));
        assert!(project.verify().is_err());

        // Create project file
        File::create(&project.path).expect("couldn't create project file");

        // File created, list should contain it
        assert_eq!(projects::list(), vec![OsString::from("project-one")]);

        // Create project with same name, this should fail
        Project::create("project-one").unwrap();
    })
}

#[test]
fn create_from_template() {
    with_projects_dir(|projects_dir| {
        let template = "this is my template";
        let project = Project::create_from_template("project-template", template.as_bytes())
            .unwrap();

        assert_eq!(project.name, "project-template");
        assert_eq!(project.path, projects_dir.join("project-template.toml"));
        assert!(project.path.exists());
        assert!(project.verify().is_err());

        let mut file = File::open(project.path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        assert_eq!(contents, template);
    })
}

#[test]
fn from_path() {
    let tempfile = NamedTempFile::new().expect("couldn't create temporary file");
    let project = Project::from_path(tempfile.path()).unwrap();
    assert_eq!(project.name, "local");
    assert_eq!(project.path, tempfile.path());
    assert!(project.verify().is_err());
}

#[test]
#[should_panic(expected = "PathDoesntExist")]
fn from_path_not_exists() {
    Project::from_path("/this/path/does/not/exist").unwrap();
}

#[test]
fn open() {
    with_projects_dir(|projects_dir| {
        let project = Project::create("project-open").unwrap();
        assert_eq!(project.name, "project-open");
        assert_eq!(project.path, projects_dir.join("project-open.toml"));
        assert!(project.verify().is_err());

        // Create project file
        File::create(&project.path).expect("couldn't create project file");

        // Open project
        let project_open = Project::open("project-open").unwrap();
        assert_eq!(project_open, project);
    })
}

#[test]
#[should_panic(expected = "UnknownProject")]
fn open_unknown_project() {
    with_projects_dir(|_| { Project::open("unknown-project").unwrap(); })
}

#[test]
fn config() {
    with_projects_dir(|projects_dir| {
        let template = r#"[general]
                          layout = "/some/layout/path"

                          [[applications]]
                          command = "mycommand""#;
        let mut project = Project::create_from_template("project-template", template.as_bytes())
            .unwrap();

        assert_eq!(project.name, "project-template");
        assert_eq!(project.path, projects_dir.join("project-template.toml"));
        assert!(project.path.exists());
        assert!(project.verify().is_ok());

        let expected = Config {
            general: General {
                working_directory: None,
                workspace: None,
                layout: Layout::Path("/some/layout/path".into()),
            },
            applications: vec![Application {
                                   command: ApplicationCommand {
                                       program: "mycommand".to_owned(),
                                       args: vec![],
                                   },
                                   working_directory: None,
                                   exec: None,
                               }],
        };

        assert_eq!(project.config().unwrap(), &expected);
    })
}

#[test]
fn config_invalid() {
    with_projects_dir(|projects_dir| {
        let template = r#"invalid template"#;
        let mut project = Project::create_from_template("project-template", template.as_bytes())
            .unwrap();

        assert_eq!(project.name, "project-template");
        assert_eq!(project.path, projects_dir.join("project-template.toml"));
        assert!(project.path.exists());
        assert!(project.verify().is_err());
        assert!(project.config().is_err());
    })
}

#[test]
fn copy() {
    with_projects_dir(|projects_dir| {
        let project = Project::create("project-existing").unwrap();
        assert_eq!(project.name, "project-existing");
        assert_eq!(project.path, projects_dir.join("project-existing.toml"));
        assert!(project.verify().is_err());

        // Create project file
        File::create(&project.path).expect("couldn't create project file");

        let project_new = project.copy("project-new").unwrap();
        assert_eq!(project_new.name, "project-new");
        assert_eq!(project_new.path, projects_dir.join("project-new.toml"));
        assert!(project.verify().is_err());
    })
}

#[test]
#[should_panic(expected = "the source path is not an existing regular file")]
fn copy_without_file() {
    with_projects_dir(|projects_dir| {
                          let project = Project::create("project-existing").unwrap();
                          assert_eq!(project.name, "project-existing");
                          assert_eq!(project.path, projects_dir.join("project-existing.toml"));
                          assert!(project.verify().is_err());

                          project.copy("project-new").unwrap();
                      })
}

#[test]
fn delete() {
    with_projects_dir(|projects_dir| {
        let project = Project::create("project-delete").unwrap();
        assert_eq!(project.name, "project-delete");
        assert_eq!(project.path, projects_dir.join("project-delete.toml"));
        assert!(project.verify().is_err());

        // Create project file
        File::create(&project.path).expect("couldn't create project file");

        assert!(project.delete().is_ok());
        assert!(!project.path.exists())
    })
}

#[test]
#[should_panic(expected = "No such file or directory")]
fn delete_without_file() {
    with_projects_dir(|projects_dir| {
                          let project = Project::create("project-delete").unwrap();
                          assert_eq!(project.name, "project-delete");
                          assert_eq!(project.path, projects_dir.join("project-delete.toml"));
                          assert!(project.verify().is_err());

                          project.delete().unwrap();
                      })
}

#[test]
fn rename() {
    with_projects_dir(|projects_dir| {
        let project = Project::create("project-rename-old").unwrap();
        assert_eq!(project.name, "project-rename-old");
        assert_eq!(project.path, projects_dir.join("project-rename-old.toml"));
        assert!(project.verify().is_err());

        // Create project file
        File::create(&project.path).expect("couldn't create project file");

        let project_new = project.rename("project-rename-new").unwrap();
        assert_eq!(project_new.name, "project-rename-new");
        assert_eq!(project_new.path,
                   projects_dir.join("project-rename-new.toml"));
        assert!(project_new.verify().is_err());

        assert!(!project.path.exists());
        assert!(project_new.path.exists());
    })
}
