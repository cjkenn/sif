use sifc_bytecode::compiler::Compiler;
use sifc_err::err::SifErr;
use sifc_parse::{lex::Lexer, parser::Parser, symtab::SymTab};
use sifc_vm::{config::VMConfig, vm::VM};

// Initial take on a macro for embedding sif programs in rust.
//
// TODO: Where does this go for exporting?
// TODO: How will error handling work?
// TODO: What values can be returned? Do we run the VM in "expression" mode,
//       where the last evaluated expression (last written register?) is returned?
macro_rules! sif {
    ($input:expr) => {
        let mut symtab = SymTab::new();
        let mut lex = Lexer::new($input);
        let mut parser = Parser::new(&mut lex, &mut symtab);
        let pr = parser.parse();

        let ast = pr.ast.unwrap();
        let mut comp = Compiler::new(&ast);
        let cr = comp.compile();

        let program = cr.program;
        let code_start = cr.code_start;
        let jumptab = cr.jumptab;
        let fntab = cr.fntab;
        let conf = VMConfig {
            trace: false,
            initial_heap_size: 32,
            initial_dreg_count: 64,
        };
        let mut vm = VM::init(program, code_start, jumptab, fntab, conf);
        vm.run()
    };
}
