use std::{
    io::{self, Read},
    thread,
    time::Duration,
};

/// Pauses execution until any key is pressed.
pub fn pause() {
    println!("Press any key to continue...");

    // Create a buffer to hold one byte
    let mut buffer = [0; 1];

    // Read one byte from standard input to pause execution
    io::stdin().read_exact(&mut buffer).unwrap();
}

/// Sleeps for a specified number of milliseconds.
pub fn sleep(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}
