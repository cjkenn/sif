# sif

![sif-build](https://github.com/cjkenn/sif/workflows/sif-build/badge.svg?branch=master)

sif is a scripting language with c-style syntax. It contains a bytecode compiler, optimizer and a register based vm. It's small and easily embeddable into rust programs using macros. There is also a nano stdlib for basic operations and interacting with arrays and tables.

sif doesn't really contain any novel features at the moment, and sort of serves as an educational compiler for me to implement what I choose to freely. 

Documentation can be found on the [wiki](https://github.com/cjkenn/sif/wiki).

## Quick Install
```sh
git clone https://github.com/cjkenn/sif.git
cd sif
cargo run <input-file>
```

## Samples

### Hello world
```
@print("Hello world")
```

### Fizzbuzz
```
fn fizzbuzz(input) {
  if input % 15 == 0 {
    @print("fizzbuzz");
  } elif input % 3 == 0 {
    @print("fizz");
  } elif input % 5 == 0 {
    @print("buzz");
  } else {
    @print(input);
  }
}

for i, v in @range(1, 15) {
    fizzbuzz(v);
}
```

## Usage
```sh
USAGE:
    sif [FLAGS] [OPTIONS] <filename>

ARGS:
    <filename>    sif file to parse and run

FLAGS:
    -a, --analysis      Performs analysis on the CFG and IR before starting the vm
        --bco           Runs the bytecode optimizer before executing in vm
        --emit-ast      Prints the syntax tree to stdout
        --emit-ir       Prints sif bytecode to stdout
    -h, --help          Prints help information
        --timings       Display basic durations for phases of sif
    -t, --trace-exec    Traces VM execution by printing running instructions to stdout
    -V, --version       Prints version information

OPTIONS:
    -H, --heap-size <heap-size>    Sets initial heap size [default: 100]
    -R, --reg-count <reg-count>    Sets the default virtual register count [default: 1024]
```
