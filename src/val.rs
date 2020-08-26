type SifNum = f64;
type SifStr = String;
type SifBool = bool;

pub trait SifVal {
    fn print(&self);
}

impl SifVal for SifNum {
    fn print(&self) {
        println!("{}", self);
    }
}

impl SifVal for SifStr {
    fn print(&self) {
        println!("{}", self);
    }
}

impl SifVal for SifBool {
    fn print(&self) {
        println!("{}", self);
    }
}
