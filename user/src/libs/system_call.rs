#![allow(dead_code)]

#[repr(u32)]
enum SystemCallTable {
    PrintTrapFrame = 0,
    Exit = 1,
    Yield = 2,
    Sleep = 3,
    Exec = 5,
    Fork = 6,
    Wait = 7,
}

struct SystemCall {
    number: usize,
    arg0: Option<usize>,
    arg1: Option<usize>,
    arg2: Option<usize>,
}

macro_rules! arg_setup {
    ($name:ident) => {
        pub fn $name(&mut self, data: usize) -> &mut Self {
            self.$name = Some(data);
            self
        }
    };
}

impl SystemCall {
    pub fn new(number: usize) -> Self {
        SystemCall {
            number,
            arg0: None,
            arg1: None,
            arg2: None,
        }
    }

    arg_setup!(arg0);
    arg_setup!(arg1);
    arg_setup!(arg2);

    pub fn call(&self) -> usize {
        let arg0 = self.arg0.unwrap_or(0);
        let arg1 = self.arg1.unwrap_or(0);
        let arg2 = self.arg2.unwrap_or(0);

        unsafe {
            core::arch::asm!(
              "int 64",
              in("edi") arg0,
              in("edx") arg1,
              in("ecx") arg2,
              in("eax") self.number
            )
        }

        let mut out;
        unsafe {
            core::arch::asm!(
              "mov {}, eax",
              out(reg) out,
            )
        };

        out
    }
}

pub fn exec(path: &str) {
    let str_address = path.as_ptr() as usize;
    let str_size = path.len();

    SystemCall::new(SystemCallTable::Exec as usize)
        .arg0(str_address)
        .arg1(str_size)
        .call();
}

pub extern "C" fn fork() -> usize {
    SystemCall::new(SystemCallTable::Fork as usize).call()
}

pub extern "C" fn wait() {
    SystemCall::new(SystemCallTable::Wait as usize).call();
}
