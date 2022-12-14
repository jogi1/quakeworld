use std::io::Read;
use std::error::Error;
use std::fs::File;
use std::env;
use quakeworld::mvd::Mvd;
use quakeworld::state::State;
use quakeworld::protocol::types::ServerMessage;

#[cfg(feature = "trace")]
use quakeworld::utils::trace::*;

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
    let mut mvd = Mvd::new(*buffer,
#[cfg(feature = "ascii_strings")]
                           None,
#[cfg(feature = "trace")]
                           true,
                           )?;

    let mut state = State::new();
    while mvd.finished == false {
        //let frame = mvd.parse_frame()?;
        let frame = match  mvd.parse_frame() {
            Ok(v) => {
                v
            },
            Err(e) => {
                #[cfg(feature = "trace")]
                print_message_trace(&mvd.message, false, 0, 2, false)?;
                return Err(Box::new(e));
            },
        };

        // frame count and demo time
        //println!("--- frame {}:{} ---", frame.frame, frame.time);
        // if you need to keep the last state
        // let old_state = state.clone();
        state.apply_messages_mvd(&frame.messages, frame.last);
        // get the players when intermission is reached
        for message in frame.messages {
            match message {
                ServerMessage::Intermission(_) => {
                    println!("{:#?}", state.players);
                }
                _ => {}
            }
        }
        #[cfg(feature = "trace")]
        if false { // if you want to print a trace of each read frame
            print_message_trace(&mvd.message, false, 0, 2, false)?;
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
