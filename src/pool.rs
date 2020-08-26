use crate::val::SifVal;
use std::collections::HashMap;

pub struct ConstPool {
    pool: HashMap<String, Box<dyn SifVal>>,
}
