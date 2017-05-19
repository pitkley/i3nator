// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

//! Errors, using [`error-chain`][error-chain].
//!
//! [error-chain]: https://crates.io/crates/error-chain

error_chain! {
    foreign_links {
        I3EstablishError(::i3ipc::EstablishError)
            #[doc = "Error caused by `i3ipc`, on establishing a connection."];

        I3MessageError(::i3ipc::MessageError)
            #[doc = "Error caused by `i3ipc`, on sending a message."];

        IoError(::std::io::Error)
            #[doc = "Error mapping to `std::io::Error`."];

        Utf8Error(::std::str::Utf8Error)
            #[doc = "Error mapping to `std::str::Utf8Error`."];

        TomlError(::toml::de::Error)
            #[doc = "Error caused by `toml`, on deserializing using Serde."];
    }

    errors {
        /// An error that can occur when splitting a string into a
        /// [`ApplicationCommand`][struct-ApplicationCommand].
        ///
        /// [struct-ApplicationCommand]: ../types/struct.ApplicationCommand.html
        CommandSplittingFailed(t: String) {
            description("command splitting failed")
            display("command splitting failed: '{}'", t)
        }

        /// An error that occurs when the default editor is not specified.
        ///
        /// One of the environment variables `$VISUAL` or `$EDITOR` has to be set.
        EditorNotFound {
            description("cannot find an editor")
            display("cannot find an editor. Please specify $VISUAL or $EDITOR")
        }

        /// An error that occurs when a `Path` (i.e. `OsStr`) cannot be converted to UTF8.
        InvalidUtF8Path(t: String) {
            description("path is invalid UTF8")
            display("path is invalid UTF8: '{}'", t)
        }

        /// An error that occurs if a specified path does not exist.
        PathDoesntExist(t: String) {
            description("path doesn't exist")
            display("path doesn't exist: '{}'", t)
        }

        /// An error that occurs if both [`layout`][structfield-General-layout] and
        /// [`layout_path`][structfield-General-layout_path] of [`General`][struct-General]
        /// have not been specified.
        ///
        /// [struct-General]: ../types/struct.General.html
        /// [structfield-General-layout]: ../types/struct.General.html#structfield.layout
        /// [structfield-General-layout_path]: ../types/struct.General.html#structfield.layout_path
        LayoutNotSpecified {
            description("layout and not specified")
            display("both `layout` and `layout_path` not specified")
        }

        /// An error that occurs if no project exists.
        NoProjectExist {
            description("no projects exist")
            display("no projects exist. Feel free to create one")
        }

        /// An error that occurs if a project under the same name already exists.
        ProjectExists(t: String) {
            description("project already exists")
            display("project already exists: '{}'", t)
        }

        /// An error that occurs if a project does not exist under a specified name.
        UnknownProject(t: String) {
            description("project is unknown")
            display("project is unknown: '{}'", t)
        }
    }
}
