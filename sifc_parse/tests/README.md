# sif_parse tests

These tests can be run using `cargo test` from the sif root directory.


## Adding a new test

1. Create a new sif file in the `input` directory. Ensure the parse expectations are defined on the first line.
2. Add the test name to the paths map in the `get_test_path` function.
3. Create the test in `parse_test.rs`. Ensure that the string passed to `util::setup`
is the same key that was added to the paths map in step 2.
