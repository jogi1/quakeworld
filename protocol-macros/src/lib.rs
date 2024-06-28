use proc_macro::TokenStream;

mod datatypeboundcheck;
mod datatyperead;
mod datatyperead_new;
mod helpers;
mod parsemessage;

#[proc_macro_derive(DataTypeRead, attributes(datatyperead))]
pub fn datatype_read(input: TokenStream) -> TokenStream {
    datatyperead::datatyperead_derive(input)
}

#[proc_macro_derive(DataTypeBoundCheckDerive, attributes(check_bounds))]
pub fn datatypbe_boundcheck(input: TokenStream) -> TokenStream {
    datatypeboundcheck::datatype_bound_check_derive(input)
}

#[proc_macro_derive(ParseMessage)]
pub fn parsemessage(input: TokenStream) -> TokenStream {
    parsemessage::parsemessage_derive(input)
}
