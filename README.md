# sif

![sif-build](https://github.com/cjkenn/sif/workflows/sif-build/badge.svg?branch=master)

sif is a scripting language with c-style syntax. It contains a bytecode compiler, optimizer and a register based vm. It's small and easily embeddable into rust programs.

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

var nums = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];
for i, v in nums {
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

