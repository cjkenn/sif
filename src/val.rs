type SifNum = f64;
type SifStr = String;
type SifBool = bool;

pub trait SifVal {
    fn print(&self);
    fn to_string(&self) -> String;
}

impl SifVal for SifNum {
    fn print(&self) {
        println!("{}", self);
    }

    fn to_string(&self) -> String {
        std::string::ToString::to_string(&self)
    }
}

impl SifVal for SifStr {
    fn print(&self) {
        println!("{}", self);
    }

    fn to_string(&self) -> String {
        std::string::ToString::to_string(&self)
    }
}

impl SifVal for SifBool {
    fn print(&self) {
        println!("{}", self);
    }

    fn to_string(&self) -> String {
        std::string::ToString::to_string(&self)
    }
}
