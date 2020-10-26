#[derive(Clone, Debug)]
pub struct VMConfig {
    /// Indicates whether or not the VM should trace execution and print
    /// currently executing instructions to stdout.
    pub trace: bool,

    /// Starting heap size to reserve when creating the VM.
    pub initial_heap_size: usize,

    /// Starting amount of data registers to make when creating the VM.
    pub initial_dreg_count: usize,
}
