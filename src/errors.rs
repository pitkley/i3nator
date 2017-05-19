// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

error_chain! {
    foreign_links {
        I3EstablishError(::i3ipc::EstablishError);
        I3MessageError(::i3ipc::MessageError);
        IoError(::std::io::Error);
        Utf8Error(::std::str::Utf8Error);
        TomlError(::toml::de::Error);
    }

    errors {
        CommandSplittingFailed(t: String) {
            description("command splitting failed")
            display("command splitting failed: '{}'", t)
        }

        EditorNotFound {
            description("cannot find an editor")
            display("cannot find an editor. Please specify $VISUAL or $EDITOR")
        }

        InvalidUtF8Path(t: String) {
            description("path is invalid UTF8")
            display("path is invalid UTF8: '{}'", t)
        }

        PathDoesntExist(t: String) {
            description("path doesn't exist")
            display("path doesn't exist: '{}'", t)
        }

        LayoutNotSpecified {
            description("layout and not specified")
            display("both `layout` and `layout_path` not specified")
        }

        NoProjectExist {
            description("no projects exist")
            display("no projects exist. Feel free to create one")
        }

        ProjectExists(t: String) {
            description("project already exists")
            display("project already exists: '{}'", t)
        }

        UnknownProject(t: String) {
            description("project is unknown")
            display("project is unknown: '{}'", t)
        }
    }
}
