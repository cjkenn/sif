# sifc_parse tests

These tests can be run using `cargo test` from the sif root directory (or from anywhere inside `sifc_parse`).

The tests work by reading input files from the `input` directory. These are `.sif` files, and named
according to the functionality they test (ie. `for_stmt.sif` tests a for loop), but there is no official
naming convention as of now. Each file has a comment at the top detailing the expectations of the test:
whether the parser should pass or fail. The syntax of the expectations string is:

```
# expect::pass | fail
```

The `setiup` method opens the input file at the path provided, parses the expectations and returns
a `ParseTestCtx` struct to set up the test.

## Adding a new test

1. Create a new sif file in the `input` directory. Ensure the parse expectations are defined on the first line.
2. Use the `test_parse` macro, passing in your test name and then the filename as a `&str`. The macro will
create the test.
