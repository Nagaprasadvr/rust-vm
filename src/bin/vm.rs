use std::{sync::Arc, time::Instant};

fn main() -> Result<(), String> {
    let mut vm = rust_vm_v2::vm::VM::new();
    let mem_cpy = Arc::clone(&vm.memory);
    let mut loaded_idx = 0;

    for i in 0..10 {
        let loaded_size = {
            let mut mem_write_lock = mem_cpy.write().map_err(|e| e.to_string())?;
            let val = mem_write_lock.load_ix(loaded_idx, &[0x01, 0x02, 0x00, 0x01])?;
            if loaded_idx == 0 {
                mem_write_lock.write_data_seg_slice(&[0x12, 0x11, 0x0f, 0x0f, 0x0a], 0)?;
            }
            val
        };
        loaded_idx += loaded_size as u16;
    }

    let start = Instant::now();

    vm.exec_seq()?;

    let duration = start.elapsed();

    println!("Time elapsed in exec is: {:?}", duration);

    Ok(())
}
