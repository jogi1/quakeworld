use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Ident, Type};

use crate::helpers::*;

pub fn datatyperead_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input_clone = input.clone();
    let ast = parse_macro_input!(input as DeriveInput);
    if ast.ident.to_string() == "BoundingBox".to_string() {
        return crate::datatyperead_new::datatyperead_derive_new(input_clone);
    }

    // Extract the name of the struct
    let struct_name = &ast.ident;

    // if struct_name.to_string() == "BoundingBox" {
    //     println!("start: +++++++++++++++++++++");
    let (have_tag, tag_value) = check_tag_value_new(&ast.attrs, "datatyperead");
    if have_tag {
        println!("tag_new_value: {:?}", tag_value);
    }
    //     println!("stop: ----------------------");
    // }

    let mut we_have_a_generic = false;

    match ast.generics.params.first() {
        Some(param) => {
            if let syn::GenericParam::Type(ty) = param {
                we_have_a_generic = true;
            } else {
            }
        }
        None => {}
    };
    // Extract field names and types
    let fields = if let syn::Data::Struct(data_struct) = &ast.data {
        if let syn::Fields::Named(fields) = &data_struct.fields {
            fields
                .named
                .iter()
                .map(|f| {
                    let ft = &f.ty;
                    // let qt = quote! {#ft};
                    let mut generic_field_type = "".to_string();
                    // extract generic type
                    if let Type::Path(path) = ft {
                        for segment in &path.path.segments {
                            match &segment.arguments {
                                syn::PathArguments::AngleBracketed(args) => {
                                    for arg in &args.args {
                                        if let syn::GenericArgument::Type(t) = arg {
                                            if let syn::Type::Path(p) = t {
                                                for seg in &p.path.segments {
                                                    if let Some(s) = seg.ident.span().source_text() {
                                                        generic_field_type.push_str(&s as &str);
                                                    }
                                                }
                                            }

                                        }
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    // let generics = &f.generics;
                    // let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
                    let mut size: usize = 0;
                    let mut do_size_env = false;
                    let mut size_env: String = "".to_string();
                    // check if we have datatype read attributes
                    // "string" signifies that the field should be cast to a GENERICSTRING
                    let (_, tag_value) = check_tag_value(&f.attrs, "datatyperead");
                    let _ = match tag_value.get("string") {
                        Some(_) => true,
                        None => false,
                    };
                    // "size" has multiple options:
                    //  - if its an Int the vector will be read to the specified size
                    //  - if its a Str vector size will be pulled from the datareader environment
                    let tv = tag_value.get("size");
                    let do_size = if let Some(v) = tv {
                        match  v {
                            syn::Lit::Int(value) => {
                                if let Ok(parsed_value) = value.base10_parse::<usize>() {
                                    size = parsed_value;
                                } else {
                                    panic!(
                                    "datatypereader attribute size's value couldnt be converted to usize"
                                );
                                }
                            },
                            syn::Lit::Str(value) => {
                                do_size_env = true;
                                size_env = value.value();
                            },
                            _ =>  size = 0,
                        }
                        true
                    } else {
                        false
                    };
                    let field_identifier = &f.ident;
                    let ty = &f.ty;

                    let qi = quote! {#field_identifier};
                    let qt = quote! {#ty};
                    let vi = format!("{}", qi);
                    let id = format_ident!("{}", format!("{}_{}", qi.to_string(), qt.to_string()).replace(&[ '<', '>', ' ' ][..], "_"));

                    let read = if do_size {
                        if do_size_env {
                        quote! {
                            trace_annotate!(datareader, #vi);
                            let size: usize = match datareader.get_env(#size_env) {
                                Some(value) => {
                                    value.into()
                                }
                                None => {panic!("datareader environtment \"{}\" not set", #size_env);}
                            };
                            let mut #id: #ty = Vec::with_capacity(size);
                            datareader.read_exact_generic(&mut #id)?;
                        }
                        } else {
                            quote! {
                                trace_annotate!(datareader, #vi);
                                let mut #id: #ty = Vec::with_capacity(#size);
                                datareader.read_exact_string(&mut #id)?;
                            }
                        }
                    } else {
                        quote! {
                            trace_annotate!(datareader, #vi);
                            let #id = <#ty as DataTypeRead>::read(datareader)?;
                        }
                    };

                    (read,
                        quote! {
                        #field_identifier : #id,
                        },
                    )
                })
                .collect::<Vec<_>>()
        } else {
            panic!("DataTypeRead can only be derived for structs with named fields");
        }
    } else {
        panic!("DataTypeRead can only be derived for structs");
    };

    // Extract generic parameters
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let field_creation: Vec<_> = fields.iter().map(|(a, _)| a).collect();
    let field_assignment: Vec<_> = fields.iter().map(|(_, b)| b).collect();

    let (_, tag_value) = check_tag_value(&ast.attrs, "datatyperead");
    let generic_dt = if we_have_a_generic { "GENERIC" } else { "" };

    let mut datatype = match tag_value.get("prefix") {
        Some(p) => {
            let prefix = if let syn::Lit::Str(p) = p {
                p.value()
            } else {
                panic!("datatypereader attribute prefix's value needs to be a String");
            };
            let i = format_ident!(
                "{}{}{}",
                prefix.to_uppercase(),
                struct_name.to_string().to_uppercase(),
                generic_dt
            );
            quote! { DataType::#i}
        }
        None => {
            let i = format_ident!("{}{}", struct_name.to_string().to_uppercase(), generic_dt);

            quote! { DataType::#i}
        }
    };
    if !we_have_a_generic {
        datatype = quote! { #datatype(self.clone()) };
    }
    println!("HELLO!: {}", datatype);

    // Generate the implementation
    let gen = quote! {
        impl #impl_generics DataTypeRead for #struct_name #ty_generics #where_clause {
            fn read(datareader: &mut DataTypeReader) -> Result<Self, DataTypeReaderError> {
                trace_start!(datareader, stringify!( #struct_name));
                #(#field_creation)*

                let s = Self {
                    #(#field_assignment)*
                };
                let d = s.clone().to_datatype();
                trace_stop!(datareader, d, #struct_name);
                Ok(s)
            }
            fn to_datatype(&self) -> DataType {
                paste! {
                    #datatype
                }
            }
        }
    };

    // Return the generated implementation as a TokenStream
    gen.into()
}
