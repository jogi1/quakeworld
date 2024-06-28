use serde::Serialize;
use strum_macros::Display;

use crate::datatypes::common::DataType;

#[derive(Serialize, Clone, Debug, Default, Display)]
pub enum TraceValue {
    #[default]
    None,
    DateType(DataType),
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct TraceEntry {
    pub field_type: String,
    pub field_name: String,
    pub index: u64,
    pub index_stop: u64,
    pub value: TraceValue,
    pub traces: Vec<TraceEntry>,
    stack: Vec<TraceEntry>,
}

#[derive(Debug, Default, Serialize)]
pub struct Trace {
    pub trace: TraceEntry,
    annotation_prepend: Option<String>,
}

impl Trace {
    pub fn new() -> Self {
        Trace {
            trace: TraceEntry {
                ..Default::default()
            },
            annotation_prepend: None,
        }
    }
    pub fn start(
        &mut self,
        index: u64,
        field_type: impl Into<String>,
        field_name: impl Into<String>,
    ) {
        let field_type = field_type.into();
        let mut field_name = field_name.into();
        if let Some(s) = &self.annotation_prepend {
            field_name = s.to_string();
            self.annotation_prepend = None;
        }

        let ts = TraceEntry {
            field_type,
            field_name,
            index,
            index_stop: index,
            traces: vec![],
            stack: vec![],
            value: TraceValue::None,
        };
        self.trace.stack.push(ts);
    }

    pub fn annotate(&mut self, annotation_prepend: impl Into<String>) {
        let s = annotation_prepend.into();
        self.annotation_prepend = Some(s);
    }

    pub fn stop(&mut self, size: u64, value: DataType) {
        let mut size = size;
        size = size.saturating_sub(1);
        // pop the most recent trace
        if let Some(mut p) = self.trace.stack.pop() {
            //p.value = value;
            p.index_stop = size;
            p.value = TraceValue::DateType(value);
            // get the last trace on the stack
            if let Some(l) = self.trace.stack.last_mut() {
                l.index_stop = p.index_stop;
                // put that trace on the last element in the stack if it exists
                l.traces.push(p);
            } else {
                // if not the trace is finished
                self.trace.traces.push(p);
            }
        } else {
            panic!("ok?");
        }
    }
}

#[allow(unused)]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}
pub(crate) use function;

#[cfg(not(feature = "trace"))]
macro_rules! trace_start {}

#[cfg(feature = "trace")]
macro_rules! trace_start {
    ( $dr:ident, $name:expr, $field_name:expr) => {
        if let Some(trace) = &mut $dr.trace {
            trace.start($dr.cursor.position(), $name, $field_name);
        }
    };
    ( $dr:ident, $name:expr, $field_name:expr) => {
        if let Some(trace) = $dr.trace {
            trace.start($dr.cursor.position(), $name, $field_name);
        }
    };
    ( $dr:ident, $name:expr) => {
        if let Some(trace) = &mut $dr.trace {
            trace.start($dr.cursor.position(), $name, "");
        }
    };
    ( $dr:ident, $name:expr) => {
        if let Some(trace) = $dr.trace {
            trace.start($dr.cursor.position(), $name, "");
        }
    };
}
pub(crate) use trace_start;

#[cfg(not(feature = "trace"))]
macro_rules! trace_stop {}

#[cfg(feature = "trace")]
macro_rules! trace_stop {
    ( $dr:ident, $value:expr, $valueType:ident) => {
        paste! {
        if let Some(trace) = &mut $dr.trace {
        trace.stop($dr.cursor.position(), $value);
        }
        }
    };
    ($dr:expr, $value:expr) => {
        paste! {
        if let Some(trace) = &mut $dr.trace {
        trace.stop($dr.cursor.position(), $value);
        }
        }
    };
    ($self:expr) => {
        // if $self.trace.enabled && !$self.trace.locked {
        //     $self.read_trace_stop(TraceValue::None);
        // }
    };
}
pub(crate) use trace_stop;

#[cfg(not(feature = "trace"))]
macro_rules! trace_annotate {}

#[cfg(feature = "trace")]
macro_rules! trace_annotate {
    ($dr:ident, $name:expr) => {
        if let Some(trace) = &mut $dr.trace {
            trace.annotate($name);
        }
    };
    ($dr:ident, $name:expr) => {
        if let Some(trace) = $dr.trace {
            trace.annotate($name);
        }
    };
}
pub(crate) use trace_annotate;
