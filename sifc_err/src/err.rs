pub trait SifErr {
    fn emit(&self);
    fn to_msg(&self) -> String;
}
