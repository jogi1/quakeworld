//! library for doing quakeworld things
#[macro_use]
extern crate lazy_static;

extern crate simple_error;
#[cfg(feature = "utils")]
pub mod utils;
#[cfg(feature = "protocol")]
pub mod protocol;
#[cfg(feature = "mvd")]
pub mod mvd;
#[cfg(feature = "state")]
pub mod state;

#[cfg(test)]
mod tests {
    use crate::utils::ascii_converter::AsciiConverter;
    #[test]
    fn ascii_converter() {
        let ascii_converter = AsciiConverter::new();
        let b = vec![177, 178, 179, 180];
        let s = ascii_converter.convert(&b);
        assert_eq!(s, "1234".to_string());
    }
}
