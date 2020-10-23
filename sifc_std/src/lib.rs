use sifc_compiler::sifv::SifVal;
use std::any::Any;
use std::collections::{HashMap, HashSet};

pub fn call(name: &String) {}

pub fn is_std(name: &String) -> bool {
    let stds = get_std_names();
    stds.contains(name)
}

fn get_std_names() -> HashSet<String> {
    let mut set = HashSet::new();
    set.insert(String::from("print"));
    set
}

fn get_std<'a>() -> HashMap<String, &'a fn(Vec<SifVal>)> {
    let mut std_map = HashMap::new();
    std_map.insert(String::from("print"), &(std_print as fn(Vec<SifVal>)));
    std_map
}

fn std_print(params: Vec<SifVal>) {
    assert!(params.len() == 1);
    let val = &params[0];
    println!("{:?}", val);
}
