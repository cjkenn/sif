# sifc_tests

This crate contains integration tests for most parts of sif. Inside each mod, there is an `inputs` folder which holds the `.sif` files
that each test module will open and run. The following tests are included:

1. `parse_pass`: Inputs expected to pass the parsing phase.
2. `parse_fail`: Inputs expected to fail the parsing phase.
3. `exec_pass`: Inputs that pass execution, without any examination for correctness of outputs.
4. `exec_fail`: Inputs that should fail at runtime.
5. `compiler`: Verifies that the compiler generates correct bytecode for the vm.
6. `vm`: Verifies that after the vm executes, outputs are correct. These tests may examine registers and the heap for expected values. 
