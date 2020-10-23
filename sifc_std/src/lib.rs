use sifc_compiler::sifv::SifVal;
use std::collections::HashMap;

/// Std contains the sif std library. This is a small amount of
/// core functions contained in a single map of strings (function names)
/// to function pointers. The implementations are not contained
/// within this struct, but it is responsible for loading the library
/// into memory so a vm can use it.
pub struct Std<'s> {
    lib: HashMap<String, &'s fn(Vec<SifVal>)>,
}

impl<'s> Std<'s> {
    pub fn new() -> Std<'s> {
        Std { lib: load() }
    }

    pub fn call(&self, name: &String, params: Vec<SifVal>) {
        let to_call = self.lib.get(name);
        let f = to_call.unwrap();
        f(params);
    }
}

fn load<'a>() -> HashMap<String, &'a fn(Vec<SifVal>)> {
    let mut std_map = HashMap::new();
    std_map.insert(String::from("print"), &(std_print as fn(Vec<SifVal>)));
    std_map
}

fn std_print(params: Vec<SifVal>) {
    assert!(params.len() == 1);
    let val = &params[0];
    match val {
        SifVal::Num(f) => println!("{}", f),
        SifVal::Str(s) => println!("{}", s),
        SifVal::Bl(b) => println!("{}", b),
        SifVal::Arr(a) => println!("{:?}", a),
        SifVal::Tab(hm) => println!("{:?}", hm),
        SifVal::Null => println!("null"),
    };
}
