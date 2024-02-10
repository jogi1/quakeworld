use std::io::Read;
use std::error::Error;
use std::fs::File;
use std::env;

use quakeworld::pak::Pak;

fn parse_file(filename: String) -> Result<bool, Box<dyn Error>> {
    // read the file into a buffer
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(err) => {
            return Err(Box::new(err))
        }
    };
    let pak = Pak::parse(file)?;
    for file in &pak.files {
        println!("{} - {} {}", String::from_utf8_lossy(&file.name), file.position, file.size);
        let b = pak.get_data(file)?;
        println!("{}", b.len());
    }
    return Ok(true)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("need to supply a pak");
        return
    }
    let filename = &args[1];
    match parse_file(filename.to_string()) {
        Ok(..) => {
            println!("{} parsed.", filename);
        }
        Err(err) => {
            eprintln!("error in file {}: {}", filename, err);
        }
    }
}
