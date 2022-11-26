use std::error::Error;

use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};


use crate::protocol::message::Message;
use crate::protocol::message::trace::ReadTrace;
#[cfg(feature = "ascii_strings")]
use crate::utils::ascii_converter::AsciiConverter;

fn fg_color_increase(color_in: u8) -> u8 {
    let min =  9;
    let max = 14;
    let mut color = color_in;
    if color < min {
        color = min;
    } else {
        color +=1;
        if color >= max {
            color = min ;
        }
    }
    color
}

fn bg_color_increase(color_in: u8) -> u8 {
    let min =  52;
    let max = 63;
    let mut color = color_in;
    if color < min {
        color = min;
    } else {
        color +=1;
        if color >= max {
            color = min ;
        }
    }
    color
}

fn fg_color_increase_mut(color_in: &mut u8) {
    let min =  9;
    let max = 14;
    let mut color = *color_in;
    if color < min {
        color = min;
    } else {
        color +=1;
        if color >= max {
            color = min ;
        }
    }
    *color_in = color;
}

fn bg_color_increase_mut(color_in: &mut u8)  {
    let min =  52;
    let max = 63;
    let mut color = *color_in;
    if color < min {
        color = min;
    } else {
        color +=1;
        if color >= max {
            color = min ;
        }
    }
    *color_in = color;
}

#[derive(Clone, Debug)]
struct RecursivePrintFlags {
    full: bool, // print full message
    indent: u32, // indentation level
    min_depth: u32, // minimum level for printing
    max_depth: u32, // maximum level for printing
    fg_color: u8, // foreground color
    bg_color: u8, // background color
}

fn print_recursive(message: &Message, traces: &ReadTrace, flags: &mut RecursivePrintFlags) /* full_message: bool, indent: u32, bg_color: u8, fg_color: u8, min_depth: u32, max_depth: u32)*/ -> Result<(), Box<dyn Error>>  {

    if flags.indent >= flags.max_depth {
        return Ok(());
    }

    if flags.indent >= flags.min_depth {
        for inner_trace in &traces.read {
            if inner_trace.read.is_empty() {
                continue;
            }
            let mut r_flags = flags.clone();
            r_flags.indent += 1;
            print_recursive(message, inner_trace, &mut r_flags)?;
        }
    }

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let trace = traces;
    let bg_color = bg_color_increase(flags.bg_color);
    let mut fg_color = fg_color_increase(flags.fg_color);
    let mut initial_color = flags.fg_color;

    let types: Vec<(usize, usize, u8)> = trace.read.clone().into_iter()
    .map(|read| {
         fg_color_increase_mut(&mut initial_color);
         (read.start, read.stop, initial_color)
    })
    .collect();

    let mut color_spec = ColorSpec::new();
    color_spec.set_fg(Some(Color::Ansi256(fg_color)));
    color_spec.set_bg(Some(Color::Black));
    stdout.set_color(&color_spec)?;

    let s = format!("{:width$}", "", width=flags.indent as usize);
    if trace.annotation.is_some() {
        let m = trace.annotation.clone();
        println!("{}{} {} {} {}", s, trace.function, m.unwrap(), trace.start, trace.stop);
    } else {
        println!("{}{} {} {}", s, trace.function, trace.start, trace.stop);
    }

    let indent = flags.indent +1;

    for trace in &traces.read {
        if trace.readahead {
            continue;
        }

        fg_color = fg_color_increase(fg_color);
        color_spec.set_fg(Some(Color::Ansi256(fg_color)));
        stdout.set_color(&color_spec)?;
        let s = format!("{:width$}", "", width=indent as usize);
        if trace.annotation.is_some() {
            let m = trace.annotation.clone();
            println!("{}{} {} {} {}", s, trace.function, m.unwrap(), trace.start, trace.stop);
        } else {
            println!("{}{} {} {}", s, trace.function, trace.start, trace.stop);
        }
    }

    let ascii_converter = AsciiConverter::new();
    let mut ascii_types: Vec<(u8, u8)> = vec![];
    if !types.is_empty() {
        let mut type_iter = types.iter();
        let mut current_type = type_iter.next().unwrap_or(&(0, 0, 0));
        let mut char_count = 0;
        for (mut count, b) in message.buffer[trace.start..trace.stop].iter().enumerate() {
            count += trace.start;
            if char_count == 0 {
                color_spec.set_fg(Some(Color::White));
                stdout.set_color(&color_spec)?;
                write!(&mut stdout, "0x{:0>8x} ", count - count % 16 )?;

                if count == trace.start {
                    for x in 0..trace.start%15 {
                        ascii_types.push((0, 32));
                        write!(&mut stdout, "   ")?;
                        if x == 8 {
                            write!(&mut stdout, " ")?;
                        }
                        char_count += 1;
                    }
                }
            }


            if char_count == 8  {
                write!(&mut stdout, " ")?;
            }

            let (_, stop, _) = current_type;
            if stop <= &count {
                current_type = if let Some(current_type) = type_iter.next() {
                    current_type
                } else {
                    current_type
                }; 
            }
            let (_, _, color) = current_type;
            color_spec.set_fg(Some(Color::Ansi256(*color)));
            stdout.set_color(&color_spec)?;
            write!(&mut stdout, "{:0>2x} ", b)?;
            ascii_types.push((*color, ascii_converter.convert_single(*b)));


            if char_count ==  15 {
                for (color, chr) in &ascii_types {
                    color_spec.set_fg(Some(Color::Ansi256(*color)));
                    stdout.set_color(&color_spec)?;
                    write!(&mut stdout, "{}", *chr as char)?;
                }
                ascii_types.clear();
                writeln!(&mut stdout, " ")?;
                char_count = 0;
            } else {
                char_count +=1;
            }
        }
    }

    color_spec.set_fg(Some(Color::White));
    stdout.set_color(&color_spec)?;
    writeln!(&mut stdout)?;

    for inner_trace in &trace.read {
        if inner_trace.read.is_empty() {
            continue;
        }
        let mut f = flags.clone();
        f.indent += 1;
        f.fg_color = fg_color;
        f.bg_color = bg_color;
        print_recursive(message, inner_trace, &mut f)?;
    }
    Ok(())
}

pub fn print_message_trace(message: &Message, full_message: bool, min_depth: u32, max_depth: u32) -> Result<(), Box<dyn Error>>  {
    let flags = RecursivePrintFlags {
        full: full_message,
        indent: 0, 
        min_depth,
        max_depth,
        fg_color: 0,
        bg_color: 0
    };
    for trace in &message.trace.read {
        print_recursive(message, trace, &mut flags.clone())?;
    }
    Ok(())
}

