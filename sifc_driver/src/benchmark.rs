use std::time::Duration;

#[derive(Default)]
pub struct Benchmarks {
    pub parse_time: Duration,
    pub compile_time: Duration,
    pub optimize_time: Duration,
    pub vm_time: Duration,
    pub total_time: Duration,
}

impl Benchmarks {
    pub fn emit(&self) {
        println!("=====================");
        println!("SIF BENCHMARKS:");
        println!("=====================");

        println!("parse duration: {:#?}", self.parse_time);
        println!("bytecode compile duration: {:#?}", self.compile_time);
        println!("bytecode optimization duration: {:#?}", self.optimize_time);
        println!("vm execution duration: {:#?}", self.vm_time);
        println!("total program duration: {:#?}", self.total_time);

        println!("=====================");
    }
}
