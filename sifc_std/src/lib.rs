use std::collections::HashSet;

pub fn is_std(name: &String) -> bool {
    let stds = get_std_names();
    stds.contains(name)
}

fn get_std_names() -> HashSet<String> {
    let mut set = HashSet::new();
    set.insert(String::from("print"));
    set
}
