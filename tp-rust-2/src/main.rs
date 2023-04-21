use interpreter::{Machine, MachineError};
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), MachineError> {
    // Take a filename as argument on the command line
    let filename = std::env::args().nth(1).unwrap();

    // Read content to buffer
    let mut fs = File::open(&filename).unwrap();
    let mut buffer = Vec::new();
    fs.read_to_end(&mut buffer).unwrap();

    // Create a machine with this memory content
    let mut machine = Machine::new(&buffer);

    // Run the machine until the end
    machine.run()
}
