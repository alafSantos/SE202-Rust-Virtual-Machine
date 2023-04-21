use interpreter::Machine;

fn create_machine(code: &[u8]) -> (Machine, Vec<u8>) {
    let mut machine = Machine::new(code);
    for &(reg, value) in &[(1, 10), (2, 25), (3, 0x1234ABCD), (4, 0), (5, 65)] {
        machine.set_reg(reg, value).unwrap();
    }
    let mut out = vec![];
    machine.step_on(&mut out).unwrap();
    (machine, out)
}

#[test]
fn test_assignment() {
    // Test that the examples given in the assignment text
    // behave as expected.

    // move_if
    let (m, _) = create_machine(&[1, 1, 2, 3]);
    assert_eq!(25, m.regs()[1]);
    let (m, _) = create_machine(&[1, 1, 2, 4]);
    assert_eq!(10, m.regs()[1]);

    // store
    let (m, _) = create_machine(&[2, 2, 3]);
    assert_eq!(&[0xcd, 0xab, 0x34, 0x12], &m.memory()[25..29]);

    // load
    let mut mem = vec![3, 1, 2];
    mem.extend(std::iter::repeat(0).take(22));
    mem.extend(&[0xcd, 0xab, 0x34, 0x12]);
    let (m, _) = create_machine(&mem);
    assert_eq!(0x1234abcd, m.regs()[1]);

    // loadimm
    let (m, _) = create_machine(&[4, 1, 0x11, 0x70]);
    assert_eq!(0x7011, m.regs()[1]);
    let (m, _) = create_machine(&[4, 1, 0x11, 0xd0]);
    assert_eq!(0xffffd011, m.regs()[1]);

    // sub
    let (m, _) = create_machine(&[5, 10, 2, 1]);
    assert_eq!(15, m.regs()[10]);
    let (m, _) = create_machine(&[5, 10, 4, 1]);
    assert_eq!(-10, m.regs()[10] as i32);

    // out
    let (_, out) = create_machine(&[6, 5]);
    assert_eq!("A".as_bytes(), &out[..]);
    let (_, out) = create_machine(&[6, 3]);
    assert_eq!("Í".as_bytes(), &out[..]);

    // exit
    let (m, _) = create_machine(&[7]);
    assert_eq!(1, m.regs()[0]);

    // out number
    let (_, out) = create_machine(&[8, 5]);
    assert_eq!(&b"65"[..], &out[..]);
    let (_, out) = create_machine(&[8, 3]);
    assert_eq!(&b"305441741"[..], &out[..]);
}
