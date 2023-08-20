#[proc_macro_derive(GeneratePostgresqlCrud)]
pub fn generate_postgresql_crud(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro_helpers::panic_location::panic_location();
    let proc_macro_name = "GeneratePostgresqlCrud";
    let ast: syn::DeriveInput = syn::parse(input).unwrap_or_else(|_| {
        panic!(
            "{proc_macro_name} {}",
            proc_macro_helpers::global_variables::hardcode::AST_PARSE_FAILED
        )
    });
    let ident = &ast.ident;
    let proc_macro_name_ident_stringified = format!("{proc_macro_name} {ident}");
    let data_struct = if let syn::Data::Struct(data_struct) = ast.data {
        data_struct
    } else {
        panic!("does not work on structs!");
    };
    let fields_named = if let syn::Fields::Named(fields_named) = data_struct.fields {
        fields_named.named
    } else {
        panic!("{proc_macro_name_ident_stringified} supports only syn::Fields::Named");
    };
    let struct_options_ident_token_stream = {
        let struct_options_ident_stringified = format!("{ident}Options");
        struct_options_ident_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {struct_options_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let fields_options = fields_named.iter().map(|field| {
        let field_vis = &field.vis;
        let field_ident = &field.ident;
        let field_type_path = &field.ty;
        quote::quote! {
            #[serde(skip_serializing_if = "Option::is_none")]
            #field_vis #field_ident: Option<#field_type_path>
        }
    });
    let struct_options_token_stream = quote::quote! {
        #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
        pub struct #struct_options_ident_token_stream {
            #(#fields_options),*
        }
    };
    let from_ident_for_ident_options_token_stream = {
        let ident_option_variants_token_stream = {
            fields_named.iter()
                .map(|field| {
                    let field_ident = field.ident
                        .clone()
                        .unwrap_or_else(|| {
                            panic!("GeneratePostgresqlCrud field.ident is None")
                        });
                    quote::quote! {
                        #field_ident: Some(value.#field_ident)
                    }
                })
        };
        quote::quote! {
            impl std::convert::From<#ident> for #struct_options_ident_token_stream {
                fn from(value: #ident) -> Self {
                    #struct_options_ident_token_stream {                        
                        #(#ident_option_variants_token_stream),*
                    }
                }
            }
        }
    };
    let column_variants = {
        let fields_named_enumerated = fields_named
            .iter()
            .enumerate()
            .map(|(index, field)| (index, field))
            .collect::<Vec<(usize, &syn::Field)>>();
        let fields_named_clone_stringified = fields_named.iter().collect::<Vec<&syn::Field>>();
        let mut veced_vec = fields_named_clone_stringified
            .iter()
            .map(|field| vec![(*field).clone()])
            .collect();
        column_names_factorial(
            fields_named_enumerated,
            fields_named_clone_stringified,
            &mut veced_vec,
        )
    };
    let structs_variants_token_stream = {
        column_variants
            .iter()
            .map(|variant_columns| {
                let struct_name_token_stream = {
                    let mut struct_name_stringified = format!("{ident}");
                    variant_columns.iter().for_each(|variant_column| {
                        use convert_case::Casing;
                        let column_title_cased = variant_column.ident
                            .clone()
                            .unwrap_or_else(|| {
                                panic!("GeneratePostgresqlCrud field.ident is None")
                            })
                            .to_string().to_case(convert_case::Case::Title);
                        struct_name_stringified.push_str(&column_title_cased);
                    });
                    struct_name_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {struct_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let genereted_fields = variant_columns.iter().map(|variant_column|{
                    let variant_column_ident = variant_column.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("GeneratePostgresqlCrud field.ident is None")
                        });
                    let variant_column_type = &variant_column.ty;
                    quote::quote! {
                        pub #variant_column_ident: #variant_column_type,
                    }
                });
                quote::quote! {
                    pub struct #struct_name_token_stream {
                        #(#genereted_fields)*
                    }
                }
            })
            .collect::<Vec<proc_macro2::TokenStream>>()
    };
    let structs_variants_impl_from_token_stream = {
        column_variants
            .iter()
            .map(|variant_columns| {
                let struct_name_token_stream = {
                    let mut struct_name_stringified = format!("{ident}");
                    variant_columns.iter().for_each(|variant_column| {
                        use convert_case::Casing;
                        let column_title_cased = variant_column.ident
                            .clone()
                            .unwrap_or_else(|| {
                                panic!("GeneratePostgresqlCrud field.ident is None")
                            })
                            .to_string().to_case(convert_case::Case::Title);
                        struct_name_stringified.push_str(&column_title_cased);
                    });
                    struct_name_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {struct_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let value_token_stream = quote::quote! {value};
                let fields_options_token_stream = fields_named.iter().map(|field|{
                    let field_ident = field.ident.clone().unwrap_or_else(|| {
                        panic!("GeneratePostgresqlCrud field.ident is None")
                    });
                    match variant_columns.contains(field) {
                        true => {
                            quote::quote! {
                                #field_ident: Some(#value_token_stream.#field_ident)
                            }
                        },
                        false => quote::quote! {
                            #field_ident: None
                        },
                    }   
                });
                quote::quote! {
                    impl std::convert::From<#struct_name_token_stream> for #struct_options_ident_token_stream {
                        fn from(#value_token_stream: #struct_name_token_stream) -> Self {
                            #struct_options_ident_token_stream {
                                #(#fields_options_token_stream),*
                            }
                        }
                    }
                }
            })
            .collect::<Vec<proc_macro2::TokenStream>>()
    };
    let select_field_token_stream = {
        let select_field_ident_token_stream = {
            let select_field_ident_stringified = format!("{ident}SelectField");
            select_field_ident_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {select_field_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let select_field_variants = fields_named
            .iter()
            .map(|field| {
                let field_ident_stringified = field.ident
                    .clone()
                    .unwrap_or_else(|| {
                        panic!("GeneratePostgresqlCrud field.ident is None")
                    })
                    .to_string();
                let serialize_deserialize_ident_token_stream = format!("\"{field_ident_stringified}\"").parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
                let variant_ident_token_stream = {
                    use convert_case::Casing;
                    let variant_ident_stringified = field_ident_stringified.to_case(convert_case::Case::Title);
                    variant_ident_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {variant_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                quote::quote! {
                    #[serde(rename(serialize = #serialize_deserialize_ident_token_stream, deserialize = #serialize_deserialize_ident_token_stream))]
                    #variant_ident_token_stream
                }
            })
            .collect::<Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            #[derive(
                Debug,
                serde::Serialize,
                serde::Deserialize,
                enum_extension::EnumExtension,
                strum_macros::EnumIter,
                PartialEq,
                Eq,
            )]
            pub enum #select_field_ident_token_stream {
                #(#select_field_variants),*
            }
            impl std::fmt::Display for #select_field_ident_token_stream {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}", Self::to_lower_snake_case(self))
                }
            }
            impl crate::common::url_encode::UrlEncode for #select_field_ident_token_stream {
                fn url_encode(&self) -> std::string::String {
                    urlencoding::encode(&self.to_string()).to_string()
                }
            }
        }
    };
    let select_token_stream = {
        let select_ident_token_stream = {
            let select_ident_stringified = format!("{ident}Select");
            select_ident_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {select_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let select_variants_token_stream = column_variants.iter().map(|column_variant|{
            let serialize_deserialize_ident_token_stream = {
                let mut serialize_deserialize_ident_stringified_handle = column_variant.iter()
                    .fold(std::string::String::from(""), |mut acc, field| {
                        let field_ident_stringified = field.ident
                            .clone()
                            .unwrap_or_else(|| {
                                panic!("GeneratePostgresqlCrud field.ident is None")
                            });
                        acc.push_str(&format!("{field_ident_stringified},"));
                        acc
                    });
                serialize_deserialize_ident_stringified_handle.pop();
                format!("\"{serialize_deserialize_ident_stringified_handle}\"").parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {serialize_deserialize_ident_stringified_handle} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let variant_ident_token_stream = {
                let variant_ident_stringified_handle = column_variant.iter()
                    .fold(std::string::String::from(""), |mut acc, field| {
                        use convert_case::Casing;
                        let field_ident_stringified = field.ident
                            .clone()
                            .unwrap_or_else(|| {
                                panic!("GeneratePostgresqlCrud field.ident is None")
                            }).to_string().to_case(convert_case::Case::Title);
                        acc.push_str(&field_ident_stringified);
                        acc
                    });
                variant_ident_stringified_handle.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {variant_ident_stringified_handle} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote! {
                #[serde(rename(serialize = #serialize_deserialize_ident_token_stream, deserialize = #serialize_deserialize_ident_token_stream))]
                #variant_ident_token_stream
            }
        });
        let select_impl_display_token_stream = column_variants.iter().map(|column_variant|{
            let write_ident_token_stream = {
                let mut write_ident_stringified_handle = column_variant.iter()
                    .fold(std::string::String::from(""), |mut acc, field| {
                        let field_ident_stringified = field.ident
                            .clone()
                            .unwrap_or_else(|| {
                                panic!("GeneratePostgresqlCrud field.ident is None")
                            });
                        acc.push_str(&format!("{field_ident_stringified},"));
                        acc
                    });
                write_ident_stringified_handle.pop();
                format!("\"{write_ident_stringified_handle}\"").parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {write_ident_stringified_handle} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let variant_ident_token_stream = {
                let variant_ident_stringified_handle = column_variant.iter()
                    .fold(std::string::String::from(""), |mut acc, field| {
                        use convert_case::Casing;
                        let field_ident_stringified = field.ident
                            .clone()
                            .unwrap_or_else(|| {
                                panic!("GeneratePostgresqlCrud field.ident is None")
                            }).to_string().to_case(convert_case::Case::Title);
                        acc.push_str(&field_ident_stringified);
                        acc
                    });
                variant_ident_stringified_handle.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {variant_ident_stringified_handle} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote! {
                #select_ident_token_stream::#variant_ident_token_stream => write!(f, #write_ident_token_stream)
            }
        });
        let default_select_variant_ident_token_stream = {
            let default_select_variant_ident_stringified = fields_named.iter()
            .fold(std::string::String::from(""), |mut acc, field| {
                use convert_case::Casing;
                let field_ident_stringified = field.ident
                    .clone()
                    .unwrap_or_else(|| {
                        panic!("GeneratePostgresqlCrud field.ident is None")
                    }).to_string().to_case(convert_case::Case::Title);
                acc.push_str(&field_ident_stringified);
                acc
            });
            default_select_variant_ident_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {default_select_variant_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let assignment_token_stream = column_variants.iter().map(|column_variant|{
            let write_ident_token_stream = column_variant.iter().map(|field|{
                let field_ident = field.ident.clone()
                .unwrap_or_else(|| {
                    panic!("GeneratePostgresqlCrud field.ident is None")
                });
                let field_ident_string_quotes_token_stream = {
                    let field_ident_string_quotes = format!("\"{field_ident}\"");
                    field_ident_string_quotes.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_string_quotes} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                quote::quote! {
                    #field_ident = row.try_get(#field_ident_string_quotes_token_stream)?;
                }  
            });
            let variant_ident_token_stream = {
                let variant_ident_stringified_handle = column_variant.iter()
                    .fold(std::string::String::from(""), |mut acc, field| {
                        use convert_case::Casing;
                        let field_ident_stringified = field.ident
                            .clone()
                            .unwrap_or_else(|| {
                                panic!("GeneratePostgresqlCrud field.ident is None")
                            }).to_string().to_case(convert_case::Case::Title);
                        acc.push_str(&field_ident_stringified);
                        acc
                    });
                variant_ident_stringified_handle.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {variant_ident_stringified_handle} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote! {
                #select_ident_token_stream::#variant_ident_token_stream => {
                    #(#write_ident_token_stream)*
                }
            }
        });
        let declaration_token_stream = fields_named.iter().map(|field|{
            let field_ident = field.ident
                .clone()
                .unwrap_or_else(|| {
                    panic!("GeneratePostgresqlCrud field.ident is None")
                });
            let field_type = &field.ty;
            quote::quote! {
                let mut #field_ident: Option<#field_type> = None;
            }
        });
        let option_fields_initiation_token_stream = fields_named.iter().map(|field|{
            field.ident
                .clone()
                .unwrap_or_else(|| {
                    panic!("GeneratePostgresqlCrud field.ident is None")
                })
        });
        quote::quote! {
            #[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
            pub enum #select_ident_token_stream {
                #(#select_variants_token_stream),*
            }
            impl std::fmt::Display for #select_ident_token_stream {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#select_impl_display_token_stream),*
                    }
                }
            }
            impl std::default::Default for #select_ident_token_stream {
                fn default() -> Self {
                    Self::#default_select_variant_ident_token_stream
                }
            }
            impl std::convert::From<Option<Self>> for #select_ident_token_stream {
                fn from(option_value: Option<Self>) -> Self {
                    match option_value {
                        Some(value) => value,
                        None => Self::default(),
                    }
                }
            }
            impl crate::common::url_encode::UrlEncode for #select_ident_token_stream {
                fn url_encode(&self) -> std::string::String {
                    urlencoding::encode(&self.to_string()).to_string()
                }
            }
            impl #select_ident_token_stream {
                fn options_try_from_sqlx_row<'a, R: ::sqlx::Row>(
                    &self,
                    row: &'a R,
                ) -> ::sqlx::Result<#struct_options_ident_token_stream>
                where
                    &'a ::std::primitive::str: ::sqlx::ColumnIndex<R>,
                    Option<i64>: ::sqlx::decode::Decode<'a, R::Database>,
                    Option<i64>: ::sqlx::types::Type<R::Database>,
                    Option<String>: ::sqlx::decode::Decode<'a, R::Database>,
                    Option<String>: ::sqlx::types::Type<R::Database>,
                    Option<String>: ::sqlx::decode::Decode<'a, R::Database>,
                    Option<String>: ::sqlx::types::Type<R::Database>,
                {
                    #(#declaration_token_stream)*
                    match self {
                        #(#assignment_token_stream)*
                    }
                    Ok(#struct_options_ident_token_stream { 
                        #(#option_fields_initiation_token_stream),*
                    })
                }
            }
        }
    };
    let gen = quote::quote! {
        #struct_options_token_stream
        #from_ident_for_ident_options_token_stream
        #(#structs_variants_token_stream)*
        #(#structs_variants_impl_from_token_stream)*
        #select_field_token_stream
        #select_token_stream

    };
    // println!("{gen}");
    gen.into()
}

fn column_names_factorial(
    original_input: Vec<(usize, &syn::Field)>,
    input: Vec<&syn::Field>,
    output: &mut Vec<Vec<syn::Field>>,
) -> Vec<Vec<syn::Field>> {
    let len = input.len();
    match len {
        0 => {
            let mut end_out = {
                let output_len = output.len();
                output
                    .iter_mut()
                    .fold(Vec::with_capacity(output_len), |mut acc, element| {
                        element.sort_by(|a, b| {
                            let (index_a, _) = original_input
                                .iter()
                                .find(|(_, field)| {
                                    a.ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })
                                        == field
                                            .ident
                                            .clone()
                                            .unwrap_or_else(|| {
                                                panic!("GeneratePostgresqlCrud field.ident is None")
                                            })
                                })
                                .unwrap_or_else(|| {
                                    panic!(
                                        "GeneratePostgresqlCrud cannot find original input index"
                                    )
                                });
                            let (index_b, _) = original_input
                                .iter()
                                .find(|(_, field)| {
                                    b.ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })
                                        == field
                                            .ident
                                            .clone()
                                            .unwrap_or_else(|| {
                                                panic!("GeneratePostgresqlCrud field.ident is None")
                                            })
                                })
                                .unwrap_or_else(|| {
                                    panic!(
                                        "GeneratePostgresqlCrud cannot find original input index"
                                    )
                                });
                            index_a.partial_cmp(index_b).unwrap_or_else(|| {
                                panic!(
                                    "GeneratePostgresqlCrud index_a.partial_cmp(index_b) is None"
                                )
                            })
                        });
                        acc.push(element.to_vec());
                        acc
                    })
            };
            end_out.sort_by(|a, b| match a.len() == b.len() {
                true => {
                    let mut order = std::cmp::Ordering::Equal;
                    for a_elem in a {
                        let mut is_order_found = false;
                        for b_elem in a {
                            if let Some(or) = a_elem
                                .ident
                                .clone()
                                .unwrap_or_else(|| {
                                    panic!("GeneratePostgresqlCrud field.ident is None")
                                })
                                .to_string()
                                .partial_cmp(
                                    &b_elem
                                        .ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })
                                        .to_string(),
                                )
                            {
                                match or {
                                    std::cmp::Ordering::Less => {
                                        order = or;
                                        is_order_found = true;
                                        break;
                                    }
                                    std::cmp::Ordering::Equal => (),
                                    std::cmp::Ordering::Greater => {
                                        order = or;
                                        is_order_found = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if let true = is_order_found {
                            break;
                        }
                    }
                    order
                }
                false => std::cmp::Ordering::Equal,
            });
            end_out.sort_by(|a, b| {
                a.len().partial_cmp(&b.len()).unwrap_or_else(|| {
                    panic!("GeneratePostgresqlCrud index_a.partial_cmp(index_b) is None")
                })
            });
            end_out
        }
        // 1 => {
        //     let mut output_handle = vec![];
        //     original_input.iter().for_each(|(_, element)| {
        //         output_handle.push(vec![element.clone()]);
        //     });
        //     let first_element = input.get(0).unwrap_or_else(||panic!("GeneratePostgresqlCrud input.get(0) is None"));
        //     output.iter().for_each(
        //         |o| {
        //             if let false = o.contains(first_element) {
        //                 let mut cl = o.clone();
        //                 cl.push(format!("{}", input[0]));
        //                 cl.sort_by(|a,b|{
        //                     let (index_a, _) = original_input.iter().find(|(_, field)|{a == field}).unwrap();
        //                     let (index_b, _) = original_input.iter().find(|(_, field)|{b == field}).unwrap();
        //                     index_a.partial_cmp(index_b).unwrap()
        //                 });
        //                 output_handle.push(cl);
        //             }
        //         },
        //     );
        //     output_handle
        // }
        _ => {
            let mut output_handle = {
                let first_element = input
                    .get(0)
                    .unwrap_or_else(|| panic!("GeneratePostgresqlCrud input.get(0) is None"));
                let output_len = output.len();
                output.iter_mut().fold(Vec::with_capacity(output_len * 2), |mut acc, out| {
                    if !acc.contains(out) {
                        out.sort_by(|a,b|{
                            let (index_a, _) = original_input.iter().find(|(_, field)|{a.ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        }) == field
                                        .ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
                            let (index_b, _) = original_input.iter().find(|(_, field)|{b.ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        }) == field
                                        .ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
                            index_a.partial_cmp(index_b).unwrap_or_else(||panic!("GeneratePostgresqlCrud index_a.partial_cmp(index_b) is None"))
                        });
                        acc.push(out.clone());
                    }
                    if let false = out.contains(first_element) {
                        let mut cl = out.clone();
                        cl.push((*first_element).clone());
                        cl.sort_by(|a,b|{
                            let (index_a, _) = original_input.iter().find(|(_, field)|{a.ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        }) == field
                                        .ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
                            let (index_b, _) = original_input.iter().find(|(_, field)|{b.ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        }) == field
                                        .ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
                            index_a.partial_cmp(index_b).unwrap_or_else(||panic!("GeneratePostgresqlCrud index_a.partial_cmp(index_b) is None"))
                        });
                        if !acc.contains(&cl) {
                            acc.push(cl);
                        }
                    }
                    acc
                })
            };
            let new_input_vec = {
                let input_len = input.len();
                input.into_iter().enumerate().fold(
                    Vec::with_capacity(input_len),
                    |mut acc, (index, value)| {
                        if let true = index != 0 {
                            acc.push(value);
                        }
                        acc
                    },
                )
            };
            column_names_factorial(original_input, new_input_vec, &mut output_handle)
        }
    }
}
