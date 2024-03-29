# Changelog

<!-- next-header -->

## Unreleased

* You can now interact with projects through the `i3nator project` subcommand.

    The commands available are identical to the "root"-commands, e.g. instead of `i3nator copy` you can now use `i3nator project copy`.
    Both styles of invocation are fully supported, you can decide which fits you better!

* Compatibility: the minimum supported Rust version is now 1.56.1, you will not be able to compile i3nator with older versions.

    (Please note that this does not affect how or where you can run the pre-built binary.)

<sub>Internal changes: dependency updates.</sub>

## 1.2.0 (2020-07-13)

* Compatibility: the minimum supported Rust version is now 1.38.0, you will not be able to compile i3nator with older versions.

    (Please note that this does not affect how or where you can run the pre-built binary.)

<sub>Internal changes: dependency updates, move CI to GitHub Actions.</sub>

## 1.1.0 (2017-06-08)

* Feature: Verify paths in configuration exist
* Feature: Added layout managing
* Fix: Expand tilde for layout-path

## 1.0.0 (2017-05-31)

This release is fully featured, everything that is mentioned in the README is implemented. Additionally, the command line interface is considered stable and will only have breaking changes with either a major or a minor version bump (still to be determined).
