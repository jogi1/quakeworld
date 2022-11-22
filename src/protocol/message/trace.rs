use serde::Serialize;

use crate::protocol::message::{ServerMessage, Packet, Message};
use crate::protocol::types::*;

#[cfg(feature = "trace")]
#[derive(Serialize, Clone, Debug, Default)]
pub struct ReadTrace {
    pub start: usize,
    pub stop: usize,
    pub readahead: bool,
    pub function: String,
    pub annotation: Option<String>,
    pub read: Vec<ReadTrace>,
    pub value: TraceValue,
}

#[cfg(feature = "trace")]
#[derive(Serialize, Clone, Debug, Default)]
pub enum TraceValue {
    #[default] None,
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
    I16(i16),
    I32(i32),
    F32(f32),
    VecU8(Vec<u8>),
    ServerMessage(ServerMessage),
    Packet(Packet),
    Stringbyte(StringByte),
}


#[cfg(feature = "trace")]
#[derive(Serialize, Clone, Default, Debug)]
pub struct MessageTrace {
    pub annotation: Option<String>,
    stack: Vec<ReadTrace>,
    pub read: Vec<ReadTrace>,
    pub enabled: bool,
}

pub(crate) trait ToTraceValue {
    fn to_tracevalue(&self) -> TraceValue;
}
impl ToTraceValue for StringByte {
    fn to_tracevalue(&self) -> TraceValue {
        TraceValue::Stringbyte(self.clone())
    }
}

impl ToTraceValue for Vec<u8> {
    fn to_tracevalue(&self) -> TraceValue {
        TraceValue::VecU8(self.clone())
    }
}

impl ToTraceValue for ServerMessage{
    fn to_tracevalue(&self) -> TraceValue {
        TraceValue::ServerMessage(self.clone())
    }
}

impl Message {
    #[cfg(feature = "trace")]
    pub fn read_trace_annotate(&mut self, annotation: &str) {
        if !self.trace.enabled {
            return;
        }
        self.trace.annotation = Some(annotation.to_string());
    }

    #[cfg(feature = "trace")]
    pub fn read_trace_start (&mut self, function: impl Into<String>, readahead: bool) {
        if !self.trace.enabled {
            return;
        }
        let function = function.into();
        let mut annotation = None;
        if self.trace.annotation.is_some() {
            annotation = self.trace.annotation.clone();
            self.trace.annotation = None;
        }
        let res = ReadTrace{
            function,
            start: self.position,
            readahead,
            stop: self.position,
            read: vec![],
            value: TraceValue::None,
            annotation,
        };
        self.trace.stack.push(res)
    }

    #[cfg(feature = "trace")]
    pub fn read_trace_stop(&mut self, value: TraceValue) {
        if !self.trace.enabled {
            return;
        }
        if let Some(mut trace) = self.trace.stack.pop() {
            trace.value = value;
            trace.stop = self.position;

            let len = self.trace.stack.len();
            if len > 0 {
                self.trace.stack[len-1].read.push(trace);
            } else {
                self.trace.read.push(trace);
            }
        }
    }
}


#[cfg(not(feature = "trace"))]
macro_rules! trace_start{
}

#[cfg(feature = "trace")]
macro_rules! trace_start{
    ($self:ident, $readahead:ident) => {
        (
                if $self.trace.enabled {
                    $self.read_trace_start(format!("{}", function!()), $readahead);
                }
         )
    }
}
pub(crate) use trace_start;

#[cfg(not(feature = "trace"))]
macro_rules! trace_stop {
}

/*
#[cfg(feature = "trace")]
macro_rules! to_value{
    ($type:ident, $value:ident) => {
        paste!{
            $type::($value)
        }
    }
}
pub(crate) use to_value;
*/

#[cfg(feature = "trace")]
macro_rules! trace_stop{
    ($self:ident, $value:ident, $valueType:ident) => {
        (
            paste!{
                if $self.trace.enabled {
                    $self.read_trace_stop(TraceValue::[< $valueType:upper >]($value));
                }
            }
         )
    };
    ($self:ident, $value:expr, $valueType:ident) => {
        (
            paste!{
                if $self.trace.enabled {
                    $self.read_trace_stop(TraceValue::[< $valueType:upper >]($value));
                }
            }
         )
    };
    ($self:ident, $value:expr) => {
        (
            paste!{
                if $self.trace.enabled {
                    $self.read_trace_stop($value.to_tracevalue());
                }
            }
         )
    }
}
pub(crate) use trace_stop;

#[cfg(not(feature = "trace"))]
macro_rules! trace_annotate {
}

#[cfg(feature = "trace")]
macro_rules! trace_annotate {
    ($self:ident, $value:literal) => {
        (
                if $self.trace.enabled {
                    $self.read_trace_annotate($value);
                }
         )
    }
}
pub(crate) use trace_annotate;
