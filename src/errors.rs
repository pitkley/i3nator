// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

error_chain! {
    foreign_links {
        Utf8Error(::std::str::Utf8Error);
    }

    errors {
        CommandSplittingFailed(t: String) {
            description("command splitting failed")
            display("command splitting failed: '{}'", t)
        }
    }
}
