pub mod process {
    use alloc::{string::String, vec::Vec};

    #[derive(Debug, Copy, Clone, PartialEq)]
    pub enum ProcessState {
        EMPTY,
        EMBRYO,
        RUNNING,
        READY,
        KILLED,
    }

    #[repr(C)]
    #[derive(Default, Debug, Copy, Clone)]
    pub struct TrapFrame {
        // This section is pushed to the stack by x86 instruction "pusha"
        pub edi: usize,
        pub esi: usize,
        pub ebp: usize,
        pub old_esp: usize,
        pub ebx: usize,
        pub edx: usize,
        pub ecx: usize,
        pub eax: usize,

        // This section is manually added to the stack
        pub gs: u16,
        pub unused_1: u16,
        pub fs: u16,
        pub unused_2: u16,
        pub es: u16,
        pub unused_3: u16,
        pub ds: u16,
        pub unused_4: u16,

        // This part is added during a sustem call
        pub trap_number: usize,
        pub err: usize,

        // This part is used during the iret to return to the user-space
        pub eip: usize,
        pub cs: u16,
        pub unused_5: u16,
        pub eflags: usize,

        // Values below are currently unused
        pub esp: usize,
        pub ss: u16,
        pub unused_6: u16,
    }

    #[repr(C)]
    #[derive(Default, Debug, Clone)]
    pub struct Context {
        pub edi: usize,
        pub esi: usize,
        pub ebx: usize,
        pub ebp: usize,
        pub eip: usize,
    }

    #[repr(C)]
    #[derive(Debug, Clone)]
    pub struct Process {
        pub pid: usize,
        pub pgdir: Option<*mut usize>,
        pub state: ProcessState,
        pub context: Option<*mut Context>,
        pub trapframe: Option<*mut TrapFrame>,
        pub kernel_stack: Option<*mut usize>,
        pub mem_size: usize,
        pub current_working_directory: String,
        pub name: String,
    }

    pub struct ProcessList(pub Vec<Process>);
}

pub mod scheduler {
    use super::process::{Context, Process};

    // Number of processes that can run at the same time.
    pub const NUM_PROCESS: usize = 1000;

    pub enum SchedulerState {
        READY,
        BUSY,
    }

    pub struct Scheduler {
        pub current_process: Option<Process>,
        pub context: *mut Context,
        pub status: SchedulerState,
    }
}
