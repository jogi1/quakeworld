use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Ident, Type};

use crate::helpers::*;

pub fn datatyperead_derive_new1(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    let (_, tag_value) = check_tag_value_new(&ast.attrs, "datatyperead");
    let struct_implement_types: Vec<String> = match tag_value.get("types") {
        Some(types) => types
            .into_iter()
            .filter_map(|t| {
                if let Some(s) = t.span().source_text() {
                    Some(s.chars().filter(|&c| c != '"').collect())
                } else {
                    None
                }
            })
            .collect(),
        None => vec![],
    };
    println!("{:?}", struct_implement_types);
    panic!();

    // Extract the name of the struct
    let struct_name = &ast.ident;

    let struct_is_generic = match ast.generics.params.first() {
        Some(param) => {
            if let syn::GenericParam::Type(ty) = param {
                true
            } else {
                false
            }
        }
        None => false,
    };

    // Extract field names and types
    let fields: Vec<(TokenStream, TokenStream)> = if let syn::Data::Struct(data_struct) = &ast.data
    {
        if let syn::Fields::Named(fields) = &data_struct.fields {
            fields
                .named
                .iter()
                .map(|f| {
                    let ft = &f.ty;
                    let qt = quote! {#ft};
                    let mut generic_field_type = "".to_string();
                    // extract generic type
                    if let Type::Path(path) = ft {
                        for segment in &path.path.segments {
                            if let arguments = &segment.arguments {
                                if let syn::PathArguments::AngleBracketed(args) = arguments {
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

                    (read.into(),
                        quote! {
                        #field_identifier : #id,
                        }.into(),
                    )
                })
                .collect::<Vec<_>>()
        } else {
            panic!("DataTypeRead can only be derived for structs with named fields");
        }
    } else {
        panic!("DataTypeRead can only be derived for structs");
    };

    let struct_field_creation: Vec<_> = fields.iter().map(|(a, _)| a).collect();
    let struct_field_assignment: Vec<_> = fields.iter().map(|(_, b)| b).collect();
    println!(
        "struct_name: {:?}\nstruct_is_generic: {:?}\nstruct_field_generation: {:?}\nstruct_field_assignment: {:?}\n",
        struct_name, struct_is_generic, struct_field_creation, struct_field_assignment
    );
    quote! {}.into()
}

pub fn datatyperead_derive_new(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    let mut generic_specific_types = false;
    let (_, tag_value) = check_tag_value_new(&ast.attrs, "datatyperead");
    let struct_implement_types: Vec<String> = match tag_value.get("types") {
        Some(types) => types
            .into_iter()
            .filter_map(|t| {
                if let Some(s) = t.span().source_text() {
                    generic_specific_types = true;
                    Some(s.chars().filter(|&c| c != '"').collect())
                } else {
                    None
                }
            })
            .collect(),
        None => vec![],
    };

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
                    let qt = quote! {#ft};
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

    let datatype_leader = format!("{}", "DataType".to_string());
    let datatype_name = format!("{}", struct_name.to_string().to_uppercase());
    let mut datatype_prefix: String = "".into();
    let datatype_prefix = match tag_value.get("prefix") {
        Some(p) => {
            let prefix = if let syn::Lit::Str(p) = p {
                format!("{}", p.value())
            } else {
                panic!("datatypereader attribute prefix's value needs to be a String");
            };
            prefix
        }
        None => "".into(),
    };

    let datatype_generic: String = if we_have_a_generic {
        "(self.clone())".into()
    } else {
        "".into()
    };

    println!("HELLO its this one!: {}", struct_name);

    let datatype_pre = format!("{}::{}{}", datatype_leader, datatype_prefix, datatype_name,);
    let datatype = quote! {
        paste!{ [< #datatype_pre >][< #datatype_generic >]}
    };
    // Generate the implementation
    let read = quote! {
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
        }
    };
    let to_datatype_impl = if generic_specific_types {
        println!("we do a generic implementation");
        let todi: Vec<_> = struct_implement_types
            .into_iter()
            .map(|t| {
                let ty = format_ident!("{}", t);
                let leader = format_ident!("{}", datatype_leader);
                let prefix =
                    format_ident!("{}{}{}", datatype_prefix, datatype_name, t.to_uppercase());
                let ty_datatype = format!(
                    "{}::{}{}{}(s.clone())",
                    datatype_leader,
                    datatype_prefix,
                    datatype_name,
                    t.to_uppercase()
                );
                let datatype_type = t.to_uppercase();

                // #datatype_leader::#datatype_prefix#datatype_name#datatype_type(self)
                let s = quote! {
                    impl  DataTypeRead for #struct_name<#ty>  {
                        fn to_datatype(&self) -> DataType {
                            #leader::#prefix(self.clone())
                        }
                    }
                };
                println!("{}", s);
                s
            })
            .collect();
        quote! { #(#todi)*}
    } else {
        println!("we dont!: {}", datatype);
        quote! {
            impl #impl_generics DataTypeRead for #struct_name #ty_generics #where_clause {
                fn to_datatype(&self) -> DataType {
                    paste! {
                        #datatype
                    }
                }
            }
        }
    };

    let gen = quote! {
        #read
        #to_datatype_impl
    };

    // Return the generated implementation as a TokenStream
    gen.into()
}
