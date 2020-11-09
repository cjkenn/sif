use sifc_bytecode::sifv::SifVal;
use std::collections::HashMap;

/// Std contains the sif std library. This is a small amount of
/// core functions contained in a single map of strings (function names)
/// to function pointers. The implementations are not contained
/// within this struct, but it is responsible for loading the library
/// into memory so a vm can use it.
pub struct Std<'s> {
    lib: HashMap<String, &'s fn(Vec<SifVal>) -> SifVal>,
}

impl<'s> Std<'s> {
    pub fn new() -> Std<'s> {
        Std { lib: load() }
    }

    pub fn call(&self, name: &String, params: Vec<SifVal>) -> SifVal {
        let to_call = self.lib.get(name);
        let f = to_call.unwrap();
        f(params)
    }
}

fn load<'a>() -> HashMap<String, &'a fn(Vec<SifVal>) -> SifVal> {
    let mut std_map = HashMap::new();
    std_map.insert(
        String::from("print"),
        &(std_print as fn(Vec<SifVal>) -> SifVal),
    );
    std_map.insert(
        String::from("range"),
        &(std_range as fn(Vec<SifVal>) -> SifVal),
    );
    std_map
}

/// Implements the print function inside the std lib. This uses the
/// fmt::Display formatter implemented by SifVal.
/// @print(value)
fn std_print(params: Vec<SifVal>) -> SifVal {
    // TODO: How to handle wrong params here? Do we print any amount?
    assert!(params.len() == 1);
    let val = &params[0];
    println!("{:#}", val);
    SifVal::Null
}

fn std_range(params: Vec<SifVal>) -> SifVal {
    let start = params[1].extract_num() as i64;
    let end = params[0].extract_num() as i64;
    let mut range = Vec::new();

    for i in start..end + 1 {
        range.push(SifVal::Num(i as f64));
    }
    SifVal::Arr(range)
}
