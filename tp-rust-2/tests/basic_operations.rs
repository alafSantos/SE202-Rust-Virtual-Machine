use interpreter::Machine;
use std::io::{self, Write};

#[test]
fn create_with_memory() {
    let memory = vec![1, 2, 3];
    let machine = Machine::new(&memory);
    assert_eq!(&memory[..], &machine.memory()[..memory.len()],);
    assert!(machine.memory()[memory.len()..].iter().all(|b| *b == 0),);
    assert!(machine.regs().iter().all(|v| *v == 0),);
}

#[test]
#[should_panic]
fn create_with_too_large_a_memory() {
    Machine::new(&[0; 4097]);
}

#[test]
fn refuse_illegal_instruction() {
    let mut machine = Machine::new(&[]);
    assert!(machine.step().is_err());
}

fn expect_on<T: Write>(machine: &mut Machine, fd: &mut T, end: bool, new_ip: usize) {
    match machine.step_on(fd) {
        Ok(r) if r == end => (),
        _ => panic!(),
    }
    assert_eq!(new_ip, machine.regs()[0] as usize,);
}

fn expect(machine: &mut Machine, end: bool, new_ip: usize) {
    expect_on(machine, &mut io::stdout().lock(), end, new_ip)
}

#[test]
fn test_exit() {
    // 0: exit
    // 1:
    let mut machine = Machine::new(&[7]);
    expect(&mut machine, true, 1);
}

#[test]
fn ip_reg() {
    // 0: invalid
    // 1: exit
    // 2:
    let mut machine = Machine::new(&[0, 7]);
    machine.set_reg(0, 1).unwrap();
    expect(&mut machine, true, 2);
}

#[test]
fn test_move_if() {
    // 0: move r1 <- r2 if r2 != 0
    // 4: move r3 <- r2 if r3 != 0
    // 8:
    let mut machine = Machine::new(&[1, 1, 2, 2, 1, 3, 2, 3]);
    machine.set_reg(2, 42).unwrap();
    expect(&mut machine, false, 4);
    assert_eq!(42, machine.regs()[1]);
    expect(&mut machine, false, 8);
    assert_eq!(0, machine.regs()[3]);
}

#[test]
fn test_move_if_out_of_bounds() {
    // 0: move r1 <- r100 if r0 != 0
    // 4:
    let mut machine = Machine::new(&[1, 1, 100, 0]);
    assert!(machine.step().is_err());

    // 0: move r100 <- r1 if r0 != 0
    // 4:
    let mut machine = Machine::new(&[1, 100, 1, 0]);
    assert!(machine.step().is_err());

    // 0: move r1 <- r1 if r100 != 0
    // 4:
    let mut machine = Machine::new(&[1, 1, 1, 100]);
    assert!(machine.step().is_err());
}

#[test]
fn early_ip_advance() {
    // 0: move r1 <- r0 if r0 != 0
    // 4: move r0 <- r1 if r0 != 0
    let mut machine = Machine::new(&[1, 1, 0, 0, 1, 0, 1, 0]);
    expect(&mut machine, false, 4);
    assert_eq!(4, machine.regs()[1],);

    expect(&mut machine, false, 4);
    assert_eq!(4, machine.regs()[1]);

    expect(&mut machine, false, 4);
    assert_eq!(4, machine.regs()[1],);
}

#[test]
fn test_load() {
    // 0: load r1 <- [r0]
    // 3: 42
    let mut machine = Machine::new(&[3, 1, 0, 42]);
    expect(&mut machine, false, 3);
    assert_eq!(42, machine.regs()[1]);

    // 0: load r0 <- [r0]
    // 4: 42
    let mut machine = Machine::new(&[3, 0, 0, 42]);
    expect(&mut machine, false, 42);

    // 0: load r1 <- [r0]
    // 4: 1
    // 5: 2
    // 6: 3
    // 7: 4
    let mut machine = Machine::new(&[3, 1, 0, 1, 2, 3, 4]);
    expect(&mut machine, false, 3);
    assert_eq!(0x04030201, machine.regs()[1]);
}

#[test]
fn test_load_out_of_bounds() {
    // 0: load r100 <- [r1]
    // 3:
    let mut machine = Machine::new(&[3, 100, 1]);
    assert!(machine.step().is_err());

    // 0: load r1 <- [r100]
    // 3:
    let mut machine = Machine::new(&[3, 1, 100]);
    assert!(machine.step().is_err());

    // 0: load r1 <- [r1] with r1 == 30000
    // 3:
    let mut machine = Machine::new(&[3, 1, 1]);
    machine.set_reg(1, 30000).unwrap();
    assert!(machine.step().is_err());
}

#[test]
fn test_store() {
    // 0: store [r0] <- r1
    // 3:
    let mut machine = Machine::new(&[2, 0, 1]);
    machine.set_reg(1, 0x01020304).unwrap();
    expect(&mut machine, false, 3);
    assert_eq!(&[04, 03, 02, 01], &machine.memory()[3..7]);
}

#[test]
fn test_store_out_of_bounds() {
    // 0: store [r100] <- r1
    // 3:
    let mut machine = Machine::new(&[2, 100, 1]);
    assert!(machine.step().is_err());

    // 0: store [r1] <- r100
    // 3:
    let mut machine = Machine::new(&[2, 1, 100]);
    assert!(machine.step().is_err());

    // 0: store [r1] <- r1 with r1 == 30000
    // 3:
    let mut machine = Machine::new(&[2, 1, 1]);
    machine.set_reg(1, 30000).unwrap();
    assert!(machine.step().is_err());
}

#[test]
fn test_load_imm() {
    // 0: loadimm r1, 0x1234
    // 4:
    let mut machine = Machine::new(&[4, 1, 0x34, 0x12]);
    expect(&mut machine, false, 4);
    assert_eq!(0x1234, machine.regs()[1]);

    // 0: loadimm r1, 0xfffe (corresponds to -2)
    // 4:
    let mut machine = Machine::new(&[4, 1, 0xfe, 0xff]);
    expect(&mut machine, false, 4);
    assert_eq!(0xfffffffe, machine.regs()[1]);
}

#[test]
fn test_load_imm_out_of_bounds() {
    // 0: loadimm r100, 0
    // 4:
    let mut machine = Machine::new(&[4, 100, 0, 0]);
    assert!(machine.step().is_err());
}

#[test]
fn test_sub() {
    // 0: sub r2 <- r1 - r0
    // 4:
    let mut machine = Machine::new(&[5, 2, 1, 0]);
    expect(&mut machine, false, 4);
    assert_eq!(0xfffffffc, machine.regs()[2]);
}

#[test]
fn test_sub_out_of_bounds() {
    // 0: sub r100 <- r0 - r0
    // 4:
    let mut machine = Machine::new(&[5, 100, 0, 0]);
    assert!(machine.step().is_err());

    // 0: sub r0 <- r100 - r0
    // 4:
    let mut machine = Machine::new(&[5, 0, 100, 0]);
    assert!(machine.step().is_err());

    // 0: sub r0 <- r0 - r100
    // 4:
    let mut machine = Machine::new(&[5, 0, 0, 100]);
    assert!(machine.step().is_err());
}

#[test]
fn test_out() {
    // 0: out r1
    // 2:
    let mut machine = Machine::new(&[6, 1]);
    machine.set_reg(1, b'A' as u32).unwrap();
    let mut out = Vec::new();
    expect_on(&mut machine, &mut out, false, 2);
    assert_eq!("A".as_bytes(), &out[..]);
}

#[test]
fn test_out_number() {
    // 0: out_number r0
    // 2:
    let mut machine = Machine::new(&[8, 0]);
    let mut out = Vec::new();
    expect_on(&mut machine, &mut out, false, 2);
    assert_eq!("2".as_bytes(), &out[..]);

    // 0: out_number r1
    // 2:
    let mut machine = Machine::new(&[8, 1]);
    let mut out = Vec::new();
    machine.set_reg(1, -1234i32 as u32).unwrap();
    expect_on(&mut machine, &mut out, false, 2);
    assert_eq!("-1234".as_bytes(), &out[..]);
}

#[test]
fn test_run_on() {
    // 0: out_number r0
    // 2: out_number r0
    // 4: exit
    // 5:
    let mut machine = Machine::new(&[8, 0, 8, 0, 7]);
    let mut out = Vec::new();
    machine.run_on(&mut out).unwrap();
    assert_eq!("24".as_bytes(), &out[..]);
}

#[test]
fn test_run() {
    // 0: sub r1 <- r1 - r0
    // 4: sub r1 <- r1 - r0
    // 8: exit
    // 9:
    let mut machine = Machine::new(&[5, 1, 1, 0, 5, 1, 1, 0, 7]);
    machine.run().unwrap();
    assert_eq!(9, machine.regs()[0]);
    assert_eq!(-12, machine.regs()[1] as i32);
}

#[test]
fn end_of_memory() {
    // memory_size-1: exit
    // memory_size  :
    let mut memory = Machine::new(&[]).memory().to_vec();
    let memory_size = memory.len();
    memory[memory_size - 1] = 7;
    let mut machine = Machine::new(&memory);
    machine.set_reg(0, (memory_size - 1) as u32).unwrap();
    expect(&mut machine, true, memory_size);
}

#[test]
fn no_wraparound_past_end_of_memory() {
    // memory_size-4: move r1 <- r1 if r1
    // 0:             exit
    // 1:
    let mut memory = Machine::new(&[]).memory().to_vec();
    let memory_size = memory.len();
    for i in memory_size - 4..memory_size {
        memory[i] = 1;
    }
    memory[0] = 7;
    let mut machine = Machine::new(&memory);
    machine.set_reg(0, (memory_size - 4) as u32).unwrap();
    expect(&mut machine, false, memory_size);
    assert!(machine.step().is_err());
}

#[test]
fn exec_near_end_of_address_space() {
    let mut machine = Machine::new(&[]);
    machine.set_reg(0, 0xFFFF_FFFF).unwrap();
    assert!(machine.step().is_err());
}

#[test]
fn exec_near_end_of_memory() {
    // end-2: sub r1 <- r1 - r1
    // end+2:
    let memory_size = Machine::new(&[]).memory().len();
    let mut memory = vec![0; memory_size - 3];
    memory.extend([5, 1].iter());
    let mut machine = Machine::new(&memory);
    assert!(machine.step().is_err());
}

#[test]
fn load_near_end_of_memory() {
    // 0: load r1 <- [r1]
    // 3:
    let mut machine = Machine::new(&[2, 1, 1]);
    machine
        .set_reg(1, (machine.memory().len() - 2) as u32)
        .unwrap();
    assert!(machine.step().is_err());
    assert_eq!(machine.regs()[0], 3);
}

#[test]
fn load_near_end_of_address_space() {
    // 0: load r1 <- [r1]
    // 3:
    let mut machine = Machine::new(&[2, 1, 1]);
    machine.set_reg(1, 0xFFFF_FFFF).unwrap();
    assert!(machine.step().is_err());
}

#[test]
fn store_near_end_of_memory() {
    // 0: store [r1] <- r1
    // 3:
    let mut machine = Machine::new(&[3, 1, 1]);
    machine
        .set_reg(1, (machine.memory().len() - 2) as u32)
        .unwrap();
    assert!(machine.step().is_err());
    assert_eq!(machine.regs()[0], 3);
}

#[test]
fn store_near_end_of_address_space() {
    // 0: store [r1] <- r1
    // 3:
    let mut machine = Machine::new(&[3, 1, 1]);
    machine.set_reg(1, 0xFFFF_FFFF).unwrap();
    assert!(machine.step().is_err());
}

#[test]
fn sub_with_wraparound() {
    // 0: sub r1 <- r2 - r1
    // 4:
    let mut machine = Machine::new(&[5, 1, 2, 1]);
    machine.set_reg(1, 0xEEEE_EEEE).unwrap();
    expect(&mut machine, false, 4);
    assert_eq!(machine.regs()[1], !0xEEEEEEEE + 1);
}

#[test]
fn sub_with_wraparound_neg() {
    // 0: sub r1 <- r1 - r2
    // 3:
    let mut machine = Machine::new(&[5, 1, 1, 2]);
    machine.set_reg(1, 2147878597).unwrap();
    machine.set_reg(2, 34080773).unwrap();
    expect(&mut machine, false, 4);
    assert_eq!(machine.regs()[1], 2113797824);
}
