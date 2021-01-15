use sifc_analysis::analyzer::Analyzer;
use sifc_bytecode::compiler::Compiler;
use sifc_parse::{lex::Lexer, parser::Parser, symtab::SymTab};

#[test]
fn build_ssa_if_stmt() {
    let program = r"
var x = 5;
var y = 0;
x = x-3;

if x < 3 {
  y = x * 2;
} else {
  y = x-3;
}
var z = x + y;
";

    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(program.as_bytes());
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let parse_result = parser.parse();
    assert_eq!(parse_result.has_err, false);

    let ast = parse_result.ast.unwrap();
    let mut compiler = Compiler::new(&ast);
    let compile_result = compiler.compile();
    assert!(compile_result.err.is_none());

    let analyzer = Analyzer::new(compile_result.program.clone());
    let ssa_cfg = analyzer.build_ssa();

    // Assert that the cfg is of proper form and the ssa instructions are correct,
    // as well as appropriate phi functions. There are only 4 bb's derived from the
    // input, and a single phi function in block 3. We cannot reasonably test everything
    // here, because the order of things like cfg edges and phi operands is not deterministic,
    // and depends on the orer in which nodes are visited in the cfg during analysis.
    let entry_bb = &ssa_cfg.nodes[0];
    assert_eq!(entry_bb.borrow().id, 0);
    assert_eq!(entry_bb.borrow().preds.len(), 0);
    assert_eq!(entry_bb.borrow().edges.len(), 2);
    assert_eq!(entry_bb.borrow().dom_set.len(), 1);
    assert!(entry_bb.borrow().dom_set.contains(&0));
    assert!(entry_bb.borrow().idom.is_none());
    assert_eq!(entry_bb.borrow().dom_front.len(), 0);
    assert_eq!(entry_bb.borrow().phis.len(), 0);

    let bb1 = &ssa_cfg.nodes[1];
    assert_eq!(bb1.borrow().id, 1);
    assert_eq!(bb1.borrow().preds.len(), 1);
    assert_eq!(bb1.borrow().preds[0].borrow().id, 0);
    assert_eq!(bb1.borrow().edges.len(), 1);
    assert_eq!(bb1.borrow().edges[0].borrow().id, 3);
    assert_eq!(bb1.borrow().dom_set.len(), 2);
    assert!(bb1.borrow().dom_set.contains(&0));
    assert!(bb1.borrow().dom_set.contains(&1));
    assert!(bb1.borrow().idom.is_some());
    assert_eq!(bb1.borrow().idom.unwrap(), 0);
    assert_eq!(bb1.borrow().dom_front.len(), 1);
    assert!(bb1.borrow().dom_front.contains(&3));
    assert_eq!(bb1.borrow().phis.len(), 0);

    let bb2 = &ssa_cfg.nodes[2];
    assert_eq!(bb2.borrow().id, 2);
    assert_eq!(bb2.borrow().preds.len(), 1);
    assert_eq!(bb2.borrow().preds[0].borrow().id, 0);
    assert_eq!(bb2.borrow().edges.len(), 1);
    assert_eq!(bb2.borrow().edges[0].borrow().id, 3);
    assert_eq!(bb2.borrow().dom_set.len(), 2);
    assert!(bb2.borrow().dom_set.contains(&0));
    assert!(bb2.borrow().dom_set.contains(&2));
    assert!(bb2.borrow().idom.is_some());
    assert_eq!(bb2.borrow().idom.unwrap(), 0);
    assert_eq!(bb2.borrow().dom_front.len(), 1);
    assert!(bb2.borrow().dom_front.contains(&3));
    assert_eq!(bb2.borrow().phis.len(), 0);

    let bb3 = &ssa_cfg.nodes[3];
    assert_eq!(bb3.borrow().id, 3);
    assert_eq!(bb3.borrow().preds.len(), 2);
    assert_eq!(bb3.borrow().edges.len(), 0);
    assert_eq!(bb3.borrow().dom_set.len(), 2);
    assert!(bb3.borrow().dom_set.contains(&0));
    assert!(bb3.borrow().dom_set.contains(&3));
    assert!(bb3.borrow().idom.is_some());
    assert_eq!(bb3.borrow().idom.unwrap(), 0);
    assert_eq!(bb3.borrow().dom_front.len(), 0);
    assert_eq!(bb3.borrow().phis.len(), 1);

    let phi_map = &bb3.borrow().phis;
    let phi = phi_map.get("y4").unwrap();
    assert_eq!(phi.initial, "y");
    assert_eq!(phi.dest, "y4");
    assert_eq!(phi.operands.len(), 2);
}

#[test]
fn build_ssa_for_stmt() {
    let program = r"
var y = 0;
for x,i in @range(0,3) {
  if x == 0 {
    y = y + x;
  } else {
    y = y * x;
  }
}
";
    let mut symtab = SymTab::new();
    let mut lex = Lexer::new(program.as_bytes());
    let mut parser = Parser::new(&mut lex, &mut symtab);

    let parse_result = parser.parse();
    assert_eq!(parse_result.has_err, false);

    let ast = parse_result.ast.unwrap();
    let mut compiler = Compiler::new(&ast);
    let compile_result = compiler.compile();
    assert!(compile_result.err.is_none());

    let analyzer = Analyzer::new(compile_result.program.clone());
    let ssa_cfg = analyzer.build_ssa();

    let entry_bb = &ssa_cfg.nodes[0];
    assert_eq!(entry_bb.borrow().id, 0);
    assert_eq!(entry_bb.borrow().preds.len(), 0);
    assert_eq!(entry_bb.borrow().edges.len(), 1);
    assert_eq!(entry_bb.borrow().edges[0].borrow().id, 1);
    assert_eq!(entry_bb.borrow().dom_set.len(), 1);
    assert!(entry_bb.borrow().dom_set.contains(&0));
    assert!(entry_bb.borrow().idom.is_none());
    assert_eq!(entry_bb.borrow().dom_front.len(), 0);
    assert_eq!(entry_bb.borrow().phis.len(), 0);
    assert_eq!(entry_bb.borrow().instrs.len(), 10);

    let bb1 = &ssa_cfg.nodes[1];
    assert_eq!(bb1.borrow().id, 1);
    assert_eq!(bb1.borrow().preds.len(), 2);
    assert_eq!(bb1.borrow().edges.len(), 2);
    assert_eq!(bb1.borrow().dom_set.len(), 2);
    assert!(bb1.borrow().dom_set.contains(&0));
    assert!(bb1.borrow().dom_set.contains(&1));
    assert!(bb1.borrow().idom.is_some());
    assert_eq!(bb1.borrow().idom.unwrap(), 0);
    assert_eq!(bb1.borrow().dom_front.len(), 0);
    assert_eq!(bb1.borrow().phis.len(), 2); // TODO: is this correct?
    assert_eq!(bb1.borrow().instrs.len(), 7);

    let bb2 = &ssa_cfg.nodes[2];
    assert_eq!(bb2.borrow().id, 2);
    assert_eq!(bb2.borrow().preds.len(), 1);
    assert_eq!(bb2.borrow().edges.len(), 1);
    assert_eq!(bb2.borrow().dom_set.len(), 3);
    assert!(bb2.borrow().dom_set.contains(&0));
    assert!(bb2.borrow().dom_set.contains(&1));
    assert!(bb2.borrow().dom_set.contains(&2));
    assert!(bb2.borrow().idom.is_some());
    assert_eq!(bb2.borrow().idom.unwrap(), 1);
    assert_eq!(bb2.borrow().dom_front.len(), 1);
    assert_eq!(bb2.borrow().phis.len(), 0);
    assert_eq!(bb2.borrow().instrs.len(), 5);

    let bb3 = &ssa_cfg.nodes[3];
    assert_eq!(bb3.borrow().id, 3);
    assert_eq!(bb3.borrow().preds.len(), 1);
    assert_eq!(bb3.borrow().edges.len(), 1);
    assert_eq!(bb3.borrow().dom_set.len(), 3);
    assert!(bb3.borrow().dom_set.contains(&0));
    assert!(bb3.borrow().dom_set.contains(&1));
    assert!(bb3.borrow().dom_set.contains(&3));
    assert!(bb3.borrow().idom.is_some());
    assert_eq!(bb3.borrow().idom.unwrap(), 1);
    assert_eq!(bb3.borrow().dom_front.len(), 1);
    assert_eq!(bb3.borrow().phis.len(), 0);
    assert_eq!(bb3.borrow().instrs.len(), 4);

    let bb4 = &ssa_cfg.nodes[4];
    assert_eq!(bb4.borrow().id, 4);
    assert_eq!(bb4.borrow().preds.len(), 2);
    assert_eq!(bb4.borrow().edges.len(), 1);
    assert_eq!(bb4.borrow().dom_set.len(), 3);
    assert!(bb4.borrow().dom_set.contains(&0));
    assert!(bb4.borrow().dom_set.contains(&1));
    assert!(bb4.borrow().dom_set.contains(&4));
    assert!(bb4.borrow().idom.is_some());
    assert_eq!(bb4.borrow().idom.unwrap(), 1);
    assert_eq!(bb4.borrow().dom_front.len(), 1);
    assert_eq!(bb4.borrow().phis.len(), 1);
    assert_eq!(bb4.borrow().instrs.len(), 5);

    let phi_map = &bb4.borrow().phis;
    let phi = phi_map.get("y5").unwrap();
    assert_eq!(phi.initial, "y");
    assert_eq!(phi.dest, "y5");
    assert_eq!(phi.operands.len(), 2);
}
