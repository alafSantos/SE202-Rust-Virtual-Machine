use interpreter::Machine;

#[test]
fn test_push_pop() {
    let mut machine = Machine::new(include_bytes!("push_pop.bin"));
    machine.run().unwrap();
    assert_eq!(26, machine.regs()[1]);
    assert_eq!(15, machine.regs()[2]);
}

#[test]
fn test_function() {
    let mut machine = Machine::new(include_bytes!("function.bin"));
    machine.run().unwrap();
    assert_eq!(42, machine.regs()[10]);
}

// Multiplication
#[test]
fn test_mult() {
    for left in &[10i32, -5, 15, -23, 0] {
        for right in &[1i32, 2, 3, 50] {
            // mult expect its arguments in r11 and r12 and the result will be in r11
            let mut machine = Machine::new(include_bytes!("multiply.bin"));
            machine.set_reg(11, *left as u32).unwrap();
            machine.set_reg(12, *right as u32).unwrap();
            machine.run().unwrap();
            assert_eq!(*left * *right, machine.regs()[11] as i32);
        }
    }
}

fn fact(n: u32) -> u32 {
    (2..=n).product()
}

// Factorial with in-register accumulator
#[test]
fn test_fact() {
    for i in 1..13 {
        let mut machine = Machine::new(include_bytes!("fact.bin"));
        machine.set_reg(10, i).unwrap();
        machine.run().unwrap();
        assert_eq!(fact(i), machine.regs()[11]);
    }
}

// Factorial with in-memory accumulator
#[test]
fn test_afact() {
    for i in 1..13 {
        let mut machine = Machine::new(include_bytes!("afact.bin"));
        machine.set_reg(10, i).unwrap();
        machine.run().unwrap();
        assert_eq!(fact(i), machine.regs()[11]);
    }
}

// Recursive factorial
#[test]
fn test_rfact() {
    for i in 1..13 {
        let mut machine = Machine::new(include_bytes!("rfact.bin"));
        machine.set_reg(10, i).unwrap();
        machine.run().unwrap();
        assert_eq!(fact(i), machine.regs()[11]);
    }
}

// Recursive factorial with final tail-recursive call to mult
#[test]
fn test_rfact_tr() {
    for i in 1..13 {
        let mut machine = Machine::new(include_bytes!("rfact_tr.bin"));
        machine.set_reg(10, i).unwrap();
        machine.run().unwrap();
        assert_eq!(fact(i), machine.regs()[11]);
    }
}

fn fibo(n: u32) -> u32 {
    (0..n).fold((0, 1), |(a, b), _| (b, a + b)).0
}

#[test]
fn test_fibo() {
    for i in 1..20 {
        let mut machine = Machine::new(include_bytes!("fibo.bin"));
        machine.set_reg(10, i).unwrap();
        machine.run().unwrap();
        assert_eq!(fibo(i), machine.regs()[11]);
    }
}
