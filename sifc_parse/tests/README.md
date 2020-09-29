# sifc_parse tests

These tests can be run using `cargo test` from the sif root directory (or from anywhere inside `sifc_parse`).

The tests work by reading input files from the `input` directory. These are `.sif` files, and named
according to the functionality they test (ie. `for_stmt.sif` tests a for loop), but there is no official
naming convention as of now. Each file has a comment at the top detailing the expectations of the test:
whether the parser should pass or fail, and if it fails, where it is supposed to fail. The syntax of the
expectations string is:

```
# expect::pass | fail::line::pos
```

Expectation parsing is done in [util/mod.rs](https://github.com/cjkenn/sif/blob/master/tests/util/mod.rs#L75).

The [setup](https://github.com/cjkenn/sif/blob/master/tests/util/mod.rs#L16) method opens the input file at
the path provided, parses the expectations and returns a `ParseTestCtx` struct to set up the test. To test
a file, create a new `#[test]` method, call `util::setup(...)`, read the file, and then call `util::check`
to determine if the test passed. Check will see that the expectations matched, and if the test failed at the desired point in the file.

## Adding a new test

1. Create a new sif file in the `input` directory. Ensure the parse expectations are defined on the first line.
2. Add the test name to the paths map in the `get_test_path` function.
3. Create the test in `parse_test.rs`. Ensure that the string passed to `util::setup`
is the same key that was added to the paths map in step 2.
