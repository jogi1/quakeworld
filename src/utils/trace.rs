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


#[derive(Debug, Default, Clone)]
struct Printable {
    start: usize,
    stop: usize,
    color: u8,
    trace: ReadTrace,
}

fn generate_printable(trace: &ReadTrace, color: u8) -> Printable {
    Printable { start: trace.start, stop: trace.stop, color, trace: trace.clone()}
}

fn print_printable(message: &Message, trace: &ReadTrace, printable: Printable, subprints: Vec<Printable>, full_message: bool) -> Result<(), Box<dyn Error>> {
    let mut start = 0;
    let mut stop = 0;
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    if full_message {
        start = 0;
        stop = message.buffer.len();
    } else {
        start = trace.start;
        stop = trace.stop;
    }

    let mut color_spec = ColorSpec::new();
    color_spec.set_bg(Some(Color::Ansi256(printable.color)));
    stdout.set_color(&color_spec)?;
    println!("Function: {}", trace.function);

    let mut count = 0;
    let mut counter = 0;
    let mut start_counter = start;
    let print_ = Printable{
        ..Default::default()
    };
    let mut sub_printable =  &print_;
    let mut sub_print_count = 0;
    let mut sub_print_use = false;
    if subprints.len() > sub_print_count {
        sub_printable = &subprints[sub_print_count];
        sub_print_use = true;
    }

    for b in &message.buffer[start..stop] {
        if count == 0 {
            stdout.set_color(ColorSpec::new().set_bg(Some(Color::Black)))?;
            write!(&mut stdout, "0x{:0>8x} ", counter)?;
        }

        if sub_print_use && sub_printable.stop <= start_counter {
            sub_print_count+=1;
            if subprints.len() > sub_print_count {
                sub_printable = &subprints[sub_print_count];
            } else {
                sub_print_use = false;
            }
        }
        let mut color_spec = ColorSpec::new();
        if sub_print_use {
            color_spec.set_fg(Some(Color::Ansi256(sub_printable.color)));
            color_spec.set_bg(Some(Color::Black));
        } else {
            color_spec.set_fg(Some(Color::White));
            color_spec.set_bg(Some(Color::Black));
        }
        if start_counter < trace.start || start_counter >= trace.stop {
            color_spec.set_bg(Some(Color::Black));
        } else {
            color_spec.set_bg(Some(Color::Ansi256(printable.color)));
        }

        stdout.set_color(&color_spec)?;
        if count == 8 {
            print!("  ");
        }

        write!(&mut stdout, "{:0>2x}", b)?;

        start_counter += 1;
        count += 1;
        counter += 1;
        if count > 15 {
            count = 0;
            color_spec.set_fg(Some(Color::White));
            color_spec.set_bg(Some(Color::White));
            stdout.set_color(&color_spec)?;
            write!(&mut stdout, " ")?;
        } else if counter == (stop - start) {
            color_spec.set_fg(Some(Color::White));
            color_spec.set_bg(Some(Color::White));
            stdout.set_color(&color_spec)?;
            writeln!(&mut stdout)?;
            break;
        }else {
            write!(&mut stdout, " ")?;
        }
    }
    writeln!(&mut stdout)?;

    for subtrace in subprints {
        let mut color_spec = ColorSpec::new();
        color_spec.set_fg(Some(Color::Ansi256(subtrace.color)));
        stdout.set_color(&color_spec)?;
        write!(&mut stdout, "{} {}\n\t{:?}\n",
               subtrace.trace.function,
               subtrace.trace.annotation.unwrap_or_default(),
               subtrace.trace.value,
               )?;
    }
    Ok(())
}

pub fn print_message_trace(message: &Message) -> Result<(), Box<dyn Error>>  {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut color: u8 = 0;
    color = bg_color_increase(color);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    for trace in &message.trace.read {
        if trace.readahead {
            continue;
        }
        let p = generate_printable(trace, color);
        let mut subtraces: Vec<Printable> = vec![];
        let mut c = color + 40;
        c = fg_color_increase(c);
        for subtrace in &trace.read {
            let p = generate_printable(subtrace, c);
            subtraces.push(p);
            c = fg_color_increase(c);
        }
        print_printable(message, trace, p, subtraces, true)?;

        for next_trace in &trace.read {
            if next_trace.readahead {
                continue;
            }
            if next_trace.read.is_empty() {
                continue;
            }

            let p = generate_printable(next_trace, color);
            let mut subtraces: Vec<Printable> = vec![];
            let mut c = color + 40;
            c = fg_color_increase(c);
            for subtrace in &next_trace.read {
                let p = generate_printable(subtrace, c);
                subtraces.push(p);
                c = fg_color_increase(c);
            }
            print_printable(message, next_trace, p, subtraces, false)?;

        }
    }
    Ok(())
}

pub fn print_message_trace_old(message: &Message) -> Result<(), Box<dyn Error>>  {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    #[cfg(feature = "ascii_strings")]
    let converter = AsciiConverter::new();
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    writeln!(&mut stdout, "Message:")?;
    let mut color: u8 = 21;
    let mut count = 0;
    let mut position = 0;
    let mut line_vec: Vec<u8> = vec![];

    let mut trace_message = 0;
    let mut trace = &message.trace.read[trace_message];
    println!("{}", message.trace.read.len());
    println!("{} {}", trace.readahead, trace.function);
    while trace.readahead && trace_message < message.trace.read.len() {
        trace_message += 1;
        trace = &message.trace.read[trace_message];
    }
    for b in &*message.buffer {
        if count % 15 == 0 && count == 0 {
            write!(&mut stdout, "0x{:0>8x} ", count)?;
        }
        if trace.start == position {
        }
        if count == 0 {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Ansi256(color))))?;
        }

        if count  == 8 {
            write!(&mut stdout, " ")?;
        }

        if position - trace.start >= trace.stop - trace.start {
            color +=1;
            if color >= 230 {
                color = 21;
            }
            stdout.set_color(ColorSpec::new().set_bg(Some(Color::Ansi256(color))))?;
            if trace_message < message.trace.read.len() -1 {
                trace_message += 1;
                trace = &message.trace.read[trace_message];
            }
            while trace.readahead && trace_message < message.trace.read.len() {
                trace_message += 1;
                trace = &message.trace.read[trace_message];
            }
        }

        write!(&mut stdout, "{:0>2x} ", b)?;
        line_vec.push(*b);

        if count % 15 == 0 && count > 0 {

            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
            #[cfg(feature = "ascii_strings")]
            {
                let s = converter.convert_to_stringbyte(line_vec.clone());
                write!(&mut stdout, "{}", s.string)?;
            }
            writeln!(&mut stdout)?;
            line_vec.clear();
            count = -1;
        }
        count += 1;
        position += 1;
    }
    for i in count..16 {
        if i == 8 {
            write!(&mut stdout, " ")?;
        }
        
        write!(&mut stdout, "   ")?;
        if i == 15 {
            #[cfg(feature = "ascii_strings")]
            {
                let s = converter.convert_to_stringbyte(line_vec.clone());
                write!(&mut stdout, "{}", s.string)?;
            }
            writeln!(&mut stdout)?;
            line_vec.clear();
        }
    }

    stdout.set_color(ColorSpec::new().set_bg(Some(Color::Ansi256(0))))?;
    let mut color = 21;
    for trace in &message.trace.read {
        if trace.readahead {
            continue;
        }
        stdout.set_color(ColorSpec::new().set_bg(Some(Color::Ansi256(0))))?;
        writeln!(&mut stdout, "{:?} {:?} {} {}", trace.start, trace.stop, trace.readahead, trace.function)?;
            color +=1;
            if color >= 230 {
                color = 21;
            }
    }
    Ok(())
}

