# Rust Doc Checker

This is a standalone tool that accepts a path to a Rust source file to check. Returns `0` if the documentation is correct and not missing, otherwise a non-zero value if an error occurred or some docs don't exist or incorrect.

Example (input file):

```Rust
/// Some docs.
fn foo(my_value: usize) {
    // ...
}
```

Tool output:

```
(exit code: 1) expected to find documentation for the argument "my_value" of the function "foo"
```

Fixed example:

```Rust
/// Some docs.
/// 
/// # Arguments
/// 
/// * `my_value`: Some docs.
fn foo(my_value: usize) {
    // ...
}

// or just:

/// Some docs.
/// 
/// * `my_value`: Some docs.
fn bar(my_value: usize) {
    // ...
}
```

You can also document return values in various ways, for example:

```Rust
/// Does something.
/// 
/// # Arguments
/// 
/// * `arg1`: some docs.
/// 
/// # Return
/// 
/// Some value.
fn foo(arg1: usize) -> usize {}

// or just:

/// Does something, returns some value.
/// 
/// # Arguments
/// 
/// * `arg1`: some docs.
fn bar(arg1: usize) -> usize {}
```

As long as the documentation contains the `return` keyword, the return value is considered documented.

# Build

To build the tool you will need [Rust](https://www.rust-lang.org/tools/install).

Then in the root directory run:

```
cargo build --release
```

The compiled binary will be located at `/target/release/`.