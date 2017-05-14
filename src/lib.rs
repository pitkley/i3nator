// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

#![warn(missing_docs)] // TODO: increase from `warn` to `deny`

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate shlex;

mod deserializers;
pub mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
