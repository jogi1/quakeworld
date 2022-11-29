use std::error::Error;

use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};


use crate::protocol::message::Message;
use crate::protocol::message::trace::{ReadTrace, TraceValue};
#[cfg(feature = "ascii_strings")]
use crate::utils::ascii_converter::AsciiConverter;


#[derive(Clone, Debug)]
pub struct ColorRotation {
    colors: Vec<u8>,
    index: usize,
}

impl ColorRotation {
    pub fn new() -> ColorRotation {
        ColorRotation {
            colors: vec![9, 10, 11, 12, 13, 14], index: 0 }
    }

    pub fn new_with_colors(colors: impl Into<Vec<u8>>) -> ColorRotation {
        let colors = colors.into();
        ColorRotation {
            colors, index: 0 }
    }

    pub fn get_color(&mut self) -> u8 {
        self.colors[self.index]
    }

    pub fn next_color(&mut self) -> u8 {
        self.index +=1;
        if self.index >= self.colors.len() {
            self.index = 0;
        }
        self.colors[self.index]
    }
}

impl Default for ColorRotation {
    fn default() -> Self {
        Self::new()
    }
}

fn print_hex(message: &Message, trace: &ReadTrace, types: Vec<(usize, usize, u8)>, full: bool, colorized: bool) -> Result<(), Box<dyn Error>>{
    if types.is_empty() {
        return Ok(());
    }

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let amount_of_lines =  trace.stop/15 - trace.start/15 + 1;
    let mut hex_start = trace.start - trace.start % 16;

    let mut cs_normal = ColorSpec::new();
    cs_normal.set_fg(Some(Color::White));
    cs_normal.set_bg(Some(Color::Black));

    let mut cs_trace = ColorSpec::new();
    cs_normal.set_fg(Some(Color::White));
    cs_normal.set_bg(Some(Color::Black));

    let ascii_converter = AsciiConverter::new();

    let mut type_iter = types.into_iter();

    let mut current_trace: (usize, usize, u8) = match type_iter.next() {
        Some(type_val) => { type_val },
        None => { (0,0,0)},
    };
    for _ in 0..amount_of_lines {
        if colorized {
            stdout.set_color(&cs_normal)?;
        }
        write!(&mut stdout, "0x{:0>8x} ", hex_start)?;

        let mut ascii_types: Vec<(u8, u8)> = vec![];
        for char_count in 0..16 {
            let trace_index = hex_start + char_count;
            if colorized && trace_index >= trace.start && trace_index < trace.stop{
                let (_, stop, _) = current_trace;
                if stop - 1 < trace_index {
                    current_trace = match type_iter.next() {
                        Some(type_val) => { type_val },
                        None => {
                            cs_trace.set_fg(Some(Color::White));
                            stdout.set_color(&cs_trace)?;
                            (0,0,0)
                        },
                    };
                }
                let (_, _, color) = current_trace;
                cs_trace.set_fg(Some(Color::Ansi256(color)));
                stdout.set_color(&cs_trace)?;
                let b = message.buffer[trace_index];
                write!(&mut stdout, "{:0>2x} ", b)?;
                ascii_types.push((color, b));
            } else {
                if colorized {
                    stdout.set_color(&cs_normal)?;
                }
                if full && trace_index < message.buffer.len() { 
                    let b = message.buffer[trace_index];
                    ascii_types.push((0, b));
                    write!(&mut stdout, "{:0>2x} ", b)?;
                } else {
                    ascii_types.push((0, b' '));
                    write!(&mut stdout, "   ")?;
                }
            }
        }
        write!(&mut stdout, " | ")?;
        for (color, c) in ascii_types {
            if colorized {
                if color == 0 {
                    stdout.set_color(&cs_normal)?;
                } else {
                    cs_trace.set_fg(Some(Color::Ansi256(color)));
                    stdout.set_color(&cs_trace)?;
                }
            }
            write!(&mut stdout, "{}", ascii_converter.convert_single(c) as char)?;
        }
        writeln!(&mut stdout)?;
        hex_start += 16;
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct RecursivePrintFlags {
    full: bool, // print full message
    indent: i32, // indentation level
    min_depth: i32, // minimum level for printing
    max_depth: i32, // maximum level for printing
    fg_color: ColorRotation, // foreground color
    colorized: bool,
}

fn print_recursive(message: &Message, traces: &ReadTrace, flags: &mut RecursivePrintFlags) -> Result<(), Box<dyn Error>>  {

    if flags.max_depth != -1 && flags.indent >= flags.max_depth {
        return Ok(());
    }
    
    if traces.readahead {
        return Ok(());
    }

    if flags.indent < flags.min_depth {
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
    let mut fg_color = flags.fg_color.clone();
    let mut trace_color = fg_color.clone();
    let mut sub_color = fg_color.clone();
    let mut initial_color = fg_color.clone();


    let types: Vec<(usize, usize, u8)> = trace.read.clone().into_iter()
    .map(|read| {
        let color = trace_color.next_color();
        (read.start, read.stop, color)
    })
    .collect();

    let mut color_spec = ColorSpec::new();
    if flags.colorized {
        color_spec.set_fg(Some(Color::Ansi256(fg_color.get_color())));
        color_spec.set_bg(Some(Color::Black));
        stdout.set_color(&color_spec)?;
    }

    let mut indent = flags.indent;
    print!("{:width$}{function}", "", width=indent as usize, function=trace.function);
    if trace.annotation.is_some() {
        let m = trace.annotation.clone();
        print!(" {}", m.unwrap());
    } 
    print!(" start({}) stop({})", trace.start, trace.stop);
    if trace.aborted {
        print!(" aborted");
    }
    if trace.value  != TraceValue::None {
        match trace.value {
            TraceValue::Packet(..)=> {},
            _ => { print!(" {:?}", trace.value);},
        };
    }
    println!();

    if flags.indent >= flags.min_depth {
        print_hex(message, trace, vec![(trace.start, trace.stop, sub_color.get_color())], flags.full, flags.colorized)?;
    }

    indent += 1;

    for trace in &traces.read {
        if trace.readahead {
            fg_color.next_color();
            continue;
        }

        color_spec.set_fg(Some(Color::Ansi256(fg_color.next_color())));
        if flags.colorized {
            stdout.set_color(&color_spec)?;
        }
        print!("{:width$}{function}", "", width=indent as usize, function=trace.function);
        if trace.annotation.is_some() {
            let m = trace.annotation.clone();
            print!(" {}", m.unwrap());
        } 
        print!(" start({}) stop({})", trace.start, trace.stop);
        if trace.aborted {
            print!(" aborted");
        }
        println!();
    }

    print_hex(message, trace, types, flags.full, flags.colorized)?;
    for inner_trace in &trace.read {
        if inner_trace.read.is_empty() {
            initial_color.next_color();
            continue;
        }
        let mut f = flags.clone();
        f.indent += 1;
        initial_color.next_color();
        f.fg_color = initial_color.clone();
        print_recursive(message, inner_trace, &mut f)?;
    }
    Ok(())
}

pub fn print_message_trace(message: &Message, full_message: bool, min_depth: i32, max_depth: i32, colorized: bool) -> Result<(), Box<dyn Error>>  {
    let flags = RecursivePrintFlags {
        full: full_message,
        indent: 0, 
        min_depth,
        max_depth,
        fg_color: ColorRotation::new(),
        colorized,
    };

    for trace in &message.trace.read {
        print_recursive(message, trace, &mut flags.clone())?;
    }
    Ok(())
}


pub fn print_message_trace_with_colors(message: &Message, full_message: bool, min_depth: i32, max_depth: i32, colorized: bool, colors: impl Into<Vec<u8>>) -> Result<(), Box<dyn Error>>  {
    let flags = RecursivePrintFlags {
        full: full_message,
        indent: 0, 
        min_depth,
        max_depth,
        fg_color: ColorRotation::new_with_colors(colors),
        colorized,
    };

    for trace in &message.trace.read {
        print_recursive(message, trace, &mut flags.clone())?;
    }
    Ok(())
}

