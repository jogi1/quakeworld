use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{Data, DataStruct, Fields};

#[proc_macro_derive(ParseMessage)]
pub fn parse_message_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_parsemessage_macro(&ast)
}

fn impl_parsemessage_macro(ast: &syn::DeriveInput) -> TokenStream {
    let fields = match &ast.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    let field_name = fields.iter().map(|field| &field.ident);
    let field_name2 = fields.iter().map(|field| &field.ident);
    let field_function = fields.iter().map(|field| {
        let ft = &field.ty;
        let q = quote! { #ft };
        format_ident!("read_{}", q.to_string().to_lowercase())
    });

    let struct_name = &ast.ident;

    let gen = quote! {
        impl #struct_name {
            fn read(message: &mut Message) -> Result<ServerMessage, MessageError>
            {
                #(
                    let #field_name = message.#field_function(false)?;
                 )*

                    Ok(
                        ServerMessage::#struct_name(
                            #struct_name{
                                #(
                                    #field_name2,
                                    )*
                            })
                      )
            }
        }
    };
    gen.into()
}

