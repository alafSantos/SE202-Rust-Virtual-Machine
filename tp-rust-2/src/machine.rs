use std::io::{self, Write};

// The memory contains 4096 bytes
const MEMORY_SIZE: usize = 4096;

// There are 16 32-bit registers
const NREGS: usize = 16;

// Register 0 is the instruction pointer (IP)
const IP: usize = 0;

// The memory contains both the program and the data
pub struct Machine {
    memory: [u8; MEMORY_SIZE], // it's addressed from address 0 to address 4095
    regs: [u32; NREGS],        // it's numbered from 0 to 15
}

#[derive(Debug)]
pub enum MachineError {
    NonExistingInstruction, // Non-existing instruction
    NonExistingRegister,    // Non-existing register
    NonExistingAddress,     // Non-existing address
    NonExistingFormat,      // Invalid format
}

impl Machine {
    /// Create a new machine in its reset state. The `memory` parameter will
    /// be copied at the beginning of the machine memory.
    ///
    /// # Panics
    /// This function panics when `memory` is larger than the machine memory.
    pub fn new(memory: &[u8]) -> Self {
        if memory.len() > MEMORY_SIZE {
            panic!();
        }
        let mut machine = Self {
            memory: [0; MEMORY_SIZE],
            regs: [0; NREGS],
        };
        machine.memory[..memory.len()].copy_from_slice(memory); // 'source slice length (840) does not match destination slice length (4096)'
        return machine;
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on `fd`.
    pub fn run_on<T: Write>(&mut self, fd: &mut T) -> Result<(), MachineError> {
        loop {
            if self.step_on(fd)? {
                break;
            }
        }
        return Ok(());
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on standard output.
    pub fn run(&mut self) -> Result<(), MachineError> {
        return self.run_on(&mut io::stdout().lock());
    }

    /// Execute the next instruction by doing the following steps:
    ///   - decode the instruction located at IP (register 0)
    ///   - increment the IP by the size of the instruction
    ///   - execute the decoded instruction
    ///
    /// If output instructions are run, they print on `fd`.
    /// If an error happens at either of those steps, an error is
    /// returned.
    ///
    /// In case of success, `true` is returned if the program is
    /// terminated (upon encountering an exit instruction), or
    /// `false` if the execution must continue.
    pub fn step_on<T: Write>(&mut self, fd: &mut T) -> Result<bool, MachineError> {
        // It contains the address of the next instruction to be executed
        let ip_aux: usize = self.regs[IP].try_into().unwrap();

        if ip_aux < MEMORY_SIZE {
            let instruction: u8 = self.memory[ip_aux];

            let result = match instruction {
                1 => self.move_if(),
                2 => self.store(),
                3 => self.load(),
                4 => self.loadimm(),
                5 => self.sub(),
                6 => self.out(fd),
                7 => self.exit(),
                8 => self.out_number(fd),
                _ => Err(MachineError::NonExistingInstruction),
            };
            if instruction == 7 {
                return result.map(|_| true); // map transforms the result of the match into a Result<bool, MachineError>
            }
            return result.map(|_| false); // map transforms the result of the match into a Result<bool, MachineError>
        }
        return Err(MachineError::NonExistingAddress);
    }

    /// Similar to [step_on](Machine::step_on).
    /// If output instructions are run, they print on standard output.
    pub fn step(&mut self) -> Result<bool, MachineError> {
        return self.step_on(&mut io::stdout().lock());
    }

    /// Reference onto the machine current set of regs.
    pub fn regs(&self) -> &[u32] {
        return &self.regs;
    }

    /// Sets a register to the given value.
    pub fn set_reg(&mut self, reg: usize, value: u32) -> Result<(), MachineError> {
        if reg < NREGS {
            self.regs[reg] = value;
            return Ok(());
        }
        return Err(MachineError::NonExistingRegister);
    }

    /// Reference onto the machine current memory.
    pub fn memory(&self) -> &[u8] {
        return &self.memory;
    }

    /**
     * Instruction Set
     */

    // --------auxiliary functions--------
    pub fn ip_sum(&self, offset: usize) -> usize {
        return (self.regs[IP] as usize) + offset;
    }

    pub fn ip_inc(&mut self, offset: u32) -> () {
        self.regs[IP] += offset;
    }
    // -----------------------------------

    /**
     * 1 reg_a reg_b reg_c: if register reg_c contains a non-zero value,
     * copy the content of register reg_b into register reg_a; otherwise do nothing.
     */
    fn move_if(&mut self) -> Result<bool, MachineError> {
        let reg_a: usize = self.memory[self.ip_sum(1)] as usize;
        let reg_b: usize = self.memory[self.ip_sum(2)] as usize;
        let reg_c: usize = self.memory[self.ip_sum(3)] as usize;

        self.ip_inc(4);

        if reg_c < NREGS && reg_b < NREGS {
            if self.regs[reg_c] == 0 {
                return Ok(false);
            }
            /*
             The ? at the end of the call to self.set_reg, which returns an Ok(()) if we got success.
             The function always returns Ok(true) if everything is ok.
            */
            self.set_reg(reg_a.into(), self.regs[reg_b as usize])?;
            return Ok(false);
        }
        return Err(MachineError::NonExistingRegister);
    }

    /**
     * 2 reg_a reg_b: store the content of register reg_b into the memory starting
     * at address pointed by register reg_a using little-endian representation.
     */
    fn store(&mut self) -> Result<bool, MachineError> {
        let reg_a: usize = self.memory[self.ip_sum(1)] as usize;
        let reg_b: usize = self.memory[self.ip_sum(2)] as usize;

        self.set_reg(IP, self.ip_sum(3) as u32)?;

        if reg_a < NREGS && reg_b < NREGS {
            let bytes: [u8; 4] = self.regs[reg_b].to_le_bytes();

            for i in 0..=3 {
                let index = (self.regs[reg_a] + i) as usize;
                if index < MEMORY_SIZE {
                    self.memory[index] = bytes[i as usize];
                } else {
                    return Err(MachineError::NonExistingAddress);
                }
            }
            return Ok(false);
        }

        return Err(MachineError::NonExistingRegister);
    }

    /**
     * 3 reg_a reg_b: load the 32-bit content from memory at address pointed by
     * register reg_b into register reg_a using little-endian representation.
     */
    fn load(&mut self) -> Result<bool, MachineError> {
        let reg_a: usize = self.memory[self.ip_sum(1)] as usize;
        let reg_b: usize = self.memory[self.ip_sum(2)] as usize;

        self.ip_inc(3);

        if reg_a < NREGS && reg_b < NREGS {
            let mut value: u32;
            value = 0;
            for i in 0..=3 {
                let index = (self.regs[reg_b] + i) as usize;
                if index < MEMORY_SIZE {
                    value += (self.memory[index] as u32) << i * 8;
                } else {
                    return Err(MachineError::NonExistingAddress);
                }
            }
            self.regs[reg_a] = value;
            return Ok(false);
        }

        return Err(MachineError::NonExistingRegister);
    }

    /**
     * 4 reg_a L H: interpret H and L respectively as the high-order and the low-order bytes
     * of a 16-bit signed value, sign-extend it to 32 bits, and store it into register reg_a.
     */
    fn loadimm(&mut self) -> Result<bool, MachineError> {
        let reg_a: usize = self.memory[self.ip_sum(1)] as usize;
        let l: u16 = self.memory[self.ip_sum(2)] as u16;
        let h: u16 = self.memory[self.ip_sum(3)] as u16;

        self.ip_inc(4);

        let value: u32 = (((h as i16) << 8) + (l as i16)) as u32;

        if reg_a < NREGS {
            self.set_reg(reg_a, value)?;
            return Ok(false);
        }
        return Err(MachineError::NonExistingRegister);
    }

    /**
     * 5 reg_a reg_b reg_c: store the content of register reg_b minus the
     * content of register reg_c into register reg_a.
     */
    fn sub(&mut self) -> Result<bool, MachineError> {
        let reg_a: usize = self.memory[self.ip_sum(1)] as usize;
        let reg_b: usize = self.memory[self.ip_sum(2)] as usize;
        let reg_c: usize = self.memory[self.ip_sum(3)] as usize;

        self.ip_inc(4);

        if reg_a < NREGS && reg_b < NREGS && reg_c < NREGS {
            self.set_reg(reg_a, u32::wrapping_sub(self.regs[reg_b], self.regs[reg_c]))?;
            return Ok(false);
        }

        return Err(MachineError::NonExistingRegister);
    }

    /**
     * 6 reg_a: output the character whose unicode value is stored in
     * the 8 low bits of register reg_a.
     */
    fn out<T: Write>(&mut self, fd: &mut T) -> Result<bool, MachineError> {
        let reg_a: usize = self.memory[self.ip_sum(1)] as usize;

        self.ip_inc(2);

        if reg_a < NREGS {
            let character_v = 0x000000FF & self.regs[reg_a];
            let character = char::from_u32(character_v as u32).unwrap();
            let result = write!(fd, "{}", character);

            match result {
                Ok(_) => return Ok(false),
                Err(_) => return Err(MachineError::NonExistingFormat),
            }
        }
        return Err(MachineError::NonExistingRegister);
    }

    /**
     * 7: exit the current program.
     */
    fn exit(&mut self) -> Result<bool, MachineError> {
        self.ip_inc(1);
        return Ok(true);
    }

    /**
     * 8 reg_a: output the signed number stored in register reg_a in decimal.
     */
    fn out_number<T: Write>(&mut self, fd: &mut T) -> Result<bool, MachineError> {
        let reg_a: usize = self.memory[self.ip_sum(1)] as usize;
        self.ip_inc(2);

        if reg_a < NREGS {
            let decimal = self.regs[reg_a] as i32;
            let result = write!(fd, "{}", decimal);

            match result {
                Ok(_) => return Ok(false),
                Err(_) => return Err(MachineError::NonExistingFormat),
            }
        }

        return Err(MachineError::NonExistingRegister);
    }
}
