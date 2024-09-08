pub const CODE_SEG_SIZE: u16 = 0x4000; // 16KB
pub const DATA_SEG_SIZE: u16 = 0x2000; // 8KB
pub const STACK_SEG_SIZE: u16 = 0xA000; // 40KB

#[derive(Debug)]
pub struct Memory {
    pub code_seg: [u8; 0x4000],  // 16KB (0x4000 is 16384 in decimal)
    pub data_seg: [u8; 0x2000],  // 8KB (0x2000 is 8192 in decimal)
    pub stack_seg: [u8; 0xA000], // 40KB (0xA000 is 40960 in decimal)
}

pub fn check_illegal_mem_access(addr: u16, mem_size: u16) -> Result<(), String> {
    if addr > mem_size {
        return Err(format!("Invalid address: {:#x}", addr));
    }
    Ok(())
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            code_seg: [0; CODE_SEG_SIZE as usize],
            data_seg: [0; DATA_SEG_SIZE as usize],
            stack_seg: [0; STACK_SEG_SIZE as usize],
        }
    }

    pub fn load_ix(&mut self, from_addr: u16, data: &[u8]) -> Result<usize, String> {
        if data.len() > CODE_SEG_SIZE as usize {
            return Err(format!("Data too large to fit in code segment"));
        }
        let start = from_addr as usize;
        let end = start + data.len();
        self.code_seg[start..end].copy_from_slice(data);
        Ok(data.len())
    }

    pub fn write_code_seg(&mut self, data: u8, addr: u16) -> Result<(), String> {
        check_illegal_mem_access(addr, CODE_SEG_SIZE)?;
        self.code_seg[addr as usize] = data;
        Ok(())
    }

    pub fn write_data_seg(&mut self, data: u8, addr: u16) -> Result<(), String> {
        check_illegal_mem_access(addr, DATA_SEG_SIZE)?;
        self.data_seg[addr as usize] = data;
        Ok(())
    }

    pub fn write_data_seg_slice(&mut self, data: &[u8], addr: u16) -> Result<(), String> {
        check_illegal_mem_access(addr, DATA_SEG_SIZE)?;
        check_illegal_mem_access(addr + data.len() as u16, DATA_SEG_SIZE)?;

        let start = addr as usize;
        let end = start + data.len();
        self.data_seg[start..end].copy_from_slice(data);
        Ok(())
    }

    pub fn write_stack_seg(&mut self, data: u8, addr: u16) -> Result<(), String> {
        check_illegal_mem_access(addr, STACK_SEG_SIZE)?;
        self.stack_seg[addr as usize] = data;
        Ok(())
    }

    pub fn read_code_seg(&self, addr: u16) -> Result<u8, String> {
        check_illegal_mem_access(addr, CODE_SEG_SIZE)?;

        Ok(self.code_seg[addr as usize])
    }

    pub fn read_code_seg_slice(&self, addr: u16, size: usize) -> Result<&[u8], String> {
        check_illegal_mem_access(addr as u16, CODE_SEG_SIZE)?;
        check_illegal_mem_access(addr as u16 + size as u16, CODE_SEG_SIZE)?;

        Ok(&self.code_seg[addr as usize..addr as usize + size])
    }

    pub fn read_data_seg(&self, addr: u16) -> Result<u8, String> {
        check_illegal_mem_access(addr, DATA_SEG_SIZE)?;
        Ok(self.data_seg[addr as usize])
    }

    pub fn read_stack_seg(&self, addr: u16) -> Result<u8, String> {
        check_illegal_mem_access(addr, STACK_SEG_SIZE)?;
        Ok(self.stack_seg[addr as usize])
    }

    pub fn read_mem(&self, addr: u16) -> Result<u8, String> {
        if addr < CODE_SEG_SIZE {
            self.read_code_seg(addr)
        } else if addr < CODE_SEG_SIZE + DATA_SEG_SIZE {
            self.read_data_seg(addr - CODE_SEG_SIZE)
        } else {
            self.read_stack_seg(addr - CODE_SEG_SIZE - DATA_SEG_SIZE)
        }
    }
}
