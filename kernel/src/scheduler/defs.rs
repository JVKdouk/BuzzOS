pub mod process {
    use alloc::string::String;

    #[derive(Debug, Copy, Clone)]
    pub enum ProcessState {
        EMBRYO,
        RUNNING,
        READY,
        STOPPED,
        KILLED,
    }

    #[repr(C)]
    #[derive(Default, Debug, Copy, Clone)]
    pub struct TrapFrame {
        pub edi: usize,
        pub esi: usize,
        pub ebp: usize,
        pub _esp: usize, // Unused ESP
        pub ebx: usize,
        pub edx: usize,
        pub ecx: usize,
        pub eax: usize,

        pub gs: u16,
        pub unused_1: u16,
        pub fs: u16,
        pub unused_2: u16,
        pub es: u16,
        pub unused_3: u16,
        pub ds: u16,
        pub unused_4: u16,
        pub trap_number: usize,

        pub err: usize,
        pub eip: usize,
        pub cs: u16,
        pub unused_5: u16,
        pub eflags: usize,

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

    #[derive(Debug)]
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
}

pub mod scheduler {
    use super::process::Process;

    pub struct Scheduler {
        pub current_process: Option<Process>,
        pub context: usize,
    }
}
