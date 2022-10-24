use std::io::Read;
use std::error::Error;
use std::fs::File;
use std::env;
use quakeworld::mvd::Mvd;

// the most basic implementation of a mvd parser
fn parse_file(filename: String) -> Result<bool, Box<dyn Error>> {
    // read the file into a buffer
    let mut buffer = Box::new(Vec::new());
    let mut file = match File::open(filename) {
        Ok(file) => file,
        Err(err) => {
            return Err(Box::new(err))
        }
    };
    match file.read_to_end(&mut buffer) {
        Ok(size) => size,
        Err(err) => {
            return Err(Box::new(err))
        }
    };
    let mut mvd = Mvd::new(*buffer)?;
    while mvd.finished == false {
        let frame = mvd.parse_frame()?;
        println!("--- frame {}:{} ---", frame.frame, frame.time);
        for message in frame.messages {
            // will just print the message name
            println!("{}", message);
            // for more verbose output
            // println!("{:?}", message);
        }
    }
    return Ok(true)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("need to supply a demo");
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
