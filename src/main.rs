use hex;
use std::fs::File;
use std::io::Read;
use std::{env, io};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let file_path = &args[1];
        let mut file = File::open(file_path)?;
        let mut header = [0; 16];
        let size = file.read(&mut header[..])?;
        assert!(size == 16);
        let hex_output = hex::encode_upper(header);
        println!("{hex_output}");
    }
    Ok(())
}
