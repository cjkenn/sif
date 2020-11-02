# sifc_bytecode

Sif's bytecode consists of the opcodes contained in [opc.rs](./sifc_bytecode/src/opc.rs). Opcodes
follow a standard 3-address linear format: `<op> <src> <src> <dest>`. Of course, some ops take less arguments,
but in general the destintation of an operation (if applicable) is the last argument to it.

| Opcode | Usage                | Description                       |
|--------|----------------------|-----------------------------------|
| `add`  | `add <src1> <src2> <dest>` | Addition |
| `sub`  | `sub <src1> <src2> <dest>` | Subtraction |
| `mul`  | `mul <src1> <src2> <dest>` | Multiplication |
| `div`  | `div <src1> <src2> <dest>` | Division |
| `modu`  | `modu <src1> <src2> <dest>` | Modulus |
| `eq`  | `eq <src1> <src2> <dest>` | Equality |
| `neq`  | `neq <src1> <src2> <dest>` | Not equal |
| `lt`  | `lt <src1> <src2> <dest>` | Less than |
| `gt`  | `gt <src1> <src2> <dest>` | Greater than |
| `lte`  | `lte <src1> <src2> <dest>` | Less than equal |
| `gte`   | `gte <src1> <src2> <dest>` | Greater equal |
| `land`  | `land <src1> <src2> <dest>` | Logical and |
| `lnot`  | `lnot <src1> <src2> <dest>` | Logical not |
| `lor`  | `lor <src1> <src2> <dest>` | Logical or |
| `lneg`  | `lneg <src> <dest>` | Logical negation |
| `nneg`  | `nneg <src> <dest>` | Numerical negation |
| `ldc`  | `ldc <val> <dest>` | Loads a constant into a register |
| `ldn`  | `ldn <name> <dest>` | Loads a named value from memory into a register |
| `mv`   | `mv <src> <dest>` | Moves a value from a register to another |
| `ldas` | `ldas <name> <dest>` | Loads the size of the array `<name>` into a register |
| `ldav` | `ldav <name> <idx> <dest>` | Loads the value at index `<idx>` for array `<name>` into a register |
| `upda` | `upda <name> <idx> <val>` | Updates the value at index `<idx>` to `<val>` for array `<name` |
| `stc` | `stc <val> <name>` | Stores a value into a named variable in memory |
| `stn` | `stn <name> <name>` | Stores the value of a named variable into memory |
| `str` | `str <src> <name>` | Stores the value of a register into memory |
| `jmpf` | `jmpf <src> <lbl>` | Jumps to `<lbl>` if `<src>` is false |
| `jmpt` | `jmpt <src> <lbl>` | Jumps to `<lbl>` if `<src>` is true |
| `jmpa` | `jmpa <lbl>` | Always jumps to `<lbl>` |
| `incrr` | `incrr <src>` | Increments value in register |
| `decrr` | `decrr <src>` | Decrements value in register |
| `fn` | `@fn <name> <params>` | Function declaration |
| `call` | `call <name>` | Call a function `<name>` |
| `stdcall` | `sdtcall <name>` | Call a std lib function |
| `ret` | `ret` | Return from a function call |
| `fstpush` | `fstpush <src>` | Push a value in `<src>` onto the function stack |
| `fstpop` | `fstpop <dest>` | Pop a value from the function stack into `<dest>` |
| `tbli` | `tbli <name> <key> <src>` | Insert the value in `<src>` into table `<name>` under `<key>` |
| `tblg` | `tblg <name> <key> <dest>` | Put `<key>` from table `<name>` into `<dest>` |
| `nop` | `nop` | No-op |
| `stop` | `stop` | Halt vm execution |
