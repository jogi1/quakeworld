use serde::Serialize;
use paste::paste;
use strum_macros::Display;

use crate::protocol::message::{ServerMessage, Packet, Message};
use crate::protocol::types::*;
use crate::mvd::*;

#[cfg(feature = "trace")]
#[derive(Serialize, Clone, Debug, Default)]
pub struct ReadTrace {
    pub start: usize,
    pub stop: usize,
    pub readahead: bool,
    pub aborted: bool,
    pub function: String,
    pub annotation: Option<String>,
    pub read: Vec<ReadTrace>,
    pub value: TraceValue,
}

/*
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
    StringByte(StringByte),
    StringVector(StringVector),
}
*/


#[cfg(feature = "trace")]
#[derive(Serialize, Clone, Default, Debug)]
pub struct MessageTrace {
    pub annotation: Option<String>,
    pub stack: Vec<ReadTrace>,
    pub read: Vec<ReadTrace>,
    pub enabled: bool,
    pub locked: bool,
}

impl MessageTrace {
    pub fn clear(&mut self) {
        self.stack.clear();
        self.read.clear();
        self.annotation = None;
    }
}

pub(crate) trait ToTraceValue {
    fn to_tracevalue(&self) -> TraceValue;
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
            aborted: false,
        };
        self.trace.stack.push(res)
    }

    #[cfg(feature = "trace")]
    pub fn read_trace_abort(&mut self) {
        if !self.trace.enabled {
            return;
        }
        if let Some(mut trace) = self.trace.stack.pop() {
            trace.aborted = true;
            trace.stop = self.position;

            let len = self.trace.stack.len();
            if len > 0 {
                self.trace.stack[len-1].read.push(trace);
            } else {
                self.trace.read.push(trace);
            }
        }
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
    ($self:expr, $readahead:ident) => {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_start(format!("{}", function!()), $readahead);
        }
    };
    ($self:ident, $readahead:ident) => {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_start(format!("{}", function!()), $readahead);
        }
    }
}
pub(crate) use trace_start;

#[cfg(not(feature = "trace"))]
macro_rules! trace_stop {
}

#[cfg(feature = "trace")]
macro_rules! trace_stop{
    ($self:expr, $value:expr, $valueType:ident) => {
        paste! {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_stop(TraceValue::[< $valueType:upper >]($value));
        }
        }
    };
    ($self:expr, $value:expr) => {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_stop($value.to_tracevalue());
        }
    };
    ($self:expr) => {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_stop(TraceValue::None);
        }
    }
}
pub(crate) use trace_stop;

#[cfg(not(feature = "trace"))]
macro_rules! trace_abort {
}

#[cfg(feature = "trace")]
macro_rules! trace_abort {
    ($self:expr) => {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_abort();
        }
    }
}

pub(crate) use trace_abort;

#[cfg(not(feature = "trace"))]
macro_rules! trace_annotate {
}

#[cfg(feature = "trace")]
macro_rules! trace_annotate {
    ($self:expr, $value:literal) => {
        if $self.trace.enabled && !$self.trace.locked {
            $self.read_trace_annotate($value);
        }
    }
}
pub(crate) use trace_annotate;

#[cfg(feature = "trace")]
macro_rules! trace_lock {
    ($self:expr) => {
        if $self.trace.enabled {
            assert_eq!($self.trace.locked, false);
            $self.trace.locked = true;
        }
    }
}
pub(crate) use trace_lock;

macro_rules! trace_unlock {
    ($self:expr) => {
        if $self.trace.enabled {
            assert_eq!($self.trace.locked, true);
            $self.trace.locked = false;
        }
    }
}
pub(crate) use trace_unlock;

macro_rules! create_trace_enums{
    ($(($ty:ident, $en:ident)), *) => {
        paste! {
            #[derive(Debug, Default, PartialEq, PartialOrd, Display, Serialize, Clone)]
            pub enum TraceValue{
                #[default] None,
                VecU8(Vec<u8>),
                $(
                [< $en >]([< $ty >]),
                )*
            }

            $(
                impl ToTraceValue for $ty {
                    fn to_tracevalue(&self) -> TraceValue {
                        TraceValue::[< $en >](self.clone())
                    }
                }
                )*
        }
    };
}

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }}
}
pub(crate) use function;


impl ToTraceValue for Vec<u8> {
    fn to_tracevalue(&self) -> TraceValue {
        TraceValue::VecU8(self.clone())
    }
}

#[cfg(feature = "trace")]
create_trace_enums!(
    (u8, U8),
    (u16, U16),
    (u32, U32),
    (i8, I8),
    (i16, I16),
    (i32, I32),
    (f32, F32),
    (ServerMessage, ServerMessage),
    (Packet, Packet),
    (StringByte, StringByte),
    (DeltaUserCommand, DeltaUserCommand),
    (StringVector, StringVector),
    (MvdFrame, MvdFrame));

