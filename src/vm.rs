use std::{
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
};

use crate::memory::{self, Memory};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
    SP,
    PC,
    DP,
}

impl TryFrom<u8> for Register {
    type Error = String;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Register::A),
            1 => Ok(Register::B),
            2 => Ok(Register::C),
            3 => Ok(Register::D),
            4 => Ok(Register::E),
            5 => Ok(Register::F),
            6 => Ok(Register::H),
            7 => Ok(Register::L),
            8 => Ok(Register::SP),
            9 => Ok(Register::PC),
            10 => Ok(Register::DP),
            _ => Err(format!("Invalid register: {:#x}", val)),
        }
    }
}

impl Register {
    pub fn into_usize(&self) -> usize {
        *self as usize
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum IxType {
    NOP,
    MOV,
    LDM,
    STM,
    ADD,
}

pub const IX_SIZE_OFFSET: u16 = 1;

pub const IX_META_SIZE: u16 = 2;

pub const IX_DATA_OFFSET: u16 = 2;

pub const CONCURRENT_THREADS: u8 = 10;

impl TryFrom<u8> for IxType {
    type Error = String;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(IxType::NOP),
            1 => Ok(IxType::MOV),
            2 => Ok(IxType::LDM),
            3 => Ok(IxType::STM),
            4 => Ok(IxType::ADD),
            _ => Err(format!("Invalid instruction: {:#x}", val)),
        }
    }
}

fn get_addr_from_two_bytes(high: u8, low: u8) -> u16 {
    let high: u16 = (high as u16) << 8;
    let low = low as u16;
    high | low
}

type RegisterArray = [u16; 10];

#[derive(Debug)]
pub struct Ix {
    pub ix_type: IxType,
    pub ix_data_size: u8,
    pub ix_data: Vec<u8>,
}

#[derive(Debug)]

pub struct VM {
    pub registers: Arc<RwLock<RegisterArray>>,
    pub memory: Arc<RwLock<Memory>>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            registers: Arc::new(RwLock::new([0; 10])),
            memory: Arc::new(RwLock::new(Memory::new())),
        }
    }

    pub fn inc_reg(&mut self, reg: Register, inc_by: u16) {
        let mut registers_write_lock = self.registers.write().unwrap();
        registers_write_lock[reg.into_usize()] += inc_by as u16;
    }

    pub fn dec_reg(&mut self, reg: Register, dec_by: u16) {
        let mut registers_write_lock = self.registers.write().unwrap();
        registers_write_lock[reg.into_usize()] -= dec_by as u16;
    }

    pub fn print_ix(ix: IxType, ix_data_size: u8, ix_data: &[u8]) {
        println!(
            "ix: {:?}, ix_size: {:?}, ix_data: {:?}",
            ix, ix_data_size, ix_data
        );
    }

    pub fn parseand_exec_ixs_seq(&mut self) -> Result<(), String> {
        let mem_cpy = Arc::clone(&self.memory);

        let reg_cpy = Arc::clone(&self.registers);

        loop {
            let reg_read_lock = reg_cpy.read().map_err(|e| e.to_string())?;
            let pc = reg_read_lock[Register::PC.into_usize()];
            let mem_read_lock = mem_cpy.read().map_err(|e| e.to_string())?;
            let ix = mem_read_lock.read_code_seg(pc)?;

            if ix == IxType::NOP as u8 {
                break;
            }

            let ix_data_size = mem_read_lock.read_code_seg(pc + IX_SIZE_OFFSET)?;

            let ix_data =
                mem_read_lock.read_code_seg_slice(pc + IX_DATA_OFFSET, ix_data_size as usize)?;

            let inx = Ix {
                ix_type: IxType::try_from(ix)?,
                ix_data_size,
                ix_data: ix_data.to_vec(),
            };

            drop(mem_read_lock);
            drop(reg_read_lock);

            Self::exec_ix(Arc::clone(&mem_cpy), Arc::clone(&reg_cpy), inx)?;
        }

        Ok(())
    }

    pub fn parse_and_exec_ixs_concurrent(&mut self) -> Result<(), String> {
        let mem_cpy = Arc::clone(&self.memory);
        let reg_cpy = Arc::clone(&self.registers);

        let mut threads: Vec<JoinHandle<Result<(), String>>> = Vec::new();

        let mut ixs_count = 0;

        let mut ix_pointer = {
            let reg_read_lock = reg_cpy.read().map_err(|e| e.to_string())?;
            let pc = reg_read_lock[Register::PC.into_usize()];
            pc
        };

        loop {
            let mem_read_lock = mem_cpy.read().map_err(|e| e.to_string())?;
            let ix = mem_read_lock.read_code_seg(ix_pointer)?;
            let ix_data_size = mem_read_lock.read_code_seg(ix_pointer + IX_SIZE_OFFSET)?;

            if ix == IxType::NOP as u8 {
                break;
            }
            ixs_count += 1;

            ix_pointer += ix_data_size as u16 + IX_META_SIZE;
            drop(mem_read_lock);
        }

        for _ in 0..(ixs_count / 10) {
            let mem_cpy = Arc::clone(&mem_cpy);
            let reg_cpy = Arc::clone(&reg_cpy);

            threads.push(thread::spawn(move || -> Result<(), String> {
                for _ in 0..10 {
                    let mem_read_lock = mem_cpy.read().map_err(|e| e.to_string())?;
                    let reg_read_lock = reg_cpy.read().map_err(|e| e.to_string())?;

                    let pc = reg_read_lock[Register::PC.into_usize()];

                    let ix = mem_read_lock.read_code_seg(pc)?;

                    let ix_data_size = mem_read_lock.read_code_seg(pc + IX_SIZE_OFFSET)?;

                    let ix_data = mem_read_lock
                        .read_code_seg_slice(pc + IX_DATA_OFFSET, ix_data_size as usize)?;

                    let inx = Ix {
                        ix_type: IxType::try_from(ix)?,
                        ix_data_size,
                        ix_data: ix_data.to_vec(),
                    };
                    drop(mem_read_lock);
                    drop(reg_read_lock);

                    Self::exec_ix(Arc::clone(&mem_cpy), Arc::clone(&reg_cpy), inx)?;
                }

                Ok(())
            }))
        }

        println!("threads: {:?}", threads.len());
        for thread in threads {
            thread.join().map_err(|_| "Thread panicked".to_string())??;
        }

        Ok(())
    }

    pub fn exec_ix(
        mem_cpy: Arc<RwLock<Memory>>,
        reg_cpy: Arc<RwLock<RegisterArray>>,
        inx: Ix,
    ) -> Result<(), String> {
        let Ix {
            ix_type,
            ix_data_size,
            ix_data,
        } = inx;

        Self::print_ix(ix_type, ix_data_size, &ix_data);

        match ix_type {
            IxType::NOP => {
                // do nothing
            }
            IxType::MOV => {
                let reg = Register::try_from(ix_data[0])?;
                let mut reg_write_lock = reg_cpy.write().map_err(|e| e.to_string())?;
                reg_write_lock[reg.into_usize()] = ix_data[1] as u16;
                drop(reg_write_lock);
            }
            IxType::LDM => {
                let addr = get_addr_from_two_bytes(ix_data[0], ix_data[1]);
                let mem_read_lock = mem_cpy.read().map_err(|e| e.to_string())?;
                let data = mem_read_lock.read_data_seg(addr)?;
                let reg = Register::try_from(ix_data[2])?;
                let mut reg_write_lock = reg_cpy.write().unwrap();
                reg_write_lock[reg.into_usize()] = data as u16;
            }
            IxType::STM => {
                let addr = get_addr_from_two_bytes(ix_data[0], ix_data[1]);
                let reg_read_lock = reg_cpy.read().map_err(|e| e.to_string())?;
                let reg_val = reg_read_lock[ix_data[2] as usize] as u8;
                mem_cpy
                    .write()
                    .unwrap()
                    .write_data_seg(reg_val as u8, addr)?;
            }
            IxType::ADD => {
                let addr = get_addr_from_two_bytes(ix_data[0], ix_data[1]);
                let reg = Register::try_from(ix_data[2])?;
                let reg_read_lock = reg_cpy.read().map_err(|e| e.to_string())?;
                let reg_val = reg_read_lock[reg.into_usize()];

                let mem_read_lock = mem_cpy.read().map_err(|e| e.to_string())?;
                let data = mem_read_lock.read_data_seg(addr)?;
                let mut reg_write_lock = reg_cpy.write().unwrap();
                reg_write_lock[reg.into_usize()] = reg_val.wrapping_add(data as u16);
            }
        }

        let mut reg_write_lock = reg_cpy.write().unwrap();
        reg_write_lock[Register::PC.into_usize()] += ix_data_size as u16 + IX_META_SIZE;

        Ok(())
    }

    pub fn exec_seq(&mut self) -> Result<(), String> {
        self.parseand_exec_ixs_seq()?;
        Ok(())
    }

    pub fn exec_concurrent(&mut self) -> Result<(), String> {
        self.parse_and_exec_ixs_concurrent()?;
        Ok(())
    }
}
