mod column_names_factorial;
mod check_for_none;
mod acquire_pool_and_connection;
mod from_log_and_return_error;
mod generate_postgres_transaction;
mod generate_postgres_execute_query;

fn get_macro_attribute<'a>(
    attrs: &'a [syn::Attribute],
    proc_macro_name_ident_stringified: &std::string::String
) -> &'a syn::Attribute {
    let attribute_path: &str = "generate_postgresql_crud::generate_postgresql_crud_route_name";
    let option_attribute = attrs.iter().find(|attr| {
        attribute_path == {
            let mut stringified_path = quote::ToTokens::to_token_stream(&attr.path).to_string();
            stringified_path.retain(|c| !c.is_whitespace());
            stringified_path
        }
    });
    if let Some(attribute) = option_attribute {
        attribute
    }
    else {
        panic!("{proc_macro_name_ident_stringified} no {attribute_path}");
    }
}

#[proc_macro_attribute]
pub fn generate_postgresql_crud_route_name(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

//todo rename ForUrlEncoding prefix
//todo clear unnesesary generated returns.
// unneeded `return` statement
// for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#needless_return
// `#[warn(clippy::needless_return)]` 
//todo validate uuid
//todo add regex filter to query parameters - now supports only in body variants
//todo regex filter support only for string-like types postgresql
//todo generate route what will return columns of the table and their rust and postgersql types
//todo - check if fields for filter are unique in the input array
//todo created at and updated at fields
//todo attributes for activation generation crud methods(like generate create, update_one, delete_one)
//todo authorization for returning concrete error or just minimal info(user role)
//todo generate rules and roles
//todo unique(meaning not primary key unique column) and nullable support
//todo add check on max postgresql bind elements
//todo add route name as argument for macro - generation constant and add to generation logic
//todo make sqlx macros instead of just queries?
//todo support for arrays as column values
//todo maybe add unnest sql types?
//todo maybe add unnest to filter parameters if its array ?
//todo swagger ui https://github.com/juhaku/utoipa/blob/master/examples/todo-axum/src/main.rs
#[proc_macro_derive(
    GeneratePostgresqlCrud,
    attributes(
        generate_postgresql_crud_primary_key,
        //todo add attributes for postgresql types
    )
)]//todo check on postgresql max length value of type
pub fn generate_postgresql_crud(input: proc_macro::TokenStream) -> proc_macro::TokenStream {//todo in few cases rows affected is usefull. (update delete for example). if 0 afftected -maybe its error? or maybe use select then update\delete?(rewrite query)
    proc_macro_helpers::panic_location::panic_location();
    let proc_macro_name = "GeneratePostgresqlCrud";
    let ast: syn::DeriveInput = syn::parse(input).unwrap_or_else(|_| {
        panic!(
            "{proc_macro_name} {}",
            proc_macro_helpers::global_variables::hardcode::AST_PARSE_FAILED
        )
    });
    let ident = &ast.ident;
    let ident_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&ident.to_string());
    let proc_macro_name_ident_stringified = format!("{proc_macro_name} {ident}");
    let stringified_tokens = {
        let attribute = get_macro_attribute(
            &ast.attrs,
            &proc_macro_name_ident_stringified
        );
        let mut stringified_tokens = quote::ToTokens::to_token_stream(&attribute.tokens).to_string();
        stringified_tokens.retain(|c| !c.is_whitespace());
        stringified_tokens
    };
    let table_name_stringified = {
        match stringified_tokens.len() > 3 {
            true => {
                let chars = &mut stringified_tokens.chars();
                match (&chars.next(), &chars.last()) {
                        (None, None) => panic!("{proc_macro_name_ident_stringified} no first and last token attribute"),
                        (None, Some(_)) => panic!("{proc_macro_name_ident_stringified} no first token attribute"),
                        (Some(_), None) => panic!("{proc_macro_name_ident_stringified} no last token attribute"),
                        (Some(first), Some(last)) => match (first == &'(', last == &')') {
                            (true, true) => {
                                match stringified_tokens.get(1..(stringified_tokens.len()-1)) {
                                    Some(inner_tokens_str) => {
                                        inner_tokens_str
                                    },
                                    None => panic!("{proc_macro_name_ident_stringified} cannot get inner_token"),
                                }
                            },
                            (true, false) => panic!("{proc_macro_name_ident_stringified} last token attribute is not )"),
                            (false, true) => panic!("{proc_macro_name_ident_stringified} first token attribute is not ("),
                            (false, false) => panic!("{proc_macro_name_ident_stringified} first token attribute is not ( and last token attribute is not )"),
                        },
                    }
            }
            false => panic!("{proc_macro_name_ident_stringified} {stringified_tokens}.len() > 3 == false"),
        }
    };
    let data_struct = if let syn::Data::Struct(data_struct) = ast.data {
        data_struct
    } else {
        panic!("{proc_macro_name_ident_stringified} does not work on structs!");
    };
    let fields_named = if let syn::Fields::Named(fields_named) = data_struct.fields {
        fields_named.named
    } else {
        panic!("{proc_macro_name_ident_stringified} supports only syn::Fields::Named");
    };
    let id_field = {
        let id_attr_name = "generate_postgresql_crud_primary_key";
        let mut id_field_option = None;
        for field_named in &fields_named {
            let attrs = &field_named.attrs;
            if let 1 = attrs.len() {
                match attrs.get(0) {
                    Some(attr) => match proc_macro_helpers::error_occurence::generate_path_from_segments::generate_path_from_segments(&attr.path.segments) == id_attr_name {
                        true => match id_field_option {
                            Some(_) => panic!("{proc_macro_name_ident_stringified} must have one id attribute"),
                            None => {
                                id_field_option = Some(field_named.clone());
                            },
                        },
                        false => (),
                    },
                    None => panic!("{proc_macro_name_ident_stringified} field_named.attrs.len() == 1, but attrs.get(0) == None"),
                }
            }
        }
        match id_field_option {
            Some(id_field) => id_field,
            None => panic!("{proc_macro_name_ident_stringified} no {id_attr_name} attribute"),
        }
    };
    let id_field_type = &id_field.ty;
    // println!("{id_field:#?}");
    let id_field_ident = id_field.ident.clone()
        .unwrap_or_else(|| {
            panic!("{proc_macro_name_ident_stringified} id_field.ident is None")
        });
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
    let table_name_declaration_token_stream = {
        let table_name_quotes_token_stream = {
            let table_name_quotes_stringified = format!("\"{table_name_stringified}\"");
            table_name_quotes_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {table_name_quotes_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        quote::quote! {pub const TABLE_NAME: &str = #table_name_quotes_token_stream;}
    };
    let error_named_derive_token_stream = quote::quote!{#[derive(Debug, thiserror::Error, error_occurence::ErrorOccurence)]};
    let derive_debug_token_stream = quote::quote!{#[derive(Debug)]};
    let derive_debug_deserialize_token_stream = quote::quote!{#[derive(Debug, serde::Deserialize)]};
    let derive_debug_serialize_deserialize_token_stream = quote::quote!{#[derive(Debug, serde::Serialize, serde::Deserialize)]};
    let struct_options_token_stream = quote::quote! {
        #derive_debug_serialize_deserialize_token_stream
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
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
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
        crate::column_names_factorial::column_names_factorial(
            fields_named_enumerated,
            fields_named_clone_stringified,
            &mut veced_vec,
            &proc_macro_name_ident_stringified
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
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
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
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    let variant_column_type = &variant_column.ty;
                    quote::quote! {
                        pub #variant_column_ident: #variant_column_type,
                    }
                });
                quote::quote! {
                    #[derive(Debug)]
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
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
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
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
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
    let column_ident_token_stream = {
        let column_ident_stringified = format!("{ident}Column");
        column_ident_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {column_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let column_token_stream = {
        let column_variants = fields_named
            .iter()
            .map(|field| {
                let field_ident_stringified = field.ident
                    .clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
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
                from_str::FromStr,
            )]
            pub enum #column_ident_token_stream {
                #(#column_variants),*
            }
            impl std::fmt::Display for #column_ident_token_stream {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}", Self::to_lower_snake_case(self))
                }
            }
        }
    };
    let column_select_ident_token_stream = {
        let column_select_ident_stringified = format!("{ident}ColumnSelect");
        column_select_ident_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {column_select_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let options_try_from_sqlx_row_name_token_stream = quote::quote!{options_try_from_sqlx_row};
    let std_primitive_str_sqlx_column_index_token_stream = quote::quote!{&'a std::primitive::str: sqlx::ColumnIndex<R>,};
    let sqlx_decode_decode_database_token_stream = quote::quote!{sqlx::decode::Decode<'a, R::Database>};
    let sqlx_types_type_database_token_stream = quote::quote!{sqlx::types::Type<R::Database>};
    let crate_common_serde_urlencoded_serde_url_encoded_parameter_token_stream = quote::quote!{crate::common::serde_urlencoded::SerdeUrlencodedParameter};
    let column_select_token_stream = {
        let column_select_struct_token_stream = {
            let column_select_variants_token_stream = column_variants.iter().map(|column_variant|{
                let variant_ident_token_stream = {
                    let variant_ident_stringified_handle = column_variant.iter()
                        .fold(std::string::String::default(), |mut acc, field| {
                            use convert_case::Casing;
                            let field_ident_stringified = field.ident
                                .clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                }).to_string().to_case(convert_case::Case::Title);
                            acc.push_str(&field_ident_stringified);
                            acc
                        });
                    variant_ident_stringified_handle.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {variant_ident_stringified_handle} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                quote::quote! {
                    #variant_ident_token_stream
                }
            });
            quote::quote! {
                #[derive(Debug, serde::Serialize, serde::Deserialize, Clone, strum_macros::Display)]
                pub enum #column_select_ident_token_stream {
                    #(#column_select_variants_token_stream),*
                }
            }
        };
        let generate_query_token_stream = {
            let generate_query_variants_token_stream = column_variants.iter().map(|column_variant|{
                let write_ident_token_stream = {
                    let mut write_ident_stringified_handle = column_variant.iter()
                        .fold(std::string::String::default(), |mut acc, field| {
                            let field_ident_stringified = field.ident
                                .clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
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
                        .fold(std::string::String::default(), |mut acc, field| {
                            use convert_case::Casing;
                            let field_ident_stringified = field.ident
                                .clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                }).to_string().to_case(convert_case::Case::Title);
                            acc.push_str(&field_ident_stringified);
                            acc
                        });
                    variant_ident_stringified_handle.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {variant_ident_stringified_handle} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                quote::quote! {
                    Self::#variant_ident_token_stream => std::string::String::from(#write_ident_token_stream)
                }
            });
            quote::quote!{
                impl crate::server::postgres::generate_query::GenerateQuery for #column_select_ident_token_stream {
                    fn generate_query(&self) -> std::string::String {
                        match self {
                            #(#generate_query_variants_token_stream),*
                        }
                    }
                }
            }
        };
        let impl_default_token_stream = {
            let default_select_variant_ident_token_stream = {
                let default_select_variant_ident_stringified = fields_named.iter()
                .fold(std::string::String::default(), |mut acc, field| {
                    use convert_case::Casing;
                    let field_ident_stringified = field.ident
                        .clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        }).to_string().to_case(convert_case::Case::Title);
                    acc.push_str(&field_ident_stringified);
                    acc
                });
                default_select_variant_ident_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {default_select_variant_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote! {
                impl std::default::Default for #column_select_ident_token_stream {
                    fn default() -> Self {
                        Self::#default_select_variant_ident_token_stream
                    }
                }
            }
        };
        let from_option_self_token_stream = {
            quote::quote! {
                impl std::convert::From<Option<Self>> for #column_select_ident_token_stream {
                    fn from(option_value: Option<Self>) -> Self {
                        match option_value {
                            Some(value) => value,
                            None => Self::default(),
                        }
                    }
                }
            }
        };
        let serde_urlencoded_parameter_token_stream = {
            quote::quote! {
                impl #crate_common_serde_urlencoded_serde_url_encoded_parameter_token_stream for #column_select_ident_token_stream {
                    fn serde_urlencoded_parameter(self) -> std::string::String {
                        self.to_string()
                    }
                }
            }
        };
        let options_try_from_sqlx_row_token_stream = {
            let declaration_token_stream = fields_named.iter().map(|field|{
                let field_ident = field.ident
                    .clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                let field_type = &field.ty;
                quote::quote! {
                    let mut #field_ident: Option<#field_type> = None;
                }
            });
            let assignment_token_stream = column_variants.iter().map(|column_variant|{
                // let write_ident_token_stream = column_variant.iter().map(|field|{
                //     let field_ident = field.ident.clone()
                //     .unwrap_or_else(|| {
                //         panic!("{proc_macro_name_ident_stringified} field.ident is None")
                //     });
                //     let field_ident_string_quotes_token_stream = {
                //         let field_ident_string_quotes = format!("\"{field_ident}\"");
                //         field_ident_string_quotes.parse::<proc_macro2::TokenStream>()
                //         .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_string_quotes} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                //     };
                //     quote::quote! {
                //         #field_ident = row.try_get(#field_ident_string_quotes_token_stream)?;
                //     }  
                // });
                /////////
                let write_ident_primary_key_token_stream = {
                    let field_ident_string_quotes_token_stream = {
                        let field_ident_string_quotes = format!("\"{id_field_ident}\"");
                        field_ident_string_quotes.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_string_quotes} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    quote::quote!{
                        let primary_key_try_get_result: Result<Option<sqlx::types::Uuid>, sqlx::Error> = row.try_get(#field_ident_string_quotes_token_stream);
                        #id_field_ident = match primary_key_try_get_result {
                            Ok(option_primary_key) => option_primary_key.map(|value| value.to_string()),
                            Err(e) => {
                                return Err(e); //todo custom type
                            }
                        };
                    }
                };
                let write_ident_token_stream = column_variant.iter().filter_map(|field|match field == &id_field {
                    true => None,
                    false => {
                        let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                        let field_ident_string_quotes_token_stream = {
                            let field_ident_string_quotes = format!("\"{field_ident}\"");
                            field_ident_string_quotes.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_string_quotes} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        Some(quote::quote!{
                            #field_ident = row.try_get(#field_ident_string_quotes_token_stream)?;
                        })
                    },
                });
                //
                let variant_ident_token_stream = {
                    let variant_ident_stringified_handle = column_variant.iter()
                        .fold(std::string::String::default(), |mut acc, field| {
                            use convert_case::Casing;
                            let field_ident_stringified = field.ident
                                .clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                }).to_string().to_case(convert_case::Case::Title);
                            acc.push_str(&field_ident_stringified);
                            acc
                        });
                    variant_ident_stringified_handle.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {variant_ident_stringified_handle} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                quote::quote! {
                    Self::#variant_ident_token_stream => {
                        #write_ident_primary_key_token_stream
                        #(#write_ident_token_stream)*
                    }
                }
            });
            let option_fields_initiation_token_stream = fields_named.iter().map(|field|{
                field.ident
                    .clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    })
            });
            let sqlx_decode_decode_and_sqlx_types_type_primary_key_token_stream = quote::quote!{
                Option<sqlx::types::Uuid>: #sqlx_decode_decode_database_token_stream,
                Option<sqlx::types::Uuid>: #sqlx_types_type_database_token_stream,
            };
            let sqlx_decode_decode_and_sqlx_types_type_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_type = &field.ty;
                    Some(quote::quote!{
                        Option<#field_type>: #sqlx_decode_decode_database_token_stream,
                        Option<#field_type>: #sqlx_types_type_database_token_stream,
                    })
                },
            });
            quote::quote! {
                impl #column_select_ident_token_stream {
                    fn #options_try_from_sqlx_row_name_token_stream<'a, R: sqlx::Row>(
                        &self,
                        row: &'a R,
                    ) -> sqlx::Result<#struct_options_ident_token_stream>
                    where
                        #std_primitive_str_sqlx_column_index_token_stream
                        #sqlx_decode_decode_and_sqlx_types_type_primary_key_token_stream
                        #(#sqlx_decode_decode_and_sqlx_types_type_with_excluded_id_token_stream)*
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
        // println!("{options_try_from_sqlx_row_token_stream}");
        quote::quote! {
            #column_select_struct_token_stream
            #generate_query_token_stream
            #impl_default_token_stream
            #from_option_self_token_stream
            #serde_urlencoded_parameter_token_stream
            #options_try_from_sqlx_row_token_stream
        }
    };
    // println!("{column_select_token_stream}");
    //todo remove primary_key_try_from_sqlx_row_name_token_stream and primary_key_try_from_sqlx_row_token_stream later - primary_key_uuid_wrapper_try_from_sqlx_row_token_stream is new version
    let primary_key_try_from_sqlx_row_name_token_stream = quote::quote!{primary_key_try_from_sqlx_row};
    let primary_key_try_from_sqlx_row_token_stream = {
        let primary_key_str_token_stream = {
            let primary_key_str_stringified = format!("\"{id_field_ident}\"");
            primary_key_str_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {primary_key_str_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let row_name_token_stream = quote::quote!{row};
        let primary_key_name_token_stream = quote::quote!{primary_key};
        quote::quote! {
            fn #primary_key_try_from_sqlx_row_name_token_stream<'a, R: sqlx::Row>(#row_name_token_stream: &'a R) -> sqlx::Result<#id_field_type>
                where
                    #std_primitive_str_sqlx_column_index_token_stream
                    #id_field_type: #sqlx_decode_decode_database_token_stream,
                    #id_field_type: #sqlx_types_type_database_token_stream,
            {
                let #primary_key_name_token_stream: #id_field_type = #row_name_token_stream.try_get(#primary_key_str_token_stream)?;
                Ok(#primary_key_name_token_stream)
            }
        }
    };
    // println!("{primary_key_try_from_sqlx_row_token_stream}");
    //
    let primary_key_uuid_wrapper_try_from_sqlx_row_name_token_stream = quote::quote!{primary_key_uuid_wrapper_try_from_sqlx_row};
    let primary_key_uuid_wrapper_try_from_sqlx_row_token_stream = {
        let primary_key_str_token_stream = {
            let primary_key_str_stringified = format!("\"{id_field_ident}\"");
            primary_key_str_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {primary_key_str_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let row_name_token_stream = quote::quote!{row};
        let primary_key_name_token_stream = quote::quote!{primary_key};
        quote::quote! {
            fn #primary_key_uuid_wrapper_try_from_sqlx_row_name_token_stream<'a, R: sqlx::Row>(#row_name_token_stream: &'a R) -> sqlx::Result<crate::server::postgres::uuid_wrapper::UuidWrapper>
            where
                #std_primitive_str_sqlx_column_index_token_stream
                sqlx::types::Uuid: #sqlx_decode_decode_database_token_stream,
                sqlx::types::Uuid: #sqlx_types_type_database_token_stream,
            {
                let #primary_key_name_token_stream: sqlx::types::Uuid = #row_name_token_stream.try_get(#primary_key_str_token_stream)?;
                Ok(crate::server::postgres::uuid_wrapper::UuidWrapper::from(#primary_key_name_token_stream))
            }
        }
    };
    // println!("{primary_key_uuid_wrapper_try_from_sqlx_row_token_stream}");
    let crate_server_postgres_order_by_order_by_token_stream = quote::quote!{crate::server::postgres::order_by::OrderBy};
    let ident_order_by_wrapper_token_stream = {
        let ident_order_by_wrapper_stringified = format!("{ident}OrderByWrapper");
        ident_order_by_wrapper_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ident_order_by_wrapper_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let order_by_wrapper_token_stream = {
        let deserialize_with_name_quotes_token_stream = {
            let deserialize_with_name_quotes_stringified = format!("\"deserialize_{ident_lower_case_stringified}_order_by\"");
            deserialize_with_name_quotes_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {deserialize_with_name_quotes_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        quote::quote!{
            #derive_debug_serialize_deserialize_token_stream
            pub struct #ident_order_by_wrapper_token_stream(
                #[serde(deserialize_with = #deserialize_with_name_quotes_token_stream)]
                pub #crate_server_postgres_order_by_order_by_token_stream<#column_ident_token_stream>,
            );
        }
    };
    let impl_crate_common_serde_urlencoded_serde_urlencoded_parameter_for_ident_order_by_wrapper_token_stream = {
        quote::quote!{
            impl #crate_common_serde_urlencoded_serde_url_encoded_parameter_token_stream for #ident_order_by_wrapper_token_stream {
                fn serde_urlencoded_parameter(self) -> std::string::String {
                    let column = &self.0.column;
                    let order = self.0.order.unwrap_or_default();
                    format!("column={column},order={order}")
                }
            }
        }
    };
    let deserialize_ident_order_by_token_stream = {
        //todo
        let ivalid_ident_order_by_handle_token_stream = {
            let ivalid_ident_order_by_handle = format!("\"Invalid {ident}OrderBy:\"");
            ivalid_ident_order_by_handle.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ivalid_ident_order_by_handle} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let deserialize_ident_order_by_lower_case_name_token_stream = {
            let deserialize_ident_order_by_lower_case_name = format!("deserialize_{ident_lower_case_stringified}_order_by");
            deserialize_ident_order_by_lower_case_name.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {deserialize_ident_order_by_lower_case_name} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        quote::quote!{
            fn #deserialize_ident_order_by_lower_case_name_token_stream<'de, D>(
                deserializer: D,
            ) -> Result<crate::server::postgres::order_by::OrderBy<#column_ident_token_stream>, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                let string_deserialized = {
                    use serde::Deserialize;
                    String::deserialize(deserializer)?
                };
                let split_inner_url_parameters_symbol = ',';
                let default_message = format!(#ivalid_ident_order_by_handle_token_stream);
                let column_equal_str = "column=";
                let order_equal_str = "order=";
                let column = match string_deserialized.find(column_equal_str) {
                    Some(index) => match index.checked_add(column_equal_str.len()) {
                        Some(offset) => match string_deserialized.get(offset..) {
                            Some(offset_slice) => match offset_slice.find(split_inner_url_parameters_symbol) {
                                Some(offset_slice_next_comma_index) => {
                                    match offset_slice.get(0..offset_slice_next_comma_index) {
                                        Some(possible_column) => match {
                                            use std::str::FromStr;
                                            #column_ident_token_stream::from_str(possible_column)
                                        } {
                                            Ok(column) => column,
                                            Err(e) => {
                                                return Err(serde::de::Error::custom(&format!(
                                                    "{default_message} {column_equal_str} {e}"
                                                )));
                                            }
                                        },
                                        None => {
                                            return Err(serde::de::Error::custom(&format!(
                                                "{default_message} {column_equal_str} failed to offset_slice.get(0..offset_slice_next_comma_index)"
                                            )));
                                        }
                                    }
                                }
                                None => match offset_slice.get(0..) {
                                    Some(possible_column) => match {
                                        use std::str::FromStr;
                                        #column_ident_token_stream::from_str(possible_column)
                                    } {
                                        Ok(column) => column,
                                        Err(e) => {
                                            return Err(serde::de::Error::custom(&format!(
                                                "{default_message} {column_equal_str} {e}"
                                            )));
                                        }
                                    },
                                    None => {
                                        return Err(serde::de::Error::custom(&format!(
                                            "{default_message} {column_equal_str} failed to offset_slice.get(0..)"
                                        )));
                                    }
                                },
                            },
                            None => {
                                return Err(serde::de::Error::custom(&format!(
                                    "{default_message} {column_equal_str} failed to string_deserialized.get(offset..)"
                                )));
                            }
                        },
                        None => {
                            return Err(serde::de::Error::custom(&format!(
                                "{default_message} {column_equal_str} index overflow"
                            )));
                        }
                    },
                    None => {
                        return Err(serde::de::Error::custom(&format!(
                            "{default_message} {column_equal_str} not found"
                        )));
                    }
                };
                let order = match string_deserialized.find(order_equal_str) {
                    Some(index) => match index.checked_add(order_equal_str.len()) {
                        Some(offset) => match string_deserialized.get(offset..) {
                            Some(offset_slice) => match offset_slice.find(split_inner_url_parameters_symbol) {
                                Some(offset_slice_next_comma_index) => {
                                    match offset_slice.get(0..offset_slice_next_comma_index) {
                                        Some(possible_order) => match {
                                            use std::str::FromStr;
                                            crate::server::postgres::order::Order::from_str(possible_order)
                                        } {
                                            Ok(order) => Some(order),
                                            Err(e) => {
                                                return Err(serde::de::Error::custom(&format!(
                                                    "{default_message} {order_equal_str} {e}"
                                                )));
                                            }
                                        },
                                        None => {
                                            return Err(serde::de::Error::custom(&format!(
                                                "{default_message} {order_equal_str} failed to offset_slice.get(0..offset_slice_next_comma_index)"
                                            )));
                                        }
                                    }
                                }
                                None => match offset_slice.get(0..) {
                                    Some(possible_order) => match {
                                        use std::str::FromStr;
                                        crate::server::postgres::order::Order::from_str(possible_order)
                                    } {
                                        Ok(order) => Some(order),
                                        Err(e) => {
                                            return Err(serde::de::Error::custom(&format!(
                                                "{default_message} {order_equal_str} {e}"
                                            )));
                                        }
                                    },
                                    None => {
                                        return Err(serde::de::Error::custom(&format!(
                                            "{default_message} {order_equal_str} failed to offset_slice.get(0..)"
                                        )));
                                    }
                                },
                            },
                            None => {
                                return Err(serde::de::Error::custom(&format!(
                                    "{default_message} {order_equal_str} failed to string_deserialized.get(offset..)"
                                )));
                            }
                        },
                        None => {
                            return Err(serde::de::Error::custom(&format!(
                                "{default_message} {order_equal_str} index overflow"
                            )));
                        }
                    },
                    None => None,
                };
                Ok(crate::server::postgres::order_by::OrderBy { column, order })
            }
        }
    };
    let allow_methods_token_stream = {
        quote::quote!{
            pub const ALLOW_METHODS: [http::Method;4] = [http::Method::GET, http::Method::POST, http::Method::PATCH, http::Method::DELETE];
        }
    };
    let ident_column_read_permission_token_stream = {
        let ident_column_read_permission_name_token_stream = {
            let ident_column_read_permission_name = format!("{ident}ColumnReadPermission");
            ident_column_read_permission_name.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ident_column_read_permission_name} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let fields_permission_token_stream = fields_named.iter().map(|field| {
            let field_ident = field.ident.clone()
                .unwrap_or_else(|| {
                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                });
            quote::quote!{
                #field_ident: bool
            }
        });
        quote::quote!{
            pub struct #ident_column_read_permission_name_token_stream {
                #(#fields_permission_token_stream),*
            }
        }
    };
    let extraction_result_lower_case_stringified = "extraction_result";
    let parameters_camel_case_stringified = "Parameters";
    // let parameters_camel_case_token_stream = parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    //     .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let parameters_lower_case_token_stream = {
        let parameters_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&parameters_camel_case_stringified);
        parameters_lower_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {parameters_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let path_camel_case_stringified = "Path";
    // let path_camel_case_token_stream = path_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    //     .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {path_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let path_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&path_camel_case_stringified);
    let path_lower_case_token_stream = path_lower_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {path_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let path_extraction_result_lower_case_token_stream = {
        let path_extraction_result_lower_case = format!("{path_lower_case_token_stream}_{extraction_result_lower_case_stringified}");
        path_extraction_result_lower_case.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {path_extraction_result_lower_case} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let query_camel_case_stringified = "Query";
    // let query_camel_case_token_stream = query_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    //     .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE)); 
    let query_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&query_camel_case_stringified);
    let query_lower_case_token_stream = query_lower_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let query_extraction_result_lower_case_token_stream = {
        let query_extraction_result_lower_case = format!("{query_lower_case_token_stream}_{extraction_result_lower_case_stringified}");
        query_extraction_result_lower_case.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_extraction_result_lower_case} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };    
    let payload_camel_case_stringified = "Payload";
    // let payload_camel_case_token_stream = payload_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    //     .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {payload_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let payload_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&payload_camel_case_stringified);
    let payload_lower_case_token_stream = payload_lower_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {payload_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let payload_extraction_result_lower_case_token_stream = {
        let payload_extraction_result_lower_case = format!("{payload_lower_case_token_stream}_{extraction_result_lower_case_stringified}");
        payload_extraction_result_lower_case.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {payload_extraction_result_lower_case} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let non_existing_primary_keys_name_token_stream = quote::quote!{non_existing_primary_keys};
    let expected_updated_primary_keys_name_token_stream = quote::quote!{expected_updated_primary_keys};
    let use_futures_try_stream_ext_token_stream = quote::quote!{use futures::TryStreamExt};
    let query_encode_token_stream = quote::quote!{QueryEncode};
    let url_encoding_token_stream = quote::quote!{url_encoding};
    let serde_urlencoded_ser_error_token_stream = quote::quote!{serde_urlencoded::ser::Error};
    let serde_json_to_string_token_stream = quote::quote!{serde_json::to_string};
    let into_url_encoding_version_name_token_stream = quote::quote!{into_url_encoding_version}; 
    let url_encoding_camel_case_stringified = "UrlEncoding";
    let for_url_encoding_camel_case_stringified = format!("For{url_encoding_camel_case_stringified}");
    let payload_element_camel_case_stringified = format!("{payload_camel_case_stringified}Element");
    let payload_element_with_serialize_deserialize_camel_case_stringified = format!("{payload_element_camel_case_stringified}WithSerializeDeserialize");
    // let from_camel_case_stringified = "From";
    let try_camel_case_stringified = "Try";
    let try_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&try_camel_case_stringified.to_string());
    let response_variants_camel_case_stringified = "ResponseVariants";
    let error_named_camel_case_stringified = "ErrorNamed";
    let tvfrr_extraction_logic_lower_case_stringified = "tvfrr_extraction_logic";
    let request_error_camel_case_stringified = "RequestError";
    let returning_stringified = "returning";
    let returning_id_stringified = format!(" {returning_stringified} {id_field_ident}");
    let primary_key_vec_name_token_stream = quote::quote!{primary_key_vec};
    let rollback_error_name_token_stream = quote::quote!{rollback_error};
    let returning_id_quotes_token_stream = {
        let returning_id_quotes_stringified = format!("\"{returning_id_stringified}\"");
        returning_id_quotes_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {returning_id_quotes_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let request_error_camel_case_token_stream = request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let request_error_lower_case_token_stream = {
        let request_error_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&request_error_camel_case_stringified);
        request_error_lower_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {request_error_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    // let path_to_crud = "crate::repositories_types::tufa_server::routes::api::cats::";
    let app_info_state_path = quote::quote!{crate::repositories_types::tufa_server::routes::api::cats::DynArcGetConfigGetPostgresPoolSendSync};
    let app_info_state_name_token_stream = quote::quote!{app_info_state};
    let error_log_call_token_stream = quote::quote!{
        crate::common::error_logs_logic::error_log::ErrorLog::error_log(
            &error,
            #app_info_state_name_token_stream.as_ref(),
        );
    };
    let code_occurence_camel_case_stringified = "CodeOccurence";
    let code_occurence_camel_case_token_stream = code_occurence_camel_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {code_occurence_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let code_occurence_lower_case_token_stream = {
        let code_occurence_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&code_occurence_camel_case_stringified);
        code_occurence_lower_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {code_occurence_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_common_code_occurence_code_occurence_token_stream = quote::quote!{crate::common::#code_occurence_lower_case_token_stream::#code_occurence_camel_case_token_stream};
    let crate_code_occurence_tufa_common_macro_call_token_stream = quote::quote!{crate::code_occurence_tufa_common!()};
    let request_error_variant_initialization_token_stream = quote::quote!{
        #request_error_camel_case_token_stream {
            #request_error_lower_case_token_stream: e,
            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let eo_error_occurence_attribute_token_stream = quote::quote!{#[eo_error_occurence]};
    let eo_display_attribute_token_stream = quote::quote!{#[eo_display]};
    let serde_json_to_string_camel_case_stringified = "SerdeJsonToString";
    let serde_json_to_string_camel_case_token_stream = serde_json_to_string_camel_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {serde_json_to_string_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let serde_json_to_string_lower_case_token_stream = {
        let serde_json_to_string_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&serde_json_to_string_camel_case_stringified);
        serde_json_to_string_lower_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {serde_json_to_string_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let serde_json_to_string_variant_initialization_token_stream = quote::quote!{
        #serde_json_to_string_camel_case_token_stream {
            #serde_json_to_string_lower_case_token_stream: e,
            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let http_request_error_named_serde_json_to_string_variant_token_stream = quote::quote!{
        #serde_json_to_string_camel_case_token_stream {
            #eo_display_attribute_token_stream
            #serde_json_to_string_lower_case_token_stream: serde_json::Error,
            #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let bind_query_variant_initialization_token_stream = quote::quote!{
        BindQuery { 
            checked_add: e.into_serialize_deserialize_version(), 
            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream
        }
    };
    let query_and_rollback_failed_token_stream = quote::quote!{
        QueryAndRollbackFailed {
            query_error: e,
            #rollback_error_name_token_stream,
            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let primary_key_from_row_and_failed_rollback_token_stream = quote::quote!{
        PrimaryKeyFromRowAndFailedRollback {
            primary_key_from_row: e,
            #rollback_error_name_token_stream,
            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let non_existing_primary_keys_token_stream = quote::quote!{
        NonExistingPrimaryKeys {
            #non_existing_primary_keys_name_token_stream,
            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let non_existing_primary_keys_and_failed_rollback_token_stream = quote::quote!{
        NonExistingPrimaryKeysAndFailedRollback {
            #non_existing_primary_keys_name_token_stream,
            #rollback_error_name_token_stream: e,
            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let commit_failed_token_stream = quote::quote!{
        CommitFailed {
            commit_error: e,
            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let not_unique_primary_keys_name_token_stream = quote::quote!{not_unique_primary_keys};
    let not_unique_primery_key_token_stream = quote::quote!{
        NotUniquePrimaryKey {
            not_unique_primary_keys,
            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let query_encode_variant_token_stream = quote::quote!{
        #query_encode_token_stream {
            #eo_display_attribute_token_stream
            #url_encoding_token_stream: #serde_urlencoded_ser_error_token_stream,
            #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let query_encode_variant_initialization_token_stream = quote::quote!{
        #query_encode_token_stream {
            #url_encoding_token_stream: e,
            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let project_commit_header_addition_token_stream = quote::quote!{
        .header(
            crate::common::git::project_git_info::PROJECT_COMMIT,
            crate::global_variables::compile_time::project_git_info::PROJECT_GIT_INFO.project_commit,
        )
    };
    let content_type_application_json_header_addition_token_stream = quote::quote!{
        .header(reqwest::header::CONTENT_TYPE, "application/json")
    };
    let impl_axum_response_into_response_token_stream = quote::quote!{impl axum::response::IntoResponse};
    let crate_server_routes_helpers_path_extractor_error_path_value_result_extractor_token_stream = quote::quote!{crate::server::routes::helpers::path_extractor_error::PathValueResultExtractor};
    let crate_server_routes_helpers_query_extractor_error_query_value_result_extractor_token_stream = quote::quote!{crate::server::routes::helpers::query_extractor_error::QueryValueResultExtractor};
    let crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream = quote::quote!{crate::server::routes::helpers::json_extractor_error::JsonValueResultExtractor};
    let axum_extract_rejection_path_rejection_token_stream = quote::quote!{axum::extract::rejection::PathRejection};
    let axum_extract_rejection_query_rejection_token_stream = quote::quote!{axum::extract::rejection::QueryRejection};
    let axum_extract_rejection_json_rejection_token_stream = quote::quote!{axum::extract::rejection::JsonRejection};
    let try_extract_value_token_stream = quote::quote!{try_extract_value};
    let server_location_name_token_stream = quote::quote!{server_location};
    let server_location_type_token_stream = quote::quote!{&str};
    let crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream = quote::quote!{crate::server::postgres::bind_query::BindQuery::bind_value_to_query};
    let crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream = quote::quote!{crate::server::postgres::bind_query::BindQuery::try_generate_bind_increments};
    let crate_server_postgres_bind_query_bind_query_try_increment_token_stream = quote::quote!{crate::server::postgres::bind_query::BindQuery::try_increment};
    let crate_common_serde_urlencoded_serde_urlencoded_parameter_serde_urlencoded_parameter_token_stream = quote::quote!{#crate_common_serde_urlencoded_serde_url_encoded_parameter_token_stream::serde_urlencoded_parameter};
    let fields_named_len = fields_named.len();
    let dot_space = ", ";
    // let pg_temp_stringified = "pg_temp";
    let pg_connection_token_stream = quote::quote!{pg_connection};
    let desirable_token_stream = quote::quote!{Desirable};
    let query_string_name_token_stream = quote::quote!{query_string};
    let binded_query_name_token_stream = quote::quote!{binded_query};
    let postgres_transaction_token_stream = quote::quote!{postgres_transaction};
    let order_by_token_stream = quote::quote!{order_by};
    let select_token_stream = quote::quote!{select};
    let sqlx_query_sqlx_postgres_token_stream = quote::quote!{sqlx::query::<sqlx::Postgres>};
    let reqwest_client_new_token_stream = quote::quote!{reqwest::Client::new()};
    let axum_extract_state_token_stream = quote::quote!{axum::extract::State};
    let axum_extract_path_token_stream = quote::quote!{axum::extract::Path};
    let axum_extract_query_token_stream = quote::quote!{axum::extract::Query};
    let axum_json_token_stream = quote::quote!{axum::Json};
    let rollback_token_stream = quote::quote!{rollback};
    let commit_token_stream = quote::quote!{commit};
    let begin_token_stream = quote::quote!{begin};
    let use_sqlx_acquire_token_stream = quote::quote!{use sqlx::Acquire};
    let increment_initialization_token_stream = quote::quote!{let mut increment: u64 = 0;};
    let current_vec_len_name_token_stream = quote::quote!{current_vec_len};
    let element_name_token_stream = quote::quote!{element};
    let acc_name_token_stream = quote::quote!{acc};
    let query_name_token_stream = quote::quote!{query};
    let underscore_vec_name_stringified = "_vec";
    let update_name_stringified = "update";
    let as_name_stringified = "as";
    let set_name_stringified = "set";
    let from_name_stringified = "from";
    let insert_name_stringified = "insert";
    let into_name_stringified = "into";
    let values_name_stringified = "values";
    let delete_name_stringified = "delete";
    let where_name_stringified = "where";
    let where_name_qoutes_token_stream = {
        let where_name_qoutes_stringified = format!("\"{where_name_stringified}\"");
        where_name_qoutes_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {where_name_qoutes_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let and_name_stringified = "and";
    let any_name_stringified = "any";
    let array_name_stringified = "array";
    let select_name_stringified = "select";
    let order_by_name_stringified = "order by";
    let limit_name_stringified = "limit";
    let offset_name_stringified = "offset";
    let in_name_stringified = "in";
    let unnest_name_stringified = "unnest";
    let create_many_token_stream = {
        let create_many_name_camel_case_stringified = "CreateMany";
        let create_many_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&create_many_name_camel_case_stringified.to_string());
        let create_many_parameters_camel_case_token_stream = {
            let create_many_parameters_camel_case_stringified = format!("{create_many_name_camel_case_stringified}{parameters_camel_case_stringified}");
            create_many_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_many_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let create_many_payload_element_camel_case_token_stream = {
            let create_many_payload_element_camel_case_stringified = format!("{create_many_name_camel_case_stringified}{payload_element_camel_case_stringified}");
            create_many_payload_element_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_many_payload_element_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let create_many_payload_camel_case_token_stream = quote::quote!{Vec<#create_many_payload_element_camel_case_token_stream>};
        let try_create_many_error_named_camel_case_token_stream = {
            let try_create_many_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{create_many_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_create_many_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_many_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_create_many_response_variants_token_stream = {
            let try_create_many_response_variants_stringified = format!("{try_camel_case_stringified}{create_many_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_create_many_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_many_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
                pub struct #create_many_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #create_many_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let payload_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        }); 
                    let field_type = &field.ty;
                    Some(quote::quote!{
                        pub #field_ident: #field_type
                    })
                },
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #create_many_payload_element_camel_case_token_stream {
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let try_create_many_error_named_token_stream = {
            let try_create_many_request_error_camel_case_token_stream = {
                let try_create_many_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{create_many_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_create_many_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_many_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_create_many_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_create_many_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_create_many_error_named_token_stream}");
        let http_request_token_stream = {
            let try_create_many_lower_case_token_stream = {
                let try_create_many_lower_case_stringified = format!("{try_lower_case_stringified}_{create_many_name_lower_case_stringified}");
                try_create_many_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_many_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{create_many_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let url_handle_token_stream = {
                let url_handle_stringified = format!("\"{{}}/{table_name_stringified}\"");
                url_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {url_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_create_many_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #create_many_parameters_camel_case_token_stream,
                ) -> Result<(), #try_create_many_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_create_many_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        #url_handle_token_stream,
                        #server_location_name_token_stream,
                    );
                    // println!("{}", url);
                    match #tvfrr_extraction_logic_token_stream(
                        #reqwest_client_new_token_stream
                        .post(&url)
                        #project_commit_header_addition_token_stream
                        #content_type_application_json_header_addition_token_stream
                        .body(#payload_lower_case_token_stream)
                        .send(),
                    )
                    .await
                    {
                        Ok(_) => Ok(()),
                        Err(e) => Err(#try_create_many_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let create_many_lower_case_token_stream = create_many_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_many_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let prepare_and_execute_query_token_stream = {
                let prepare_and_execute_query_error_token_stream = {
                    let error_path_stringified = format!("{try_camel_case_stringified}{create_many_name_camel_case_stringified}");
                    error_path_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &prepare_and_execute_query_error_token_stream,
                    &error_log_call_token_stream,
                    &try_create_many_response_variants_token_stream,
                );
                let query_string_token_stream = {
                    let column_names = {
                        let fields_named_filtered = fields_named.iter().filter(|field|*field != &id_field).collect::<Vec<&syn::Field>>();
                            let fields_named_len = fields_named_filtered.len();
                            fields_named_filtered.iter().enumerate().fold(std::string::String::default(), |mut acc, (index, field)| {
                                let field_ident = field.ident.clone().unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                            let incremented_index = index.checked_add(1).unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} {index} {}", proc_macro_helpers::global_variables::hardcode::CHECKED_ADD_NONE_OVERFLOW_MESSAGE));
                            match incremented_index == fields_named_len {
                                true => {
                                    acc.push_str(&format!("{field_ident}"));
                                },
                                false => {
                                    acc.push_str(&format!("{field_ident}{dot_space}"));
                                },
                            }
                            acc
                        })
                    };
                    let column_increments = {
                        let mut column_increments = fields_named.iter().filter(|field|*field != &id_field).enumerate().fold(std::string::String::default(), |mut acc, (index, _)| {
                            acc.push_str(&format!("${}, ", index.checked_add(1).unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} {index} {}", proc_macro_helpers::global_variables::hardcode::CHECKED_ADD_NONE_OVERFLOW_MESSAGE))));
                            acc
                        });
                        column_increments.pop();
                        column_increments.pop();
                        column_increments
                    };
                    let query_stringified = format!(
                        "\"{insert_name_stringified} {into_name_stringified} {table_name_stringified} ({column_names}) {select_name_stringified} {column_names} {from_name_stringified} {unnest_name_stringified}({column_increments}) {as_name_stringified} a({column_names})\""
                    );
                    query_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                // println!("{query_string_token_stream}");
                let binded_query_token_stream = {
                    let column_vecs_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                        true => None,
                        false => {
                            let field_ident_underscore_vec_token_stream = {
                                let field_ident_underscore_vec_stringified = {
                                    let field_ident = field.ident.clone()
                                        .unwrap_or_else(|| {
                                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                        });
                                    format!("{field_ident}{underscore_vec_name_stringified}")
                                };
                                field_ident_underscore_vec_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_underscore_vec_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            Some(field_ident_underscore_vec_token_stream)
                        },
                    });
                    let column_vecs_with_capacity_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                        true => None,
                        false => Some(quote::quote!{Vec::with_capacity(#current_vec_len_name_token_stream)}),
                    });
                    let columns_acc_push_elements_token_stream = fields_named.iter().filter(|field|*field != &id_field).enumerate().map(|(index, field)|{
                        let field_ident = field.ident.clone()
                            .unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                        let index_token_stream = {
                            let index_stringified = format!("{index}");
                            index_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {index_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{#acc_name_token_stream.#index_token_stream.push(#element_name_token_stream.#field_ident);}
                    });
                    let column_query_bind_vecs_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                        true => None,
                        false => {
                            let field_ident_underscore_vec_token_stream = {
                                let field_ident_underscore_vec_stringified = {
                                    let field_ident = field.ident.clone()
                                        .unwrap_or_else(|| {
                                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                        });
                                    format!("{field_ident}{underscore_vec_name_stringified}")
                                };
                                field_ident_underscore_vec_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_underscore_vec_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            Some(quote::quote!{#query_name_token_stream = #query_name_token_stream.bind(#field_ident_underscore_vec_token_stream);})
                        },
                    });
                    quote::quote!{
                        let mut #query_name_token_stream = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                        let #current_vec_len_name_token_stream = #parameters_lower_case_token_stream.#payload_lower_case_token_stream.len();
                        let (
                            #(#column_vecs_token_stream),*
                        ) = #parameters_lower_case_token_stream.#payload_lower_case_token_stream.into_iter().fold((
                            #(#column_vecs_with_capacity_token_stream),*
                        ), |mut #acc_name_token_stream, #element_name_token_stream| {
                            #(#columns_acc_push_elements_token_stream)*
                            #acc_name_token_stream
                        });
                        #(#column_query_bind_vecs_token_stream)*
                        #query_name_token_stream
                    }
                };
                // println!("{binded_query_token_stream}");
                let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                    &from_log_and_return_error_token_stream,
                    &pg_connection_token_stream
                );
                crate::generate_postgres_execute_query::generate_postgres_execute_query(
                    &query_string_name_token_stream,
                    &query_string_token_stream,
                    &binded_query_name_token_stream,
                    &binded_query_token_stream,
                    &acquire_pool_and_connection_token_stream,
                    &pg_connection_token_stream,
                    &try_create_many_response_variants_token_stream,
                    &desirable_token_stream,
                    &from_log_and_return_error_token_stream,
                )
            };
            // println!("{prepare_and_execute_query_token_stream}");
            quote::quote!{
                pub async fn #create_many_lower_case_token_stream(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#create_many_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #create_many_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #create_many_payload_camel_case_token_stream,
                            #try_create_many_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #prepare_and_execute_query_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #try_create_many_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{create_many_token_stream}");
    let create_one_token_stream = {
        let create_one_name_camel_case_stringified = "CreateOne";
        let create_one_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&create_one_name_camel_case_stringified.to_string());
        let create_one_parameters_camel_case_token_stream = {
            let create_one_parameters_camel_case_stringified = format!("{create_one_name_camel_case_stringified}{parameters_camel_case_stringified}");
            create_one_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_one_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let create_one_payload_camel_case_token_stream = {
            let create_one_payload_camel_case_stringified = format!("{create_one_name_camel_case_stringified}{payload_camel_case_stringified}");
            create_one_payload_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_one_payload_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_create_one_error_named_camel_case_token_stream = {
            let try_create_one_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{create_one_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_create_one_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_one_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_create_one_response_variants_token_stream = {
            let try_create_one_response_variants_stringified = format!("{try_camel_case_stringified}{create_one_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_create_one_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_one_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
                pub struct #create_one_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #create_one_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let payload_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    let field_type = &field.ty;
                    Some(quote::quote!{
                        pub #field_ident: #field_type
                    })
                },
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #create_one_payload_camel_case_token_stream {
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let try_create_one_error_named_token_stream = {
            let try_create_one_request_error_camel_case_token_stream = {
                let try_create_one_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{create_one_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_create_one_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_one_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_create_one_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_create_one_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_create_error_named_token_stream}");
        let http_request_token_stream = {
            let try_create_one_lower_case_token_stream = {
                let try_create_one_lower_case_stringified = format!("{try_lower_case_stringified}_{create_one_name_lower_case_stringified}");
                try_create_one_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_one_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{create_one_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let url_handle_token_stream = {
                let url_handle_stringified = format!("\"{{}}/{table_name_stringified}\"");
                url_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {url_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_create_one_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #create_one_parameters_camel_case_token_stream,
                ) -> Result<(), #try_create_one_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_create_one_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        #url_handle_token_stream,
                        #server_location_name_token_stream
                    );
                    // println!("{}", url);
                    match #tvfrr_extraction_logic_token_stream(
                        #reqwest_client_new_token_stream
                        .post(&url)
                        #project_commit_header_addition_token_stream
                        #content_type_application_json_header_addition_token_stream
                        .body(#payload_lower_case_token_stream)
                        .send(),
                    )
                    .await
                    {
                        Ok(_) => Ok(()),
                        Err(e) => Err(#try_create_one_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let create_one_lower_case_token_stream = create_one_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_one_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let prepare_and_execute_query_token_stream = {
                let prepare_and_execute_query_error_token_stream = {
                    let error_path_stringified = format!("{try_camel_case_stringified}{create_one_name_camel_case_stringified}");
                    error_path_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &prepare_and_execute_query_error_token_stream,
                    &error_log_call_token_stream,
                    &try_create_one_response_variants_token_stream,
                );
                let query_string_token_stream = {
                    let (
                        column_names,
                        column_increments
                    ) = {
                        let fields_named_filtered = fields_named.iter().filter(|field|*field != &id_field).collect::<Vec<&syn::Field>>();
                        let fields_named_len = fields_named_filtered.len();
                        fields_named_filtered.iter().enumerate().fold((
                            std::string::String::default(),
                            std::string::String::default()
                        ), |mut acc, (index, field)| {
                            let field_ident = field.ident.clone().unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                            let incremented_index = index.checked_add(1).unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} {index} {}", proc_macro_helpers::global_variables::hardcode::CHECKED_ADD_NONE_OVERFLOW_MESSAGE));
                            match incremented_index == fields_named_len {
                                true => {
                                    acc.0.push_str(&format!("{field_ident}"));
                                    acc.1.push_str(&format!("${incremented_index}"));
                                },
                                false => {
                                    acc.0.push_str(&format!("{field_ident}{dot_space}"));
                                    acc.1.push_str(&format!("${incremented_index}{dot_space}"));
                                },
                            }
                            acc
                        })
                    };
                    let query_stringified = format!("\"{insert_name_stringified} {into_name_stringified} {table_name_stringified}({column_names}) {values_name_stringified} ({column_increments})\"");
                    query_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                // println!("{query_string_token_stream}");
                let binded_query_token_stream = {
                    let binded_query_modifications_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                        true => None,
                        false => {
                            let field_ident = field.ident.clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                });
                            Some(quote::quote!{
                                query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(#parameters_lower_case_token_stream.#payload_lower_case_token_stream.#field_ident, query);
                            })
                        },
                    });
                    quote::quote!{
                        let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                        #(#binded_query_modifications_token_stream)*
                        query
                    }
                };
                // println!("{binded_query_token_stream}");
                let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                    &from_log_and_return_error_token_stream,
                    &pg_connection_token_stream
                );
                crate::generate_postgres_execute_query::generate_postgres_execute_query(
                    &query_string_name_token_stream,
                    &query_string_token_stream,
                    &binded_query_name_token_stream,
                    &binded_query_token_stream,
                    &acquire_pool_and_connection_token_stream,
                    &pg_connection_token_stream,
                    &try_create_one_response_variants_token_stream,
                    &desirable_token_stream,
                    &from_log_and_return_error_token_stream,
                )
            };
            // println!("{prepare_and_execute_query_token_stream}");
            quote::quote!{
                pub async fn #create_one_lower_case_token_stream(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#create_one_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #create_one_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #create_one_payload_camel_case_token_stream,
                            #try_create_one_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #prepare_and_execute_query_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #try_create_one_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{create_one_token_stream}");
    let read_one_token_stream = {
        let read_one_name_camel_case_stringified = "ReadOne";
        let read_one_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&read_one_name_camel_case_stringified.to_string());
        let read_one_parameters_camel_case_token_stream = {
            let read_one_parameters_camel_case_stringified = format!("{read_one_name_camel_case_stringified}{parameters_camel_case_stringified}");
            read_one_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_one_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let read_one_path_camel_case_token_stream = {
            let read_one_path_camel_case_stringified = format!("{read_one_name_camel_case_stringified}{path_camel_case_stringified}");
            read_one_path_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_one_path_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE)) 
        };
        let read_one_path_for_url_encoding_camel_case_token_stream = {
            let read_one_path_for_url_encoding_camel_case_stringified = format!("{read_one_name_camel_case_stringified}{path_camel_case_stringified}{for_url_encoding_camel_case_stringified}");
            read_one_path_for_url_encoding_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_one_path_for_url_encoding_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let read_one_path_try_from_read_one_path_for_url_encoding_camel_case_stringified = format!("{read_one_name_camel_case_stringified}{path_camel_case_stringified}TryFrom{read_one_name_camel_case_stringified}{path_camel_case_stringified}{for_url_encoding_camel_case_stringified}");
        let read_one_path_try_from_read_one_path_for_url_encoding_error_named_camel_case_token_stream = {//todo reuse TryFrom
            let read_one_path_try_from_read_one_path_for_url_encoding_error_named_camel_case_stringified = format!("{read_one_path_try_from_read_one_path_for_url_encoding_camel_case_stringified}{error_named_camel_case_stringified}");
            read_one_path_try_from_read_one_path_for_url_encoding_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_one_path_try_from_read_one_path_for_url_encoding_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let read_one_path_try_from_read_one_path_for_url_encoding_camel_case_token_stream = {//todo reuse TryFrom
            read_one_path_try_from_read_one_path_for_url_encoding_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_one_path_try_from_read_one_path_for_url_encoding_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let read_one_path_try_from_read_one_path_for_url_encoding_lower_case_token_stream = {//todo reuse TryFrom
            let read_one_path_try_from_read_one_path_for_url_encoding_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&read_one_path_try_from_read_one_path_for_url_encoding_camel_case_stringified.to_string());
            read_one_path_try_from_read_one_path_for_url_encoding_lower_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_one_path_try_from_read_one_path_for_url_encoding_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let read_one_query_camel_case_token_stream = {
            let read_one_query_camel_case_stringified = format!("{read_one_name_camel_case_stringified}{query_camel_case_stringified}");
            read_one_query_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_one_query_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let read_one_query_for_url_encoding_camel_case_token_stream = {
            let read_one_query_for_url_encoding_camel_case_stringified = format!("{read_one_name_camel_case_stringified}{query_camel_case_stringified}{for_url_encoding_camel_case_stringified}");
            read_one_query_for_url_encoding_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_one_query_for_url_encoding_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_read_one_error_named_camel_case_token_stream = {
            let try_read_one_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{read_one_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_read_one_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_one_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_read_one_response_variants_token_stream = {
            let try_read_one_response_variants_stringified = format!("{try_camel_case_stringified}{read_one_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_read_one_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_one_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #read_one_parameters_camel_case_token_stream {
                    pub #path_lower_case_token_stream: #read_one_path_camel_case_token_stream,
                    pub #query_lower_case_token_stream: #read_one_query_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let path_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #read_one_path_camel_case_token_stream {
                    pub #id_field_ident: crate::server::postgres::uuid_wrapper::UuidWrapper,
                }
            }
        };
        // println!("{path_token_stream}");
        let path_for_url_encoding_token_stream = {
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #read_one_path_for_url_encoding_camel_case_token_stream {
                    #id_field_ident: crate::server::postgres::uuid_wrapper::PossibleUuidWrapper,
                }
            }
        };
        // println!("{path_for_url_encoding_token_stream}");
        let read_one_path_try_from_read_one_path_for_url_encoding_error_named_token_stream = {
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #read_one_path_try_from_read_one_path_for_url_encoding_error_named_camel_case_token_stream {
                    NotUuid {
                        #eo_error_occurence_attribute_token_stream
                        not_uuid: crate::server::postgres::uuid_wrapper::UuidWrapperTryFromPossibleUuidWrapperErrorNamed,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                } 
            }
        };
        // println!("{read_one_path_try_from_read_one_path_for_url_encoding_error_named_token_stream}");
        let impl_std_convert_try_from_read_one_path_for_url_encoding_for_read_one_path_token_stream = {
            quote::quote!{
                impl std::convert::TryFrom<#read_one_path_for_url_encoding_camel_case_token_stream> for #read_one_path_camel_case_token_stream {
                    type Error = #read_one_path_try_from_read_one_path_for_url_encoding_error_named_camel_case_token_stream;
                    fn try_from(value: #read_one_path_for_url_encoding_camel_case_token_stream) -> Result<Self, Self::Error> {
                        let #id_field_ident = match crate::server::postgres::uuid_wrapper::UuidWrapper::try_from(value.#id_field_ident) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(Self::Error::NotUuid {
                                    not_uuid: e,
                                    #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                });
                            }
                        };
                        Ok(Self {
                            #id_field_ident
                        })
                    }
                }      
            }
        };
        // println!("{impl_std_convert_try_from_read_one_path_for_url_encoding_for_read_one_path_token_stream}");
        let query_token_stream = {
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #read_one_query_camel_case_token_stream {
                    pub #select_token_stream: Option<#column_select_ident_token_stream>,
                }
            }
        };
        // println!("{query_token_stream}");
        let query_for_url_encoding_token_stream = {
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                struct #read_one_query_for_url_encoding_camel_case_token_stream {
                    #select_token_stream: Option<std::string::String>,
                } 
            }
        };
        // println!("{query_for_url_encoding_token_stream}");
        let into_url_encoding_version_token_stream = {
            quote::quote!{
                impl #read_one_query_camel_case_token_stream {
                    fn #into_url_encoding_version_name_token_stream(self) -> #read_one_query_for_url_encoding_camel_case_token_stream {
                        let #select_token_stream = self.#select_token_stream.map(|value| {
                            #crate_common_serde_urlencoded_serde_urlencoded_parameter_serde_urlencoded_parameter_token_stream(
                                value,
                            )
                        });
                        #read_one_query_for_url_encoding_camel_case_token_stream {
                            #select_token_stream
                        }
                    }
                }
            }
        };
        // println!("{into_url_encoding_version_token_stream}");
        let try_read_one_error_named_token_stream = {
            let try_read_one_request_error_camel_case_token_stream = {
                let try_read_one_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{read_one_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_read_one_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_one_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_read_one_error_named_camel_case_token_stream {
                    #query_encode_variant_token_stream,
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_read_one_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{try_read_one_error_named_token_stream}");
        let http_request_token_stream = {
            let try_read_one_lower_case_token_stream = {
                let try_read_one_lower_case_stringified = format!("{try_lower_case_stringified}_{read_one_name_lower_case_stringified}");
                try_read_one_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_one_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{read_one_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let url_handle_token_stream = {
                let url_handle_stringified = format!("\"{{}}/{table_name_stringified}/{{}}?{{}}\"");//todo where
                url_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {url_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_read_one_lower_case_token_stream(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #read_one_parameters_camel_case_token_stream,
                ) -> Result<
                    #struct_options_ident_token_stream,
                    #try_read_one_error_named_camel_case_token_stream,
                > {
                    let encoded_query = match serde_urlencoded::to_string(#parameters_lower_case_token_stream.#query_lower_case_token_stream.#into_url_encoding_version_name_token_stream()) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_read_one_error_named_camel_case_token_stream::#query_encode_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        #url_handle_token_stream,
                        #server_location_name_token_stream,
                        #parameters_lower_case_token_stream.#path_lower_case_token_stream.id,
                        encoded_query
                    );
                    // println!("{}", url);
                    match #tvfrr_extraction_logic_token_stream(
                        #reqwest_client_new_token_stream
                        .get(&url)
                        #project_commit_header_addition_token_stream
                        .send(),
                    )
                    .await
                    {
                        Ok(value) => Ok(value),
                        Err(e) => Err(#try_read_one_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let read_one_lower_case_token_stream = read_one_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_one_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let prepare_and_execute_query_error_token_stream = {
                let error_path_stringified = format!("{try_camel_case_stringified}{read_one_name_camel_case_stringified}");
                error_path_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let prepare_and_execute_query_token_stream = {
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &prepare_and_execute_query_error_token_stream,
                    &error_log_call_token_stream,
                    &try_read_one_response_variants_token_stream,
                );
                let query_string_token_stream = {
                    let query_token_stream = {
                        let query_stringified = format!("\"{select_name_stringified} {{}} {from_name_stringified} {table_name_stringified} {where_name_stringified} {id_field_ident} = $1\"");
                        query_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    quote::quote!{
                        format!(
                            #query_token_stream,
                            crate::server::postgres::generate_query::GenerateQuery::generate_query(&#select_token_stream),
                        )
                    }
                };
                let binded_query_token_stream = {
                    let binded_query_modifications_token_stream = quote::quote!{
                        let query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(#parameters_lower_case_token_stream.#path_lower_case_token_stream.#id_field_ident, query);
                    };
                    quote::quote!{
                        let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                        #binded_query_modifications_token_stream
                        query
                    }
                };
                let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                    &from_log_and_return_error_token_stream,
                    &pg_connection_token_stream
                );
                quote::quote!{
                    let #select_token_stream = #parameters_lower_case_token_stream.#query_lower_case_token_stream.#select_token_stream.unwrap_or_default();
                    let #query_string_name_token_stream = {
                        #query_string_token_stream
                    };
                    println!("{}", #query_string_name_token_stream);
                    let #binded_query_name_token_stream = {
                        #binded_query_token_stream
                    };
                    #acquire_pool_and_connection_token_stream
                    match #binded_query_name_token_stream.fetch_one(#pg_connection_token_stream.as_mut()).await {
                        Ok(row) => match #select_token_stream.#options_try_from_sqlx_row_name_token_stream(&row) {
                            Ok(value) => #try_read_one_response_variants_token_stream::#desirable_token_stream(value),
                            Err(e) => {
                                #from_log_and_return_error_token_stream
                            },
                        },
                        Err(e) => {
                            #from_log_and_return_error_token_stream
                        },
                    }
                }
            };
            // println!("{prepare_and_execute_query_token_stream}");
            quote::quote!{
                pub async fn #read_one_lower_case_token_stream(
                    #path_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_path_token_stream<#read_one_path_for_url_encoding_camel_case_token_stream>,
                        #axum_extract_rejection_path_rejection_token_stream,
                    >,
                    #query_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_query_token_stream<#read_one_query_camel_case_token_stream>,
                        #axum_extract_rejection_query_rejection_token_stream,
                    >,
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #read_one_parameters_camel_case_token_stream {
                        #path_lower_case_token_stream: match #crate_server_routes_helpers_path_extractor_error_path_value_result_extractor_token_stream::<
                            #read_one_path_for_url_encoding_camel_case_token_stream,
                            #try_read_one_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#path_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => match #read_one_path_camel_case_token_stream::try_from(value) {
                                Ok(value) => value,
                                Err(e) => {
                                    let error = #prepare_and_execute_query_error_token_stream::#read_one_path_try_from_read_one_path_for_url_encoding_camel_case_token_stream {
                                        #read_one_path_try_from_read_one_path_for_url_encoding_lower_case_token_stream: e,
                                        #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                    };
                                    #error_log_call_token_stream
                                    return #try_read_one_response_variants_token_stream::from(error);
                                }
                            },
                            Err(err) => {
                                return err;
                            }
                        },
                        #query_lower_case_token_stream: match #crate_server_routes_helpers_query_extractor_error_query_value_result_extractor_token_stream::<
                            #read_one_query_camel_case_token_stream,
                            #try_read_one_response_variants_token_stream,
                        >::#try_extract_value_token_stream(
                            #query_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream
                        ) {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #prepare_and_execute_query_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #path_token_stream
            #path_for_url_encoding_token_stream
            #read_one_path_try_from_read_one_path_for_url_encoding_error_named_token_stream
            #impl_std_convert_try_from_read_one_path_for_url_encoding_for_read_one_path_token_stream
            #query_token_stream
            #query_for_url_encoding_token_stream
            #into_url_encoding_version_token_stream
            #try_read_one_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{read_one_token_stream}");
    let read_many_with_body_token_stream = {
        let read_many_with_body_name_camel_case_stringified = "ReadManyWithBody";
        let read_many_with_body_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&read_many_with_body_name_camel_case_stringified.to_string());
        let read_many_with_body_parameters_camel_case_token_stream = {
            let read_many_with_body_parameters_camel_case_stringified = format!("{read_many_with_body_name_camel_case_stringified}{parameters_camel_case_stringified}");
            read_many_with_body_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_many_with_body_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let read_many_with_body_payload_camel_case_token_stream = {
            let read_many_with_body_payload_camel_case_stringified = format!("{read_many_with_body_name_camel_case_stringified}{payload_camel_case_stringified}");
            read_many_with_body_payload_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_many_with_body_payload_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_read_many_with_body_error_named_camel_case_token_stream = {
            let try_read_many_with_body_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{read_many_with_body_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_read_many_with_body_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_many_with_body_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_read_many_with_body_response_variants_token_stream = {
            let try_read_many_with_body_response_variants_stringified = format!("{try_camel_case_stringified}{read_many_with_body_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_read_many_with_body_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_many_with_body_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
                pub struct #read_many_with_body_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #read_many_with_body_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let payload_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    Some(quote::quote!{
                        pub #field_ident: Option<Vec<crate::server::postgres::regex_filter::RegexFilter>>,
                    })
                },
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #read_many_with_body_payload_camel_case_token_stream {
                    pub #select_token_stream: #column_select_ident_token_stream,
                    pub #id_field_ident: Option<Vec<std::string::String>>,//crate::server::postgres::bigserial::Bigserial todo uuid builder
                    #(#fields_with_excluded_id_token_stream)*
                    pub #order_by_token_stream: #crate_server_postgres_order_by_order_by_token_stream<#column_ident_token_stream>,
                    pub limit: crate::server::postgres::postgres_bigint::PostgresBigint,
                    pub offset: crate::server::postgres::postgres_bigint::PostgresBigint,
                }
            }
        };
        // println!("{payload_token_stream}");
        let try_read_many_with_body_error_named_token_stream = {
            let try_read_many_with_body_request_error_camel_case_token_stream = {
                let try_read_many_with_body_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{read_many_with_body_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_read_many_with_body_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_many_with_body_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_read_many_with_body_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_read_many_with_body_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_read_many_with_body_error_named_token_stream}");
        let http_request_token_stream = {
            let try_read_many_with_body_lower_case_token_stream = {
                let try_read_many_with_body_lower_case_stringified = format!("{try_lower_case_stringified}_{read_many_with_body_name_lower_case_stringified}");
                try_read_many_with_body_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_many_with_body_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{read_many_with_body_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let url_handle_token_stream = {
                let url_handle_stringified = format!("\"{{}}/{table_name_stringified}/search\"");//todo where
                url_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {url_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_read_many_with_body_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #read_many_with_body_parameters_camel_case_token_stream,
                ) -> Result<
                    Vec<#struct_options_ident_token_stream>,
                    #try_read_many_with_body_error_named_camel_case_token_stream,
                > {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_read_many_with_body_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        #url_handle_token_stream ,
                        #server_location_name_token_stream
                    );
                    // println!("{}", url);
                    match #tvfrr_extraction_logic_token_stream(
                        #reqwest_client_new_token_stream
                        .post(&url)
                        #project_commit_header_addition_token_stream
                        #content_type_application_json_header_addition_token_stream
                        .body(#payload_lower_case_token_stream)
                        .send(),
                    )
                    .await
                    {
                        Ok(value) => Ok(value),
                        Err(e) => Err(#try_read_many_with_body_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let read_many_with_body_lower_case_token_stream = read_many_with_body_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_many_with_body_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let prepare_and_execute_query_token_stream = {
                let prepare_and_execute_query_error_token_stream = {
                    let error_path_stringified = format!("{try_camel_case_stringified}{read_many_with_body_name_camel_case_stringified}");
                    error_path_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &prepare_and_execute_query_error_token_stream,
                    &error_log_call_token_stream,
                    &try_read_many_with_body_response_variants_token_stream,
                );
                let filter_unique_parameters_token_stream = {
                    let filter_unique_parameters_primary_key_token_stream = quote::quote!{
                        if let Some(#id_field_ident) = &#parameters_lower_case_token_stream.#payload_lower_case_token_stream.#id_field_ident {
                            let #not_unique_primary_keys_name_token_stream = {
                                let mut vec = Vec::with_capacity(#id_field_ident.len());
                                let mut #not_unique_primary_keys_name_token_stream = Vec::with_capacity(#id_field_ident.len());
                                for element in #id_field_ident {
                                    let handle = element;
                                    match vec.contains(&handle) {
                                        true => {
                                            #not_unique_primary_keys_name_token_stream.push(element.clone());
                                        },
                                        false => {
                                            vec.push(element);
                                        }
                                    }
                                }
                                #not_unique_primary_keys_name_token_stream
                            };
                            if let false = #not_unique_primary_keys_name_token_stream.is_empty() {
                                let error = #prepare_and_execute_query_error_token_stream::#not_unique_primery_key_token_stream;
                                #error_log_call_token_stream
                                return #try_read_many_with_body_response_variants_token_stream::from(error);
                            }
                        }
                    };
                    let filter_unique_parameters_other_columns_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                        true => None,
                        false => {
                            let field_ident = field.ident.clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                });
                            let field_handle_token_stream = {
                                let field_handle_stringified = format!("{field_ident}_handle");
                                field_handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            let not_unique_field_vec_lower_case_token_stream = {
                                let not_unique_field_vec_lower_case_stringified = format!("not_unique_{field_ident}_vec");
                                not_unique_field_vec_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {not_unique_field_vec_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            let not_unique_field_vec_vec_pascal_token_stream = {
                                let not_unique_field_vec_pascal_stringified = format!(
                                    "NotUnique{}Vec",
                                    {
                                        use convert_case::Casing;
                                        field_ident.to_string().to_case(convert_case::Case::Pascal)
                                    }
                                );
                                not_unique_field_vec_pascal_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {not_unique_field_vec_pascal_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            Some(quote::quote!{
                                let #field_handle_token_stream = match #parameters_lower_case_token_stream.#payload_lower_case_token_stream.#field_ident {
                                    Some(value) => {
                                        let is_unique = {
                                            let mut vec = Vec::with_capacity(value.len());
                                            let mut is_unique = true;
                                            for element in &value {
                                                match vec.contains(&element) {
                                                    true => {
                                                        is_unique = false;
                                                        break;
                                                    }
                                                    false => {
                                                        vec.push(element);
                                                    }
                                                }
                                            }
                                            is_unique
                                        };
                                        match is_unique {
                                            true => Some(value),
                                            false => {
                                                let #not_unique_field_vec_lower_case_token_stream = {
                                                    let mut vec = Vec::with_capacity(value.len());
                                                    let mut #not_unique_field_vec_lower_case_token_stream = Vec::with_capacity(value.len());
                                                    for element in value {
                                                        match vec.contains(&element) {
                                                            true => {
                                                                #not_unique_field_vec_lower_case_token_stream.push(element);
                                                            }
                                                            false => {
                                                                vec.push(element);
                                                            }
                                                        }
                                                    }
                                                    #not_unique_field_vec_lower_case_token_stream
                                                };
                                                let error = #prepare_and_execute_query_error_token_stream::#not_unique_field_vec_vec_pascal_token_stream {
                                                    #not_unique_field_vec_lower_case_token_stream,
                                                    #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                                };
                                                #error_log_call_token_stream
                                                return #try_read_many_with_body_response_variants_token_stream::from(error);
                                            }
                                        }
                                    }
                                    None => None,
                                };
                            })
                        },
                    });
                    quote::quote!{
                        #filter_unique_parameters_primary_key_token_stream
                        #(#filter_unique_parameters_other_columns_token_stream)*
                    }
                };
                let query_string_token_stream = {
                    let additional_parameters_id_modification_token_stream = {
                        let prefix_false_handle_token_stream = {
                            let prefix_false_handle_stringified = format!("\" {and_name_stringified}\"");
                            prefix_false_handle_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {prefix_false_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        let handle_token_stream = {
                            let handle_stringified = format!("\"{{}} {id_field_ident} {in_name_stringified} ({select_name_stringified} {unnest_name_stringified}(${{}}))\"");
                            handle_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{
                            if let Some(value) = &#parameters_lower_case_token_stream.#payload_lower_case_token_stream.#id_field_ident {
                                let prefix = match additional_parameters.is_empty() {
                                    true => #where_name_qoutes_token_stream,
                                    false => #prefix_false_handle_token_stream,
                                };
                                match increment.checked_add(1) {
                                    Some(value) => {
                                        increment = value;
                                    },
                                    None => {
                                        //todo - think what to do with TryGenerateBindIncrementsErrorNamed and how handle it 
                                        let e = crate::server::postgres::bind_query::TryGenerateBindIncrementsErrorNamed::CheckedAdd { 
                                            checked_add: std::string::String::from("checked_add is None"), 
                                            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream, 
                                        };
                                        return #try_read_many_with_body_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                    },
                                }
                                additional_parameters.push_str(&format!(
                                    #handle_token_stream,
                                    prefix,
                                    increment
                                ));
                            }
                        }
                    };
                    let additional_parameters_modification_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                        true => None,
                        false => {
                            let field_ident = field.ident.clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                });
                            let handle_token_stream = {
                                let handle_stringified = format!("\"{field_ident} ~ {{value}} \"");
                                handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            let prefix_false_handle_token_stream = {
                                let prefix_false_handle_stringified = format!("\" {and_name_stringified}\"");
                                prefix_false_handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {prefix_false_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            let field_handle_token_stream = {
                                let field_handle_stringified = format!("{field_ident}_handle");
                                field_handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            Some(quote::quote!{
                                if let Some(value) = &#field_handle_token_stream {
                                    let prefix = match additional_parameters.is_empty() {
                                        true => #where_name_qoutes_token_stream,
                                        false => #prefix_false_handle_token_stream,
                                    };
                                    let bind_increments = {
                                        let mut bind_increments = std::string::String::default();
                                        for (index, element) in value.iter().enumerate() {
                                            match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(
                                                element,
                                                &mut increment
                                            ) {
                                                Ok(value) => {
                                                    let handle = format!(#handle_token_stream);
                                                    match index == 0 {
                                                        true => {
                                                            bind_increments.push_str(&handle);
                                                        },
                                                        false => {
                                                            bind_increments.push_str(&format!("{} {handle}", element.conjuctive_operator));
                                                        },
                                                    }
                                                },
                                                Err(e) => {
                                                    return #try_read_many_with_body_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                                },
                                            }
                                        }
                                        if let false = bind_increments.is_empty() {
                                            bind_increments.pop();
                                        }
                                        bind_increments
                                    };
                                    additional_parameters.push_str(&format!("{prefix} {bind_increments}"));
                                }
                            })
                        },
                    });
                    let handle_token_stream = {
                        let handle_stringified = format!("\"{select_name_stringified} {{}} {from_name_stringified} {table_name_stringified} {{}}\"");
                        handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    let additional_parameters_order_by_handle_token_stream = {
                        let additional_parameters_order_by_handle_stringified = format!("\"{{}}{order_by_name_stringified} {{}} {{}}\"");
                        additional_parameters_order_by_handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {additional_parameters_order_by_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    let additional_parameters_limit_handle_token_stream = {
                        let additional_parameters_limit_handle_stringified = format!("\"{{}}{limit_name_stringified} {{}}\"");
                        additional_parameters_limit_handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {additional_parameters_limit_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    let additional_parameters_offset_handle_token_stream = {
                        let additional_parameters_offset_handle_stringified = format!("\"{{}}{offset_name_stringified} {{}}\"");
                        additional_parameters_offset_handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {additional_parameters_offset_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    quote::quote!{
                        format!(
                            #handle_token_stream,
                            crate::server::postgres::generate_query::GenerateQuery::generate_query(
                                &#parameters_lower_case_token_stream.#payload_lower_case_token_stream.#select_token_stream
                            ),
                            {
                                #increment_initialization_token_stream
                                let mut additional_parameters = std::string::String::default();
                                #additional_parameters_id_modification_token_stream
                                #(#additional_parameters_modification_token_stream)*
                                {
                                    let prefix = match additional_parameters.is_empty() {
                                        true => "",
                                        false => " ",
                                    };
                                    let value = &#parameters_lower_case_token_stream.#payload_lower_case_token_stream.#order_by_token_stream;
                                    let order_stringified = match &value.order {
                                        Some(order) => order.to_string(),
                                        None => crate::server::postgres::order::Order::default().to_string(),
                                    };
                                    additional_parameters.push_str(&format!(
                                        #additional_parameters_order_by_handle_token_stream,
                                        prefix,
                                        value.column,
                                        order_stringified
                                    ));
                                }
                                {
                                    let prefix = match additional_parameters.is_empty() {
                                        true => "",
                                        false => " ",
                                    };
                                    let value = match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(
                                        &#parameters_lower_case_token_stream.#payload_lower_case_token_stream.limit,
                                        &mut increment
                                    ) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return #try_read_many_with_body_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                        },
                                    };
                                    additional_parameters.push_str(&format!(
                                        #additional_parameters_limit_handle_token_stream,
                                        prefix,
                                        value
                                    ));
                                }
                                {
                                    let prefix = match additional_parameters.is_empty() {
                                        true => "",
                                        false => " ",
                                    };
                                    let value = match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(
                                        &#parameters_lower_case_token_stream.#payload_lower_case_token_stream.offset,
                                        &mut increment
                                    ) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return #try_read_many_with_body_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                        },
                                    };
                                    additional_parameters.push_str(&format!(
                                        #additional_parameters_offset_handle_token_stream,
                                        prefix,
                                        value
                                    ));
                                }
                                additional_parameters
                            }
                        )
                    }
                };
                let binded_query_token_stream = {
                    let binded_query_id_modification_token_stream = quote::quote!{
                        if let Some(value) = #parameters_lower_case_token_stream.#payload_lower_case_token_stream.#id_field_ident {
                            let mut vec_for_bind = Vec::with_capacity(value.len());
                            for element in value {
                                match sqlx::types::Uuid::parse_str(&element) {
                                    Ok(value) => {
                                        vec_for_bind.push(value);
                                    }
                                    Err(e) => {
                                        let error = #prepare_and_execute_query_error_token_stream::NotUuid {
                                            not_uuid: e,
                                            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                        };
                                        #error_log_call_token_stream
                                        return #try_read_many_with_body_response_variants_token_stream::from(error);
                                    }
                                }
                            }
                            query = query.bind(vec_for_bind);
                        }
                    };
                    let binded_query_modifications_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                        true => None,
                        false => {
                            let field_ident = field.ident.clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                });
                            let field_handle_token_stream = {
                                let field_handle_stringified = format!("{field_ident}_handle");
                                field_handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            Some(quote::quote!{
                                if let Some(values) = #field_handle_token_stream {
                                    for value in values {
                                        query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                                            value, query,
                                        );
                                    }
                                }
                            })
                        },
                    }); 
                    quote::quote!{
                        let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                        #binded_query_id_modification_token_stream
                        #(#binded_query_modifications_token_stream)*
                        query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                            #parameters_lower_case_token_stream.#payload_lower_case_token_stream.limit,
                            query,
                        );
                        query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                            #parameters_lower_case_token_stream.#payload_lower_case_token_stream.offset,
                            query,
                        );
                        query
                    }
                };
                let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                    &from_log_and_return_error_token_stream,
                    &pg_connection_token_stream
                );
                quote::quote!{
                    #filter_unique_parameters_token_stream                    
                    let #query_string_name_token_stream = {
                        #query_string_token_stream
                    };
                    println!("{}", #query_string_name_token_stream);
                    let #binded_query_name_token_stream = {
                        #binded_query_token_stream
                    };
                    let vec_values = {
                        #acquire_pool_and_connection_token_stream
                        let mut rows = #binded_query_name_token_stream.fetch(#pg_connection_token_stream.as_mut());
                        let mut vec_values = Vec::new();
                        while let Some(row) = {
                            match {
                                #use_futures_try_stream_ext_token_stream;
                                rows.try_next()
                            }
                            .await
                            {
                                Ok(value) => value,
                                Err(e) => {
                                    #from_log_and_return_error_token_stream;
                                }
                            }
                        } {
                            match #parameters_lower_case_token_stream.#payload_lower_case_token_stream.#select_token_stream.#options_try_from_sqlx_row_name_token_stream(&row) {
                                Ok(value) => {
                                    vec_values.push(value);
                                }
                                Err(e) => {
                                    #from_log_and_return_error_token_stream;
                                }
                            }
                        }
                        vec_values
                    };
                    #try_read_many_with_body_response_variants_token_stream::#desirable_token_stream(vec_values)
                }
            };
            // println!("{prepare_and_execute_query_token_stream}");
            quote::quote!{
                pub async fn #read_many_with_body_lower_case_token_stream(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#read_many_with_body_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #read_many_with_body_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #read_many_with_body_payload_camel_case_token_stream,
                            #try_read_many_with_body_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #prepare_and_execute_query_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #try_read_many_with_body_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{read_many_with_body_token_stream}");
    let read_many_token_stream = {
        let read_many_name_camel_case_stringified = "ReadMany";
        let read_many_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&read_many_name_camel_case_stringified.to_string());
        let read_many_parameters_camel_case_token_stream = {
            let read_many_parameters_camel_case_stringified = format!("{read_many_name_camel_case_stringified}{parameters_camel_case_stringified}");
            read_many_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_many_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let read_many_query_camel_case_token_stream = {
            let read_many_query_camel_case_stringified = format!("{read_many_name_camel_case_stringified}{query_camel_case_stringified}");
            read_many_query_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_many_query_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE)) 
        };
        let read_many_query_for_url_encoding_camel_case_token_stream = {
            let read_many_query_for_url_encoding_camel_case_stringified = format!("{read_many_name_camel_case_stringified}{query_camel_case_stringified}{for_url_encoding_camel_case_stringified}");
            read_many_query_for_url_encoding_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_many_query_for_url_encoding_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE)) 
        };
        let try_read_many_error_named_camel_case_token_stream = {
            let try_read_many_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{read_many_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_read_many_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_many_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_read_many_response_variants_token_stream = {
            let try_read_many_response_variants_stringified = format!("{try_camel_case_stringified}{read_many_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_read_many_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_many_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_deserialize_token_stream
                pub struct #read_many_parameters_camel_case_token_stream {
                    pub #query_lower_case_token_stream: #read_many_query_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let query_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    Some(quote::quote!{
                        pub #field_ident: Option<crate::server::routes::helpers::strings_deserialized_from_string_splitted_by_comma::StringsDeserializedFromStringSplittedByComma>,
                    })
                },
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #read_many_query_camel_case_token_stream {
                    pub #select_token_stream: Option<#column_select_ident_token_stream>,
                    pub #id_field_ident: Option<Vec<std::string::String>>,//crate::server::postgres::bigserial_ids::BigserialIds
                    #(#fields_with_excluded_id_token_stream)*
                    pub #order_by_token_stream: Option<#ident_order_by_wrapper_token_stream>,//todo
                    pub limit: crate::server::postgres::postgres_bigint::PostgresBigint,
                    pub offset: Option<crate::server::postgres::postgres_bigint::PostgresBigint>,
                }
            }
        };
        // println!("{query_token_stream}");
        let query_for_url_encoding_token_stream = {
            let fields_for_url_encoding_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    Some(quote::quote!{
                        pub #field_ident: Option<std::string::String>,
                    })
                },
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                struct #read_many_query_for_url_encoding_camel_case_token_stream {
                    #select_token_stream: Option<std::string::String>,
                    pub #id_field_ident: Option<std::string::String>,
                    #(#fields_for_url_encoding_with_excluded_id_token_stream)*
                    #order_by_token_stream: Option<std::string::String>,
                    limit: std::string::String,
                    offset: Option<std::string::String>,
                }
            }
        };
        // println!("{query_for_url_encoding_token_stream}");
        let into_url_encoding_version_token_stream = {
            let fields_into_url_encoding_version_with_excluded_id_token_stream = fields_named.iter().map(|field| {
                let field_ident = field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    let #field_ident = self.#field_ident.map(|value| {
                        #crate_common_serde_urlencoded_serde_urlencoded_parameter_serde_urlencoded_parameter_token_stream(
                            value,
                        )
                    });
                }
            });
            let fields_into_url_encoding_version_constract_with_excluded_id_token_stream = fields_named.iter().map(|field|{
                let field_ident = field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    #field_ident,
                }
            });
            quote::quote!{
                impl #read_many_query_camel_case_token_stream {
                    fn #into_url_encoding_version_name_token_stream(self) -> #read_many_query_for_url_encoding_camel_case_token_stream {
                        let #select_token_stream = self.#select_token_stream.map(|value| {
                            #crate_common_serde_urlencoded_serde_urlencoded_parameter_serde_urlencoded_parameter_token_stream(
                                value,
                            )
                        });
                        #(#fields_into_url_encoding_version_with_excluded_id_token_stream)*
                        let #order_by_token_stream = self.#order_by_token_stream.map(|value| {
                            #crate_common_serde_urlencoded_serde_urlencoded_parameter_serde_urlencoded_parameter_token_stream(
                                value,
                            )
                        });
                        let limit = #crate_common_serde_urlencoded_serde_urlencoded_parameter_serde_urlencoded_parameter_token_stream(
                            self.limit,
                        );
                        let offset = self.offset.map(|value| {
                            #crate_common_serde_urlencoded_serde_urlencoded_parameter_serde_urlencoded_parameter_token_stream(
                                value,
                            )
                        });
                        #read_many_query_for_url_encoding_camel_case_token_stream {
                            #select_token_stream,
                            #(#fields_into_url_encoding_version_constract_with_excluded_id_token_stream)*
                            #order_by_token_stream,
                            limit,
                            offset,
                        }
                    }
                }
            }
        };
        // println!("{into_url_encoding_version_token_stream}");
        let try_read_many_error_named_token_stream = {
            let try_read_many_request_error_camel_case_token_stream = {
                let try_read_many_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{read_many_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_read_many_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_many_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_read_many_error_named_camel_case_token_stream {
                    #query_encode_variant_token_stream,
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_read_many_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{try_read_many_error_named_token_stream}");
        let http_request_token_stream = {
            let try_read_many_lower_case_token_stream = {
                let try_read_many_lower_case_stringified = format!("{try_lower_case_stringified}_{read_many_name_lower_case_stringified}");
                try_read_many_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_many_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{read_many_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let url_handle_token_stream = {
                let url_handle_stringified = format!("\"{{}}/{table_name_stringified}?{{}}\"");//todo where
                url_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {url_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_read_many_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #read_many_parameters_camel_case_token_stream,
                ) -> Result<
                    Vec<#struct_options_ident_token_stream>,
                    #try_read_many_error_named_camel_case_token_stream,
                > {
                    let encoded_query = match serde_urlencoded::to_string(#parameters_lower_case_token_stream.#query_lower_case_token_stream.#into_url_encoding_version_name_token_stream()) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_read_many_error_named_camel_case_token_stream::#query_encode_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        #url_handle_token_stream,
                        #server_location_name_token_stream,
                        encoded_query
                    );
                    // println!("{}", url);
                    match #tvfrr_extraction_logic_token_stream(
                        #reqwest_client_new_token_stream
                        .get(&url)
                        #project_commit_header_addition_token_stream
                        .send(),
                    )
                    .await
                    {
                        Ok(value) => Ok(value),
                        Err(e) => Err(#try_read_many_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let read_many_lower_case_token_stream = read_many_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_many_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let prepare_and_execute_query_token_stream = {
                let prepare_and_execute_query_error_token_stream = {
                    let error_path_stringified = format!("{try_camel_case_stringified}{read_many_name_camel_case_stringified}");
                    error_path_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let filter_unique_parameters_token_stream = {
                    let filter_unique_parameters_primary_key_token_stream = quote::quote!{
                        if let Some(#id_field_ident) = &#parameters_lower_case_token_stream.#query_lower_case_token_stream.#id_field_ident {
                            let #not_unique_primary_keys_name_token_stream = {
                                let mut vec = Vec::with_capacity(#id_field_ident.len());
                                let mut #not_unique_primary_keys_name_token_stream = Vec::with_capacity(#id_field_ident.len());
                                for element in #id_field_ident {
                                    let handle = element;
                                    match vec.contains(&handle) {
                                        true => {
                                            #not_unique_primary_keys_name_token_stream.push(element.clone());
                                        },
                                        false => {
                                            vec.push(element);
                                        }
                                    }
                                }
                                #not_unique_primary_keys_name_token_stream
                            };
                            if let false = #not_unique_primary_keys_name_token_stream.is_empty() {
                                let error = #prepare_and_execute_query_error_token_stream::#not_unique_primery_key_token_stream;
                                #error_log_call_token_stream
                                return #try_read_many_response_variants_token_stream::from(error);
                            }
                        }
                    };
                    let filter_unique_parameters_other_columns_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                        true => None,
                        false => {
                            let field_ident = field.ident.clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                });
                            let field_handle_token_stream = {
                                let field_handle_stringified = format!("{field_ident}_handle");
                                field_handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            let not_unique_field_vec_lower_case_token_stream = {
                                let not_unique_field_vec_lower_case_stringified = format!("not_unique_{field_ident}_vec");
                                not_unique_field_vec_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {not_unique_field_vec_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            let not_unique_field_vec_vec_pascal_token_stream = {
                                let not_unique_field_vec_pascal_stringified = format!(
                                    "NotUnique{}Vec",
                                    {
                                        use convert_case::Casing;
                                        field_ident.to_string().to_case(convert_case::Case::Pascal)
                                    }
                                );
                                not_unique_field_vec_pascal_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {not_unique_field_vec_pascal_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            Some(quote::quote!{
                                let #field_handle_token_stream = match #parameters_lower_case_token_stream.#query_lower_case_token_stream.#field_ident {
                                    Some(value) => {
                                        let is_unique = {
                                            let mut vec = Vec::with_capacity(value.0.len());
                                            let mut is_unique = true;
                                            for element in &value.0 {
                                                match vec.contains(&element) {
                                                    true => {
                                                        is_unique = false;
                                                        break;
                                                    }
                                                    false => {
                                                        vec.push(element);
                                                    }
                                                }
                                            }
                                            is_unique
                                        };
                                        match is_unique {
                                            true => Some(value),
                                            false => {
                                                let #not_unique_field_vec_lower_case_token_stream = {
                                                    let mut vec = Vec::with_capacity(value.0.len());
                                                    let mut #not_unique_field_vec_lower_case_token_stream = Vec::with_capacity(value.0.len());
                                                    for element in value.0 {
                                                        match vec.contains(&element) {
                                                            true => {
                                                                #not_unique_field_vec_lower_case_token_stream.push(element);
                                                            }
                                                            false => {
                                                                vec.push(element);
                                                            }
                                                        }
                                                    }
                                                    #not_unique_field_vec_lower_case_token_stream
                                                };
                                                let error = #prepare_and_execute_query_error_token_stream::#not_unique_field_vec_vec_pascal_token_stream {
                                                    #not_unique_field_vec_lower_case_token_stream,
                                                    #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                                };
                                                #error_log_call_token_stream
                                                return #try_read_many_response_variants_token_stream::from(error);
                                            }
                                        }
                                    }
                                    None => None,
                                };
                            })
                        },
                    });
                    quote::quote!{
                        #filter_unique_parameters_primary_key_token_stream
                        #(#filter_unique_parameters_other_columns_token_stream)*
                    }
                };
                let query_string_token_stream = {
                    let additional_parameters_id_modification_token_stream = {
                        let prefix_false_handle_token_stream = {
                            let prefix_false_handle_stringified = format!("\" {and_name_stringified}\"");
                            prefix_false_handle_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {prefix_false_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        let handle_token_stream = {
                            let handle_stringified = format!("\"{{}} {id_field_ident} {in_name_stringified} ({select_name_stringified} {unnest_name_stringified}(${{}}))\"");
                            handle_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{
                            if let Some(value) = &#parameters_lower_case_token_stream.#query_lower_case_token_stream.#id_field_ident {
                                let prefix = match additional_parameters.is_empty() {
                                    true => #where_name_qoutes_token_stream,
                                    false => #prefix_false_handle_token_stream,
                                };
                                match increment.checked_add(1) {
                                    Some(value) => {
                                        increment = value;
                                    },
                                    None => {
                                        //todo - think what to do with TryGenerateBindIncrementsErrorNamed and how handle it 
                                        let e = crate::server::postgres::bind_query::TryGenerateBindIncrementsErrorNamed::CheckedAdd { 
                                            checked_add: std::string::String::from("checked_add is None"), 
                                            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream, 
                                        };
                                        return #try_read_many_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                    },
                                }
                                additional_parameters.push_str(&format!(
                                    #handle_token_stream,
                                    prefix,
                                    increment
                                ));
                            }
                        }
                    };
                    let additional_parameters_modification_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                        true => None,
                        false => {
                            let field_ident = field.ident.clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                });
                            let handle_token_stream = {
                                let handle_stringified = format!("\"{{prefix}} {field_ident} = {any_name_stringified}({array_name_stringified}[{{value}}])\"");
                                handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            let prefix_false_handle_token_stream = {
                                let prefix_false_handle_stringified = format!("\" {and_name_stringified}\"");
                                prefix_false_handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {prefix_false_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            let field_handle_token_stream = {
                                let field_handle_stringified = format!("{field_ident}_handle");
                                field_handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            Some(quote::quote!{
                                if let Some(value) = &#field_handle_token_stream {
                                    let prefix = match additional_parameters.is_empty() {
                                        true => #where_name_qoutes_token_stream,
                                        false => #prefix_false_handle_token_stream,
                                    };
                                    let value = match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(
                                        value,
                                        &mut increment
                                    ) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return #try_read_many_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                        },
                                    };
                                    additional_parameters.push_str(&format!(#handle_token_stream));
                                }
                            })
                        },
                    });
                    let handle_token_stream = {
                        let handle_stringified = format!("\"{select_name_stringified} {{}} {from_name_stringified} {table_name_stringified} {{}}\"");
                        handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    let additional_parameters_order_by_handle_token_stream = {
                        let additional_parameters_order_by_handle_stringified = format!("\"{{}}{order_by_name_stringified} {{}} {{}}\"");
                        additional_parameters_order_by_handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {additional_parameters_order_by_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    let additional_parameters_limit_handle_token_stream = {
                        let additional_parameters_limit_handle_stringified = format!("\"{{}}{limit_name_stringified} {{}}\"");
                        additional_parameters_limit_handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {additional_parameters_limit_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    let additional_parameters_offset_handle_token_stream = {
                        let additional_parameters_offset_handle_stringified = format!("\"{{}}{offset_name_stringified} {{}}\"");
                        additional_parameters_offset_handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {additional_parameters_offset_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    quote::quote!{
                        format!(
                            #handle_token_stream,
                            crate::server::postgres::generate_query::GenerateQuery::generate_query(&#select_token_stream),
                            {
                                #increment_initialization_token_stream
                                let mut additional_parameters = std::string::String::default();
                                #additional_parameters_id_modification_token_stream
                                #(#additional_parameters_modification_token_stream)*
                                if let Some(value) = &#parameters_lower_case_token_stream.#query_lower_case_token_stream.#order_by_token_stream {
                                    let prefix = match additional_parameters.is_empty() {
                                        true => "",
                                        false => " ",
                                    };
                                    let order_stringified = match &value.0.order {
                                        Some(order) => order.to_string(),
                                        None => crate::server::postgres::order::Order::default().to_string(),
                                    };
                                    additional_parameters.push_str(&format!(
                                        #additional_parameters_order_by_handle_token_stream,
                                        prefix,
                                        value.0.column,
                                        order_stringified
                                    ));
                                }
                                {
                                    let prefix = match additional_parameters.is_empty() {
                                        true => "",
                                        false => " ",
                                    };
                                    let value = match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(
                                        &#parameters_lower_case_token_stream.#query_lower_case_token_stream.limit,
                                        &mut increment
                                    ) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return #try_read_many_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                        },
                                    };
                                    additional_parameters.push_str(&format!(
                                        #additional_parameters_limit_handle_token_stream,
                                        prefix,
                                        value
                                    ));
                                }
                                if let Some(value) = &#parameters_lower_case_token_stream.#query_lower_case_token_stream.offset {
                                    let prefix = match additional_parameters.is_empty() {
                                        true => "",
                                        false => " ",
                                    };
                                    let value = match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(
                                        value,
                                        &mut increment
                                    ) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            return #try_read_many_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                        },
                                    };
                                    additional_parameters.push_str(&format!(
                                        #additional_parameters_offset_handle_token_stream,
                                        prefix,
                                        value
                                    ));
                                }
                                additional_parameters
                            }
                        )
                    }
                };
                let binded_query_token_stream = {
                    let binded_query_id_modification_token_stream = {
                        quote::quote!{
                            if let Some(value) = #parameters_lower_case_token_stream.#query_lower_case_token_stream.#id_field_ident {
                                query = query.bind(
                                    value
                                    .into_iter()
                                    .map(|element| element.clone())
                                    .collect::<Vec<#id_field_type>>()
                                );
                            }
                        }
                    };
                    let binded_query_modifications_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                        true => None,
                        false => {
                            let field_ident = field.ident.clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                });
                            let field_handle_token_stream = {
                                let field_handle_stringified = format!("{field_ident}_handle");
                                field_handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            Some(quote::quote!{
                                if let Some(value) = #field_handle_token_stream {
                                    query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                                        value, query,
                                    );
                                }
                            })
                        },
                    });
                    quote::quote!{
                        let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                        #binded_query_id_modification_token_stream
                        #(#binded_query_modifications_token_stream)*
                        query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                            #parameters_lower_case_token_stream.#query_lower_case_token_stream.limit,
                            query,
                        );
                        if let Some(value) = #parameters_lower_case_token_stream.#query_lower_case_token_stream.offset {
                            query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                                value, query,
                            );
                        }
                        query
                    }
                };
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &prepare_and_execute_query_error_token_stream,
                    &error_log_call_token_stream,
                    &try_read_many_response_variants_token_stream,
                );
                let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                    &from_log_and_return_error_token_stream,
                    &pg_connection_token_stream
                );
                quote::quote!{
                    #filter_unique_parameters_token_stream
                    //todo select_token_stream in read_many_with_body and in read_many are not in the same place
                    let #select_token_stream = #column_select_ident_token_stream::from(#parameters_lower_case_token_stream.#query_lower_case_token_stream.#select_token_stream.clone());
                    let #query_string_name_token_stream = {
                        #query_string_token_stream
                    };
                    println!("{}", #query_string_name_token_stream);
                    let #binded_query_name_token_stream = {
                        #binded_query_token_stream
                    };
                    let vec_values = {
                        #acquire_pool_and_connection_token_stream
                        let mut rows = #binded_query_name_token_stream.fetch(#pg_connection_token_stream.as_mut());
                        let mut vec_values = Vec::new();
                        while let Some(row) = {
                            match {
                                #use_futures_try_stream_ext_token_stream;
                                rows.try_next()
                            }
                            .await
                            {
                                Ok(value) => value,
                                Err(e) => {
                                    #from_log_and_return_error_token_stream;
                                }
                            }
                        } {
                            match #select_token_stream.#options_try_from_sqlx_row_name_token_stream(&row) {
                                Ok(value) => {
                                    vec_values.push(value);
                                }
                                Err(e) => {
                                    #from_log_and_return_error_token_stream;
                                }
                            }
                        }
                        vec_values
                    };
                    #try_read_many_response_variants_token_stream::#desirable_token_stream(vec_values)
                }
            };
            // println!("{prepare_and_execute_query_token_stream}");
            quote::quote!{
                pub async fn #read_many_lower_case_token_stream(
                    #query_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_query_token_stream<#read_many_query_camel_case_token_stream>,
                        #axum_extract_rejection_query_rejection_token_stream,
                    >,
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #read_many_parameters_camel_case_token_stream {
                        #query_lower_case_token_stream: match #crate_server_routes_helpers_query_extractor_error_query_value_result_extractor_token_stream::<
                            #read_many_query_camel_case_token_stream,
                            #try_read_many_response_variants_token_stream,
                        >::#try_extract_value_token_stream(
                            #query_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream
                        ) {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #prepare_and_execute_query_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #query_token_stream
            #query_for_url_encoding_token_stream
            #into_url_encoding_version_token_stream
            #try_read_many_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{read_many_token_stream}");
    let update_one_token_stream = {//todo WHY ITS RETURN SUCCESS EVEN IF ROW DOES NOT EXISTS?
        let update_one_name_camel_case_stringified = "UpdateOne";
        let update_one_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&update_one_name_camel_case_stringified.to_string());
        let update_one_parameters_camel_case_token_stream = {
            let update_one_parameters_camel_case_stringified = format!("{update_one_name_camel_case_stringified}{parameters_camel_case_stringified}");
            update_one_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_one_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_one_path_camel_case_token_stream = {
            let update_one_path_camel_case_stringified = format!("{update_one_name_camel_case_stringified}{path_camel_case_stringified}");
            update_one_path_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_one_path_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        //
        let update_one_path_for_url_encoding_camel_case_token_stream = {
            let update_one_path_for_url_encoding_camel_case_stringified = format!("{update_one_name_camel_case_stringified}{path_camel_case_stringified}{for_url_encoding_camel_case_stringified}");
            update_one_path_for_url_encoding_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_one_path_for_url_encoding_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_one_path_try_from_update_one_path_for_url_encoding_camel_case_stringified = format!("{update_one_name_camel_case_stringified}{path_camel_case_stringified}TryFrom{update_one_name_camel_case_stringified}{path_camel_case_stringified}{for_url_encoding_camel_case_stringified}");
        let update_one_path_try_from_update_one_path_for_url_encoding_error_named_camel_case_token_stream = {//todo reuse TryFrom
            let update_one_path_try_from_update_one_path_for_url_encoding_error_named_camel_case_stringified = format!("{update_one_path_try_from_update_one_path_for_url_encoding_camel_case_stringified}{error_named_camel_case_stringified}");
            update_one_path_try_from_update_one_path_for_url_encoding_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_one_path_try_from_update_one_path_for_url_encoding_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_one_path_try_from_update_one_path_for_url_encoding_camel_case_token_stream = {//todo reuse TryFrom
            update_one_path_try_from_update_one_path_for_url_encoding_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_one_path_try_from_update_one_path_for_url_encoding_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_one_path_try_from_update_one_path_for_url_encoding_lower_case_token_stream = {//todo reuse TryFrom
            let update_one_path_try_from_update_one_path_for_url_encoding_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&update_one_path_try_from_update_one_path_for_url_encoding_camel_case_stringified.to_string());
            update_one_path_try_from_update_one_path_for_url_encoding_lower_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_one_path_try_from_update_one_path_for_url_encoding_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_one_payload_camel_case_token_stream = {
            let update_one_payload_camel_case_stringified = format!("{update_one_name_camel_case_stringified}{payload_camel_case_stringified}");
            update_one_payload_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_one_payload_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_update_one_error_named_camel_case_token_stream = {
            let try_update_one_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{update_one_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_update_one_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_one_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_update_one_response_variants_token_stream = {
            let try_update_one_response_variants_stringified = format!("{try_camel_case_stringified}{update_one_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_update_one_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_one_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #update_one_parameters_camel_case_token_stream {
                    pub #path_lower_case_token_stream: #update_one_path_camel_case_token_stream,
                    pub #payload_lower_case_token_stream: #update_one_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let path_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #update_one_path_camel_case_token_stream {
                    pub #id_field_ident: crate::server::postgres::uuid_wrapper::UuidWrapper,
                }
            }
        };
        // println!("{path_token_stream}");
        let path_for_url_encoding_token_stream = {
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #update_one_path_for_url_encoding_camel_case_token_stream {
                    #id_field_ident: crate::server::postgres::uuid_wrapper::PossibleUuidWrapper,
                }
            }
        };
        // println!("{path_for_url_encoding_token_stream}");
        let update_one_path_try_from_update_one_path_for_url_encoding_error_named_token_stream = {
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #update_one_path_try_from_update_one_path_for_url_encoding_error_named_camel_case_token_stream {
                    NotUuid {
                        #eo_error_occurence_attribute_token_stream
                        not_uuid: crate::server::postgres::uuid_wrapper::UuidWrapperTryFromPossibleUuidWrapperErrorNamed,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                } 
            }
        };
        // println!("{update_one_path_try_from_update_one_path_for_url_encoding_error_named_token_stream}");
        let impl_std_convert_try_from_update_one_path_for_url_encoding_for_update_one_path_token_stream = {
            quote::quote!{
                impl std::convert::TryFrom<#update_one_path_for_url_encoding_camel_case_token_stream> for #update_one_path_camel_case_token_stream {
                    type Error = #update_one_path_try_from_update_one_path_for_url_encoding_error_named_camel_case_token_stream;
                    fn try_from(value: #update_one_path_for_url_encoding_camel_case_token_stream) -> Result<Self, Self::Error> {
                        let #id_field_ident = match crate::server::postgres::uuid_wrapper::UuidWrapper::try_from(value.#id_field_ident) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(Self::Error::NotUuid {
                                    not_uuid: e,
                                    #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                });
                            }
                        };
                        Ok(Self {
                            #id_field_ident
                        })
                    }
                }      
            }
        };
        // println!("{impl_std_convert_try_from_update_one_path_for_url_encoding_for_update_one_path_token_stream}");
        let payload_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    let field_type = &field.ty;
                    Some(quote::quote!{
                        pub #field_ident: Option<#field_type>
                    })
                },
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #update_one_payload_camel_case_token_stream {
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let try_update_one_error_named_token_stream = {
            let try_update_one_request_error_camel_case_token_stream = {
                let try_update_one_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{update_one_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_update_one_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_one_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_update_one_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_update_one_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_update_one_error_named_token_stream}");
        let http_request_token_stream = {
            let try_update_one_lower_case_token_stream = {
                let try_update_one_lower_case_stringified = format!("{try_lower_case_stringified}_{update_one_name_lower_case_stringified}");
                try_update_one_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_one_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{update_one_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let url_handle_token_stream = {
                let url_handle_stringified = format!("\"{{}}/{table_name_stringified}/{{}}\"");//todo where
                url_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {url_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_update_one_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #update_one_parameters_camel_case_token_stream,
                ) -> Result<(), #try_update_one_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_update_one_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        #url_handle_token_stream,
                        #server_location_name_token_stream,
                        #parameters_lower_case_token_stream.#path_lower_case_token_stream.#id_field_ident.to_inner()
                    );
                    // println!("{}", url);
                    match #tvfrr_extraction_logic_token_stream(
                        #reqwest_client_new_token_stream
                        .patch(&url)
                        #project_commit_header_addition_token_stream
                        #content_type_application_json_header_addition_token_stream
                        .body(#payload_lower_case_token_stream)
                        .send(),
                    )
                    .await
                    {
                        Ok(_) => Ok(()),
                        Err(e) => Err(#try_update_one_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let update_one_lower_case_token_stream = update_one_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_one_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let prepare_and_execute_query_error_token_stream = {
                let error_path_stringified = format!("{try_camel_case_stringified}{update_one_name_camel_case_stringified}");
                error_path_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let prepare_and_execute_query_token_stream = {
                let check_for_none_token_stream_excluding_primary_key = crate::check_for_none::check_for_none(
                    &fields_named,
                    &id_field,
                    &proc_macro_name_ident_stringified,
                    dot_space,
                    &try_update_one_response_variants_token_stream,
                    crate::check_for_none::QueryPart::Payload,
                    true
                );
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &prepare_and_execute_query_error_token_stream,
                    &error_log_call_token_stream,
                    &try_update_one_response_variants_token_stream,
                );
                let query_string_token_stream = {
                    let additional_parameters_modification_token_stream = {
                        let fields_named_filtered = fields_named.iter().filter(|field|*field != &id_field).collect::<Vec<&syn::Field>>();
                        let fields_named_len = fields_named_filtered.len();
                        fields_named_filtered.iter().enumerate().map(|(index, field)| {
                            let field_ident = field.ident.clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                });
                            let handle_token_stream = {
                                let possible_dot_space = match (
                                    index.checked_add(1).unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} {index} {}", proc_macro_helpers::global_variables::hardcode::CHECKED_ADD_NONE_OVERFLOW_MESSAGE))
                                ) == fields_named_len {
                                    true => "",
                                    false => dot_space,
                                };
                                let handle_stringified = format!("\"{field_ident} = ${{increment}}{possible_dot_space}\"");
                                handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            quote::quote!{
                                if let Some(value) = &#parameters_lower_case_token_stream.#payload_lower_case_token_stream.#field_ident {
                                    match #crate_server_postgres_bind_query_bind_query_try_increment_token_stream(value, &mut increment) {
                                        Ok(_) => {
                                            query.push_str(&format!(#handle_token_stream));//add dot_space for all elements except last
                                        },
                                        Err(e) => {
                                            return #try_update_one_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                        },
                                    }
                                }
                            }
                        }).collect::<Vec<proc_macro2::TokenStream>>()
                    };
                    let additional_parameters_id_modification_token_stream = {
                        let query_part_token_stream = {
                            let query_part_stringified = format!("\" where {id_field_ident} = ${{increment}}\"");//todo where
                            query_part_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_part_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{
                            match #crate_server_postgres_bind_query_bind_query_try_increment_token_stream(&#parameters_lower_case_token_stream.#path_lower_case_token_stream.#id_field_ident, &mut increment) {
                                Ok(_) => {
                                    query.push_str(&format!(#query_part_token_stream));
                                },
                                Err(e) => {
                                    return #try_update_one_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                },
                            }
                        }
                    };
                    let handle_token_stream = {
                        let handle_stringified = format!("\"{update_name_stringified} {table_name_stringified} {set_name_stringified} \"");//todo where
                        handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    quote::quote!{
                        #increment_initialization_token_stream
                        let mut query = std::string::String::from(#handle_token_stream);
                        #(#additional_parameters_modification_token_stream)*
                        #additional_parameters_id_modification_token_stream
                        query.push_str(&format!(#returning_id_quotes_token_stream));
                        query
                    }
                };
                // println!("{query_string_token_stream}");
                let binded_query_token_stream = {
                    let binded_query_modifications_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                        true => None,
                        false => {
                            let field_ident = field.ident.clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                });
                            Some(quote::quote!{
                                if let Some(value) = #parameters_lower_case_token_stream.#payload_lower_case_token_stream.#field_ident {
                                    query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                                        value,
                                        query,
                                    );
                                }
                            })
                        }
                    });
                    let binded_query_id_modification_token_stream = quote::quote!{
                        query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                            #parameters_lower_case_token_stream.#path_lower_case_token_stream.#id_field_ident,
                            query,
                        );
                    };
                    quote::quote!{
                        let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                        #(#binded_query_modifications_token_stream)*
                        #binded_query_id_modification_token_stream
                        query
                    }
                };
                let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                    &from_log_and_return_error_token_stream,
                    &pg_connection_token_stream
                );
                quote::quote!{
                    #check_for_none_token_stream_excluding_primary_key
                    let #query_string_name_token_stream = {
                        #query_string_token_stream
                    };
                    println!("{}", #query_string_name_token_stream);
                    let #binded_query_name_token_stream = {
                        #binded_query_token_stream
                    };
                    #acquire_pool_and_connection_token_stream
                    match #binded_query_name_token_stream
                        .fetch_one(#pg_connection_token_stream.as_mut())
                        .await
                    {
                        Ok(_) => #try_update_one_response_variants_token_stream::#desirable_token_stream(()),//todo () type token_stream
                        Err(e) => {
                            #from_log_and_return_error_token_stream;
                        }
                    }
                }
            };
            // println!("{prepare_and_execute_query_token_stream}");
            quote::quote!{
                pub async fn #update_one_lower_case_token_stream<'a>(
                    #path_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_path_token_stream<#update_one_path_for_url_encoding_camel_case_token_stream>,
                        #axum_extract_rejection_path_rejection_token_stream,
                    >,
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#update_one_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #update_one_parameters_camel_case_token_stream {
                        #path_lower_case_token_stream: match #crate_server_routes_helpers_path_extractor_error_path_value_result_extractor_token_stream::<
                            #update_one_path_for_url_encoding_camel_case_token_stream,
                            #try_update_one_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#path_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => match #update_one_path_camel_case_token_stream::try_from(value) {
                                Ok(value) => value,
                                Err(e) => {
                                    let error = #prepare_and_execute_query_error_token_stream::#update_one_path_try_from_update_one_path_for_url_encoding_camel_case_token_stream {
                                        #update_one_path_try_from_update_one_path_for_url_encoding_lower_case_token_stream: e,
                                        #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                    };
                                    #error_log_call_token_stream
                                    return #try_update_one_response_variants_token_stream::from(error);
                                }
                            },
                            Err(err) => {
                                return err;
                            }
                        },
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #update_one_payload_camel_case_token_stream,
                            #try_update_one_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #prepare_and_execute_query_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #path_token_stream
            #path_for_url_encoding_token_stream
            #update_one_path_try_from_update_one_path_for_url_encoding_error_named_token_stream
            #impl_std_convert_try_from_update_one_path_for_url_encoding_for_update_one_path_token_stream
            #payload_token_stream
            #try_update_one_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{update_one_token_stream}");
    let update_many_token_stream = {
        let update_many_name_camel_case_stringified = "UpdateMany";
        let update_many_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&update_many_name_camel_case_stringified.to_string());
        let update_many_parameters_camel_case_token_stream = {
            let update_many_parameters_camel_case_stringified = format!("{update_many_name_camel_case_stringified}{parameters_camel_case_stringified}");
            update_many_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_many_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_many_payload_element_camel_case_token_stream = {
            let update_many_payload_element_camel_case_stringified = format!("{update_many_name_camel_case_stringified}{payload_element_camel_case_stringified}");
            update_many_payload_element_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_many_payload_element_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_many_payload_camel_case_token_stream = quote::quote!{Vec<#update_many_payload_element_camel_case_token_stream>};
        let update_many_payload_element_with_serialize_deserialize_camel_case_token_stream = {
            let update_many_payload_element_with_serialize_deserialize_camel_case_stringified = format!("{update_many_name_camel_case_stringified}{payload_element_with_serialize_deserialize_camel_case_stringified}");
            update_many_payload_element_with_serialize_deserialize_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_many_payload_element_with_serialize_deserialize_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_many_payload_with_serialize_deserialize_camel_case_token_stream = quote::quote!{Vec<#update_many_payload_element_with_serialize_deserialize_camel_case_token_stream>};
        let update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_camel_case_stringified = format!(
            "{update_many_name_camel_case_stringified}{payload_element_camel_case_stringified}TryFrom{update_many_name_camel_case_stringified}{payload_element_with_serialize_deserialize_camel_case_stringified}"
        );
        let update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_camel_case_token_stream = {
            update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_lower_case_token_stream = {
            let update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_camel_case_stringified);
            update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_lower_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_error_named_camel_case_token_stream = {
            let update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_error_named_camel_case_stringified = format!(
                "{update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_camel_case_stringified}ErrorNamed"
            );
            update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_update_many_error_named_camel_case_token_stream = {
            let try_update_many_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{update_many_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_update_many_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_many_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_update_many_response_variants_token_stream = {
            let try_update_many_response_variants_stringified = format!("{try_camel_case_stringified}{update_many_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_update_many_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_many_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #update_many_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #update_many_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let payload_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    let field_type = &field.ty;
                    Some(quote::quote!{
                        pub #field_ident: #field_type
                    })
                },
            });
            quote::quote!{
                #derive_debug_token_stream
                pub struct #update_many_payload_element_camel_case_token_stream {
                    pub #id_field_ident: crate::server::postgres::uuid_wrapper::UuidWrapper,
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let payload_with_serialize_deserialize_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    let field_type = &field.ty;
                    Some(quote::quote!{
                        pub #field_ident: #field_type
                    })
                },
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #update_many_payload_element_with_serialize_deserialize_camel_case_token_stream {
                    pub #id_field_ident: crate::server::postgres::uuid_wrapper::PossibleUuidWrapper,
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_with_serialize_deserialize_token_stream}");
        let impl_std_convert_from_update_many_payload_elemen_for_update_many_payload_element_with_serialize_deserialize_token_stream = {
            let fields_assignments_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    Some(quote::quote!{
                        let #field_ident = value.#field_ident;
                    })
                },
            });
            let self_init_fields_token_stream = fields_named.iter().map(|field|{
                let field_ident = field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    #field_ident
                }
            });
            quote::quote!{
                impl std::convert::From<#update_many_payload_element_camel_case_token_stream> for #update_many_payload_element_with_serialize_deserialize_camel_case_token_stream {
                    fn from(value: #update_many_payload_element_camel_case_token_stream) -> Self {
                        let #id_field_ident = crate::server::postgres::uuid_wrapper::PossibleUuidWrapper::from(value.#id_field_ident);
                        #(#fields_assignments_token_stream)*
                        Self {
                            #(#self_init_fields_token_stream),*
                        }
                    }
                }
            }
        };
        // println!("{impl_std_convert_from_update_many_payload_elemen_for_update_many_payload_element_with_serialize_deserialize_token_stream}");
        //
        let update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_error_named_token_stream = {
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_error_named_camel_case_token_stream {
                    NotUuid {
                        #eo_display_attribute_token_stream
                        not_uuid: sqlx::types::uuid::Error,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_error_named_token_stream}");
        let impl_std_convert_try_from_update_many_payload_element_with_serialize_deserialize_for_update_many_payload_element_token_stream = {
            let fields_assignments_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    Some(quote::quote!{
                        let #field_ident = value.#field_ident;
                    })
                },
            });
            let self_init_fields_token_stream = fields_named.iter().map(|field|{
                let field_ident = field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    #field_ident
                }
            });
            quote::quote!{
                impl std::convert::TryFrom<#update_many_payload_element_with_serialize_deserialize_camel_case_token_stream> for #update_many_payload_element_camel_case_token_stream {
                    type Error = #update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_error_named_camel_case_token_stream;
                    fn try_from(value: #update_many_payload_element_with_serialize_deserialize_camel_case_token_stream) -> Result<Self, Self::Error> {
                        let #id_field_ident = match sqlx::types::Uuid::parse_str(value.#id_field_ident.to_inner()) {
                            Ok(value) => crate::server::postgres::uuid_wrapper::UuidWrapper::from(value),
                            Err(e) => {
                                return Err(#update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_error_named_camel_case_token_stream::NotUuid {
                                    not_uuid: e,
                                    #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                });
                            },
                        };
                        #(#fields_assignments_token_stream)*
                        Ok(Self{
                            #(#self_init_fields_token_stream),*
                        })
                    }
                }
            }
        };
        // println!("{impl_std_convert_try_from_update_many_payload_element_with_serialize_deserialize_for_update_many_payload_element_token_stream}");
        let try_update_many_error_named_token_stream = {
            let try_update_many_request_error_camel_case_token_stream = {
                let try_update_many_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{update_many_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_update_many_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_many_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_update_many_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_update_many_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_update_many_error_named_token_stream}");
        let http_request_token_stream = {
            let try_update_many_lower_case_token_stream = {
                let try_update_many_lower_case_stringified = format!("{try_lower_case_stringified}_{update_many_name_lower_case_stringified}");
                try_update_many_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_many_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{update_many_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let url_handle_token_stream = {
                let url_handle_stringified = format!("\"{{}}/{table_name_stringified}/\"");//todo where
                url_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {url_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_update_many_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #update_many_parameters_camel_case_token_stream,
                ) -> Result<(), #try_update_many_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&
                        #parameters_lower_case_token_stream.#payload_lower_case_token_stream
                        .into_iter()
                        .map(
                            |element|
                            #update_many_payload_element_with_serialize_deserialize_camel_case_token_stream::from(element)
                        )
                        .collect::<Vec<#update_many_payload_element_with_serialize_deserialize_camel_case_token_stream>>()
                    ) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_update_many_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        #url_handle_token_stream,
                        #server_location_name_token_stream
                    );
                    // println!("{}", url);
                    match #tvfrr_extraction_logic_token_stream(
                        #reqwest_client_new_token_stream
                        .patch(&url)
                        #project_commit_header_addition_token_stream
                        #content_type_application_json_header_addition_token_stream
                        .body(#payload_lower_case_token_stream)
                        .send(),
                    )
                    .await
                    {
                        Ok(_) => Ok(()),
                        Err(e) => Err(#try_update_many_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let update_many_lower_case_token_stream = update_many_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_many_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let prepare_and_execute_query_error_token_stream = {
                let error_path_stringified = format!("{try_camel_case_stringified}{update_many_name_camel_case_stringified}");
                error_path_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let prepare_and_execute_query_token_stream = {
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &prepare_and_execute_query_error_token_stream,
                    &error_log_call_token_stream,
                    &try_update_many_response_variants_token_stream,
                );
                let expected_updated_primary_keys_token_stream = quote::quote!{
                    #parameters_lower_case_token_stream
                    .#payload_lower_case_token_stream
                    .iter()
                    .map(|element| element.#id_field_ident.clone()) //todo - maybe its not a good idea to remove .clone here coz in macro dont know what type
                    .collect::<Vec<crate::server::postgres::uuid_wrapper::UuidWrapper>>()
                };
                let query_string_token_stream = {
                    let column_names = fields_named.iter().enumerate().fold(std::string::String::default(), |mut acc, (index, field)| {
                        let field_ident = field.ident.clone()
                            .unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                        let possible_dot_space = match (
                            index.checked_add(1).unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} {index} {}", proc_macro_helpers::global_variables::hardcode::CHECKED_ADD_NONE_OVERFLOW_MESSAGE))
                        ) == fields_named_len {
                            true => "",
                            false => dot_space,
                        };
                        acc.push_str(&format!("{field_ident}{possible_dot_space}"));
                        acc
                    });
                    let column_increments = fields_named.iter().enumerate().fold(std::string::String::default(), |mut acc, (index, _)| {
                        let incremented_index = index.checked_add(1).unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} {index} {}", proc_macro_helpers::global_variables::hardcode::CHECKED_ADD_NONE_OVERFLOW_MESSAGE));
                        let possible_dot_space = match (incremented_index) == fields_named_len {
                            true => "",
                            false => dot_space,
                        };
                        acc.push_str(&format!("${incremented_index}{possible_dot_space}"));
                        acc
                    });
                    let declarations = {
                        let fields_named_filtered = fields_named.iter().filter(|field|*field != &id_field).collect::<Vec<&syn::Field>>();
                        let fields_named_len = fields_named_filtered.len();
                        fields_named_filtered.iter().enumerate().fold(std::string::String::default(), |mut acc, (index, field)| {
                            let field_ident = field.ident.clone().unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                            let possible_dot_space = match (
                                index.checked_add(1).unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} {index} {}", proc_macro_helpers::global_variables::hardcode::CHECKED_ADD_NONE_OVERFLOW_MESSAGE))
                            ) == fields_named_len {
                                true => "",
                                false => dot_space,
                            };
                            acc.push_str(&format!("{field_ident} = data.{field_ident}{possible_dot_space}"));
                            acc
                        })
                    };
                    let query_stringified = format!("\"{update_name_stringified} {table_name_stringified} {as_name_stringified} t {set_name_stringified} {declarations} {from_name_stringified} ({select_name_stringified} * {from_name_stringified} {unnest_name_stringified}({column_increments})) as data({column_names}) where t.{id_field_ident} = data.{id_field_ident} {returning_stringified} data.{id_field_ident}\"");
                    query_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let binded_query_token_stream = {
                    let column_vecs_token_stream = fields_named.iter().map(|field|{
                        let field_ident_underscore_vec_stringified = {
                            let field_ident = field.ident.clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                });
                            format!("{field_ident}{underscore_vec_name_stringified}")
                        };
                        field_ident_underscore_vec_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_underscore_vec_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    });
                    let column_vecs_with_capacity_token_stream = fields_named.iter().map(|_|quote::quote!{Vec::with_capacity(#current_vec_len_name_token_stream)});
                    let columns_acc_push_elements_token_stream = fields_named.iter().enumerate().map(|(index, field)|{
                        let field_ident = field.ident.clone()
                            .unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                        let index_token_stream = {
                            let index_stringified = format!("{index}");
                            index_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {index_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{#acc_name_token_stream.#index_token_stream.push(#element_name_token_stream.#field_ident);}
                    });
                    let column_query_bind_id_vec_token_stream = {
                        let field_ident_underscore_vec_token_stream = {
                            let field_ident_underscore_vec_stringified = format!("{id_field_ident}{underscore_vec_name_stringified}");
                            field_ident_underscore_vec_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_underscore_vec_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{
                            #query_name_token_stream = #query_name_token_stream.bind(
                                #field_ident_underscore_vec_token_stream
                                .into_iter()
                                .map(|element| element.into_inner())
                                .collect::<Vec<sqlx::types::Uuid>>()
                            );
                        }
                    };
                    let column_query_bind_vecs_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                        true => None,
                        false => {
                            let field_ident_underscore_vec_token_stream = {
                                let field_ident_underscore_vec_stringified = {
                                    let field_ident = field.ident.clone()
                                        .unwrap_or_else(|| {
                                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                        });
                                    format!("{field_ident}{underscore_vec_name_stringified}")
                                };
                                field_ident_underscore_vec_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_underscore_vec_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            Some(quote::quote!{#query_name_token_stream = #query_name_token_stream.bind(#field_ident_underscore_vec_token_stream);})
                        },
                    });
                    quote::quote!{
                        let mut #query_name_token_stream = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                        let #current_vec_len_name_token_stream = #parameters_lower_case_token_stream.#payload_lower_case_token_stream.len();
                        let (
                            #(#column_vecs_token_stream),*
                        ) = #parameters_lower_case_token_stream.#payload_lower_case_token_stream.into_iter().fold((
                            #(#column_vecs_with_capacity_token_stream),*
                        ), |mut #acc_name_token_stream, #element_name_token_stream| {
                            #(#columns_acc_push_elements_token_stream)*
                            #acc_name_token_stream
                        });
                        #column_query_bind_id_vec_token_stream
                        #(#column_query_bind_vecs_token_stream)*
                        #query_name_token_stream
                    }
                };
                let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                    &from_log_and_return_error_token_stream,
                    &pg_connection_token_stream
                );
                let generate_postgres_transaction_token_stream = crate::generate_postgres_transaction::generate_postgres_transaction(
                    &expected_updated_primary_keys_token_stream,
                    &query_string_name_token_stream,
                    &query_string_token_stream,
                    &binded_query_token_stream,
                    &acquire_pool_and_connection_token_stream,
                    &use_sqlx_acquire_token_stream,
                    &pg_connection_token_stream,
                    &begin_token_stream,
                    &binded_query_name_token_stream,
                    &use_futures_try_stream_ext_token_stream,
                    &query_and_rollback_failed_token_stream,
                    &primary_key_uuid_wrapper_try_from_sqlx_row_name_token_stream,
                    &from_log_and_return_error_token_stream,
                    &rollback_error_name_token_stream,
                    &primary_key_from_row_and_failed_rollback_token_stream,
                    &non_existing_primary_keys_name_token_stream,
                    &expected_updated_primary_keys_name_token_stream,
                    &primary_key_vec_name_token_stream,
                    &rollback_token_stream,
                    &non_existing_primary_keys_token_stream,
                    &non_existing_primary_keys_and_failed_rollback_token_stream,
                    &postgres_transaction_token_stream,
                    &commit_token_stream,
                    &try_update_many_response_variants_token_stream,
                    &desirable_token_stream,
                    &prepare_and_execute_query_error_token_stream,
                    &commit_failed_token_stream,
                    &error_log_call_token_stream,
                );
                quote::quote!{
                    {
                        let #not_unique_primary_keys_name_token_stream = {
                            let mut vec = Vec::with_capacity(#parameters_lower_case_token_stream.#payload_lower_case_token_stream.len());
                            let mut #not_unique_primary_keys_name_token_stream = Vec::with_capacity(#parameters_lower_case_token_stream.#payload_lower_case_token_stream.len());
                            for element in &parameters.payload {
                                let handle = &element.#id_field_ident;
                                match vec.contains(&handle) {
                                    true => {
                                        #not_unique_primary_keys_name_token_stream.push(element.#id_field_ident.clone());
                                    },
                                    false => {
                                        vec.push(&element.#id_field_ident);
                                    }
                                }
                            }
                            #not_unique_primary_keys_name_token_stream
                        };
                        if let false = #not_unique_primary_keys_name_token_stream.is_empty() {
                            let error = #prepare_and_execute_query_error_token_stream::#not_unique_primery_key_token_stream;
                            #error_log_call_token_stream
                            return #try_update_many_response_variants_token_stream::from(error);
                        }
                    }
                    #generate_postgres_transaction_token_stream
                }  
            };
            // println!("{prepare_and_execute_query_token_stream}");
            quote::quote!{
                pub async fn #update_many_lower_case_token_stream<'a>(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#update_many_payload_with_serialize_deserialize_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #update_many_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #update_many_payload_with_serialize_deserialize_camel_case_token_stream,
                            #try_update_many_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => match value.into_iter()
                                .map(|element|#update_many_payload_element_camel_case_token_stream::try_from(element))
                                .collect::<Result<
                                    #update_many_payload_camel_case_token_stream, 
                                    #update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_error_named_camel_case_token_stream
                                >>() 
                                {
                                    Ok(value) => value,
                                    Err(e) => {
                                        let error = #prepare_and_execute_query_error_token_stream::#update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_camel_case_token_stream {
                                            #update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_lower_case_token_stream: e,
                                            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                        };
                                        #error_log_call_token_stream
                                        return #try_update_many_response_variants_token_stream::from(error);
                                    }
                                },
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #prepare_and_execute_query_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #payload_with_serialize_deserialize_token_stream
            #impl_std_convert_from_update_many_payload_elemen_for_update_many_payload_element_with_serialize_deserialize_token_stream
            #update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_error_named_token_stream
            #impl_std_convert_try_from_update_many_payload_element_with_serialize_deserialize_for_update_many_payload_element_token_stream
            #try_update_many_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{update_many_token_stream}");
    let delete_one_token_stream = {
        let delete_one_name_camel_case_stringified = "DeleteOne";
        let delete_one_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&delete_one_name_camel_case_stringified.to_string());
        let delete_one_parameters_camel_case_token_stream = {
            let delete_one_parameters_camel_case_stringified = format!("{delete_one_name_camel_case_stringified}{parameters_camel_case_stringified}");
            delete_one_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_one_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let delete_one_path_camel_case_token_stream = {
            let delete_one_path_camel_case_stringified = format!("{delete_one_name_camel_case_stringified}{path_camel_case_stringified}");
            delete_one_path_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_one_path_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        //
        let delete_one_path_with_serialize_deserialize_camel_case_token_stream = {
            let delete_one_path_with_serialize_deserialize_camel_case_stringified = format!("{delete_one_name_camel_case_stringified}{path_camel_case_stringified}WithSerializeDeserialize");
            delete_one_path_with_serialize_deserialize_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_one_path_with_serialize_deserialize_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        //
        let delete_one_path_try_from_delete_one_path_with_serialize_deserialize_error_named_name_token_stream = {
            let delete_one_path_try_from_delete_one_path_with_serialize_deserialize_error_named_stringified = format!(
                "{delete_one_name_camel_case_stringified}{path_camel_case_stringified}TryFrom{delete_one_name_camel_case_stringified}{path_camel_case_stringified}WithSerializeDeserialize{error_named_camel_case_stringified}"
            );
            delete_one_path_try_from_delete_one_path_with_serialize_deserialize_error_named_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_one_path_try_from_delete_one_path_with_serialize_deserialize_error_named_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        //
        let try_delete_one_error_named_camel_case_token_stream = {
            let try_delete_one_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{delete_one_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_delete_one_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_one_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_delete_one_response_variants_token_stream = {
            let try_delete_one_response_variants_stringified = format!("{try_camel_case_stringified}{delete_one_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_delete_one_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_one_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #delete_one_parameters_camel_case_token_stream {
                    pub #path_lower_case_token_stream: #delete_one_path_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let path_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #delete_one_path_camel_case_token_stream {
                    pub #id_field_ident: crate::server::postgres::uuid_wrapper::UuidWrapper,
                }
            }
        };
        // println!("{path_token_stream}");
        let path_with_serialize_deserialize_token_stream = {
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #delete_one_path_with_serialize_deserialize_camel_case_token_stream {
                    pub #id_field_ident: crate::server::postgres::uuid_wrapper::PossibleUuidWrapper,
                }
            }
        };
        // println!("{path_with_serialize_deserialize_token_stream}");
        let delete_one_path_try_from_delete_one_path_with_serialize_deserialize_error_named_token_stream = {
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #delete_one_path_try_from_delete_one_path_with_serialize_deserialize_error_named_name_token_stream {
                    NotUuid {
                        #eo_error_occurence_attribute_token_stream
                        not_uuid: crate::server::postgres::uuid_wrapper::UuidWrapperTryFromPossibleUuidWrapperErrorNamed,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{delete_one_path_try_from_delete_one_path_with_serialize_deserialize_error_named_token_stream}");
        let impl_std_convert_try_from_delete_one_path_with_serialize_deserialize_for_delete_one_path_token_stream = {
            quote::quote!{
                impl std::convert::TryFrom<#delete_one_path_with_serialize_deserialize_camel_case_token_stream> for #delete_one_path_camel_case_token_stream {
                    type Error = #delete_one_path_try_from_delete_one_path_with_serialize_deserialize_error_named_name_token_stream;
                    fn try_from(value: #delete_one_path_with_serialize_deserialize_camel_case_token_stream) -> Result<Self, Self::Error> {
                        match crate::server::postgres::uuid_wrapper::UuidWrapper::try_from(value.#id_field_ident) {
                            Ok(value) => Ok(Self { #id_field_ident: value }),
                            Err(e) => Err(#delete_one_path_try_from_delete_one_path_with_serialize_deserialize_error_named_name_token_stream::NotUuid {
                                not_uuid: e,
                                #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                            }),
                        }
                    }
                }
            }
        };
        // println!("{impl_std_convert_try_from_delete_one_path_with_serialize_deserialize_for_delete_one_path_token_stream}");
        let try_delete_one_error_named_token_stream = {
            let try_delete_one_request_error_camel_case_token_stream = {
                let try_delete_one_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{delete_one_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_delete_one_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_one_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_delete_one_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_delete_one_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{try_delete_one_error_named_token_stream}");
        let http_request_token_stream = {
            let try_delete_one_lower_case_token_stream = {
                let try_delete_one_lower_case_stringified = format!("{try_lower_case_stringified}_{delete_one_name_lower_case_stringified}");
                try_delete_one_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_one_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{delete_one_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let url_handle_token_stream = {
                let url_handle_stringified = format!("\"{{}}/{table_name_stringified}/{{}}\"");//todo where
                url_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {url_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_delete_one_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #delete_one_parameters_camel_case_token_stream,
                ) -> Result<(), #try_delete_one_error_named_camel_case_token_stream> {
                    let url = format!(
                        #url_handle_token_stream,
                        #server_location_name_token_stream,
                        #parameters_lower_case_token_stream.#path_lower_case_token_stream.#id_field_ident
                    );
                    // println!("{}", url);
                    match #tvfrr_extraction_logic_token_stream(
                        #reqwest_client_new_token_stream
                        .delete(&url)
                        #project_commit_header_addition_token_stream
                        .send(),
                    )
                    .await
                    {
                        Ok(value) => Ok(value),
                        Err(e) => Err(#try_delete_one_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let delete_one_lower_case_token_stream = delete_one_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_one_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let prepare_and_execute_query_token_stream = {
                let prepare_and_execute_query_error_token_stream = {
                    let error_path_stringified = format!("{try_camel_case_stringified}{delete_one_name_camel_case_stringified}");
                    error_path_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &prepare_and_execute_query_error_token_stream,
                    &error_log_call_token_stream,
                    &try_delete_one_response_variants_token_stream,
                );
                let query_string_token_stream = {
                    let additional_parameters_id_modification_token_stream = {
                        let query_part_token_stream = {
                            let query_part_stringified = format!("\" {id_field_ident} = $1\"");//todo where
                            query_part_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_part_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{
                            query.push_str(&format!(#query_part_token_stream));
                        }
                    };
                    let handle_token_stream = {
                        let handle_stringified = format!("\"{delete_name_stringified} {from_name_stringified} {table_name_stringified} {where_name_stringified}\"");//todo where
                        handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    quote::quote!{
                        let mut query = format!(#handle_token_stream);
                        #additional_parameters_id_modification_token_stream
                        query.push_str(&format!(#returning_id_quotes_token_stream));
                        query
                    }
                };
                let binded_query_token_stream = {
                    let binded_query_modifications_token_stream = quote::quote!{
                        query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(#parameters_lower_case_token_stream.#path_lower_case_token_stream.#id_field_ident, query);
                    };
                    quote::quote!{
                        let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                        #binded_query_modifications_token_stream
                        query
                    }
                };
                let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                    &from_log_and_return_error_token_stream,
                    &pg_connection_token_stream
                );
                quote::quote!{
                    let #query_string_name_token_stream = {
                        #query_string_token_stream
                    };
                    println!("{}", #query_string_name_token_stream);
                    let #binded_query_name_token_stream = {
                        #binded_query_token_stream
                    };
                    #acquire_pool_and_connection_token_stream
                    match #binded_query_name_token_stream
                        .fetch_one(#pg_connection_token_stream.as_mut())
                        .await
                    {
                        Ok(row) => #try_delete_one_response_variants_token_stream::#desirable_token_stream(()),//todo - () as variable token stream
                        Err(e) => {
                            #from_log_and_return_error_token_stream;
                        }
                    }
                }
            };
            // println!("{prepare_and_execute_query_token_stream}");
            quote::quote!{
                #[utoipa::path(
                    delete,
                    path = "/api/cats",
                    responses(
                        (status = 200, description = "delete cat by id", body = [TryDeleteOne])
                    )
                )]
                pub async fn #delete_one_lower_case_token_stream<'a>(
                    #path_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_path_token_stream<#delete_one_path_camel_case_token_stream>,
                        #axum_extract_rejection_path_rejection_token_stream,
                    >,
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #delete_one_parameters_camel_case_token_stream {
                        #path_lower_case_token_stream: match #crate_server_routes_helpers_path_extractor_error_path_value_result_extractor_token_stream::<
                            #delete_one_path_camel_case_token_stream,
                            #try_delete_one_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#path_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #prepare_and_execute_query_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #path_token_stream
            #path_with_serialize_deserialize_token_stream
            #delete_one_path_try_from_delete_one_path_with_serialize_deserialize_error_named_token_stream
            #impl_std_convert_try_from_delete_one_path_with_serialize_deserialize_for_delete_one_path_token_stream
            #try_delete_one_error_named_token_stream
            // #http_request_token_stream
            // #route_handler_token_stream
        }
    };
    // println!("{delete_one_token_stream}");
    let delete_many_with_body_token_stream = {
        let delete_many_with_body_name_camel_case_stringified = "DeleteManyWithBody";
        let delete_many_with_body_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&delete_many_with_body_name_camel_case_stringified.to_string());
        let delete_many_with_body_parameters_camel_case_token_stream = {
            let delete_many_with_body_parameters_camel_case_stringified = format!("{delete_many_with_body_name_camel_case_stringified}{parameters_camel_case_stringified}");
            delete_many_with_body_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_many_with_body_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let delete_many_with_body_payload_camel_case_token_stream = {
            let delete_many_with_body_payload_camel_case_stringified = format!("{delete_many_with_body_name_camel_case_stringified}{payload_camel_case_stringified}");
            delete_many_with_body_payload_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_many_with_body_payload_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_delete_many_with_body_error_named_camel_case_token_stream = {
            let try_delete_many_with_body_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{delete_many_with_body_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_delete_many_with_body_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_many_with_body_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_delete_many_with_body_response_variants_token_stream = {
            let try_delete_many_with_body_response_variants_stringified = format!("{try_camel_case_stringified}{delete_many_with_body_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_delete_many_with_body_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_many_with_body_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_deserialize_token_stream
                pub struct #delete_many_with_body_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #delete_many_with_body_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let payload_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    Some(quote::quote!{
                        pub #field_ident: Option<Vec<crate::server::postgres::regex_filter::RegexFilter>>//todo
                    })
                },
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #delete_many_with_body_payload_camel_case_token_stream {
                    pub #id_field_ident: Option<Vec<std::string::String>>,//crate::server::postgres::bigserial::Bigserial
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let try_delete_many_with_body_error_named_token_stream = {
            let try_delete_many_with_body_request_error_camel_case_token_stream = {
                let try_delete_many_with_body_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{delete_many_with_body_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_delete_many_with_body_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_many_with_body_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_delete_many_with_body_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_delete_many_with_body_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_delete_many_with_body_error_named_token_stream}");
        let http_request_token_stream = {
            let try_delete_many_with_body_lower_case_token_stream = {
                let try_delete_many_with_body_lower_case_stringified = format!("{try_lower_case_stringified}_{delete_many_with_body_name_lower_case_stringified}");
                try_delete_many_with_body_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_many_with_body_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{delete_many_with_body_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let url_handle_token_stream = {
                let url_handle_stringified = format!("\"{{}}/{table_name_stringified}/search\"");//todo where
                url_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {url_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_delete_many_with_body_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #delete_many_with_body_parameters_camel_case_token_stream,
                ) -> Result<(), #try_delete_many_with_body_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_delete_many_with_body_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        #url_handle_token_stream,
                        #server_location_name_token_stream,
                    );
                    // println!("{}", url);
                    match #tvfrr_extraction_logic_token_stream(
                        #reqwest_client_new_token_stream
                        .delete(&url)
                        #project_commit_header_addition_token_stream
                        #content_type_application_json_header_addition_token_stream
                        .body(#payload_lower_case_token_stream)
                        .send(),
                    )
                    .await
                    {
                        Ok(value) => Ok(value),
                        Err(e) => Err(#try_delete_many_with_body_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let delete_many_with_body_lower_case_token_stream = delete_many_with_body_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_many_with_body_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let prepare_and_execute_query_token_stream = {
                let prepare_and_execute_query_error_token_stream = {
                    let error_path_stringified = format!("{try_camel_case_stringified}{delete_many_with_body_name_camel_case_stringified}");
                    error_path_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &prepare_and_execute_query_error_token_stream,
                    &error_log_call_token_stream,
                    &try_delete_many_with_body_response_variants_token_stream,
                );
                let query_part = crate::check_for_none::QueryPart::Payload;
                let check_for_none_token_stream = crate::check_for_none::check_for_none(
                    &fields_named,
                    &id_field,
                    &proc_macro_name_ident_stringified,
                    dot_space,
                    &try_delete_many_with_body_response_variants_token_stream,
                    query_part,
                    false
                );
                let parameters_match_token_stream = fields_named.iter().map(|field| {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    quote::quote!{
                        &#parameters_lower_case_token_stream.#payload_lower_case_token_stream.#field_ident
                    }
                });
                let parameters_match_primary_key_some_other_none_token_stream = fields_named.iter().map(|field| {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    match field_ident == id_field_ident {
                        true => quote::quote!{Some(#id_field_ident)},
                        false => quote::quote!{None}
                    }
                });
                let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                    &from_log_and_return_error_token_stream,
                    &pg_connection_token_stream
                );
                let generate_postgres_transaction_token_stream = {
                    let filter_unique_parameters_token_stream = {
                        let filter_unique_parameters_primary_key_token_stream = quote::quote!{
                            let #not_unique_primary_keys_name_token_stream = {
                                let mut vec = Vec::with_capacity(#id_field_ident.len());
                                let mut #not_unique_primary_keys_name_token_stream = Vec::with_capacity(#id_field_ident.len());
                                for element in #id_field_ident {
                                    let handle = element;
                                    match vec.contains(&handle) {
                                        true => {
                                            #not_unique_primary_keys_name_token_stream.push(element.clone());
                                        },
                                        false => {
                                            vec.push(element);
                                        }
                                    }
                                }
                                #not_unique_primary_keys_name_token_stream
                            };
                            if let false = #not_unique_primary_keys_name_token_stream.is_empty() {
                                let error = #prepare_and_execute_query_error_token_stream::#not_unique_primery_key_token_stream;
                                #error_log_call_token_stream
                                return #try_delete_many_with_body_response_variants_token_stream::from(error);
                            }
                        };
                        quote::quote!{
                            #filter_unique_parameters_primary_key_token_stream
                        }
                    };
                    let expected_updated_primary_keys_token_stream = quote::quote!{
                        #id_field_ident
                        .iter()
                        .map(|element| element.clone()) //todo - maybe its not a good idea to remove .clone here coz in macro dont know what type
                        .collect::<Vec<#id_field_type>>()
                    };
                    let query_string_primary_key_some_other_none_token_stream = {
                        let handle_stringified = format!("\"{delete_name_stringified} {from_name_stringified} {table_name_stringified} {where_name_stringified} {id_field_ident} {in_name_stringified} ({select_name_stringified} {unnest_name_stringified}($1)){returning_id_stringified}\"");
                        handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    let binded_query_primary_key_some_other_none_token_stream = quote::quote!{
                        let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                        query = query.bind(
                            #id_field_ident
                            .into_iter()
                            .map(|element| element.clone())
                            .collect::<Vec<#id_field_type>>()
                        );
                        query
                    };
                    let generate_postgres_transaction_token_stream = crate::generate_postgres_transaction::generate_postgres_transaction(
                        &expected_updated_primary_keys_token_stream,
                        &query_string_name_token_stream,
                        &query_string_primary_key_some_other_none_token_stream,
                        &binded_query_primary_key_some_other_none_token_stream,
                        &acquire_pool_and_connection_token_stream,
                        &use_sqlx_acquire_token_stream,
                        &pg_connection_token_stream,
                        &begin_token_stream,
                        &binded_query_name_token_stream,
                        &use_futures_try_stream_ext_token_stream,
                        &query_and_rollback_failed_token_stream,
                        &primary_key_try_from_sqlx_row_name_token_stream,
                        &from_log_and_return_error_token_stream,
                        &rollback_error_name_token_stream,
                        &primary_key_from_row_and_failed_rollback_token_stream,
                        &non_existing_primary_keys_name_token_stream,
                        &expected_updated_primary_keys_name_token_stream,
                        &primary_key_vec_name_token_stream,
                        &rollback_token_stream,
                        &non_existing_primary_keys_token_stream,
                        &non_existing_primary_keys_and_failed_rollback_token_stream,
                        &postgres_transaction_token_stream,
                        &commit_token_stream,
                        &try_delete_many_with_body_response_variants_token_stream,
                        &desirable_token_stream,
                        &prepare_and_execute_query_error_token_stream,
                        &commit_failed_token_stream,
                        &error_log_call_token_stream,
                    );
                    quote::quote!{
                        #filter_unique_parameters_token_stream
                        #generate_postgres_transaction_token_stream
                    }
                };
                let generate_postgres_execute_query_token_stream = {
                    let filter_unique_parameters_token_stream = {
                        let filter_unique_parameters_primary_key_token_stream = quote::quote!{
                            if let Some(#id_field_ident) = &#parameters_lower_case_token_stream.#payload_lower_case_token_stream.#id_field_ident {
                                let #not_unique_primary_keys_name_token_stream = {
                                    let mut vec = Vec::with_capacity(#id_field_ident.len());
                                    let mut #not_unique_primary_keys_name_token_stream = Vec::with_capacity(#id_field_ident.len());
                                    for element in #id_field_ident {
                                        let handle = element;
                                        match vec.contains(&handle) {
                                            true => {
                                                #not_unique_primary_keys_name_token_stream.push(element.clone());
                                            },
                                            false => {
                                                vec.push(element);
                                            }
                                        }
                                    }
                                    #not_unique_primary_keys_name_token_stream
                                };
                                if let false = #not_unique_primary_keys_name_token_stream.is_empty() {
                                    let error = #prepare_and_execute_query_error_token_stream::#not_unique_primery_key_token_stream;
                                    #error_log_call_token_stream
                                    return #try_delete_many_with_body_response_variants_token_stream::from(error);
                                }
                            }
                        };
                        let filter_unique_parameters_other_columns_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                            true => None,
                            false => {
                                let field_ident = field.ident.clone()
                                    .unwrap_or_else(|| {
                                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                    });
                                let field_handle_token_stream = {
                                    let field_handle_stringified = format!("{field_ident}_handle");
                                    field_handle_stringified.parse::<proc_macro2::TokenStream>()
                                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                };
                                let not_unique_field_vec_lower_case_token_stream = {
                                    let not_unique_field_vec_lower_case_stringified = format!("not_unique_{field_ident}_vec");
                                    not_unique_field_vec_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {not_unique_field_vec_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                };
                                let not_unique_field_vec_vec_pascal_token_stream = {
                                    let not_unique_field_vec_pascal_stringified = format!(
                                        "NotUnique{}Vec",
                                        {
                                            use convert_case::Casing;
                                            field_ident.to_string().to_case(convert_case::Case::Pascal)
                                        }
                                    );
                                    not_unique_field_vec_pascal_stringified.parse::<proc_macro2::TokenStream>()
                                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {not_unique_field_vec_pascal_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                };
                                Some(quote::quote!{
                                    let #field_handle_token_stream = match #parameters_lower_case_token_stream.#payload_lower_case_token_stream.#field_ident {
                                        Some(value) => {
                                            let is_unique = {
                                                let mut vec = Vec::with_capacity(value.len());
                                                let mut is_unique = true;
                                                for element in &value {
                                                    match vec.contains(&element) {
                                                        true => {
                                                            is_unique = false;
                                                            break;
                                                        }
                                                        false => {
                                                            vec.push(element);
                                                        }
                                                    }
                                                }
                                                is_unique
                                            };
                                            match is_unique {
                                                true => Some(value),
                                                false => {
                                                    let #not_unique_field_vec_lower_case_token_stream = {
                                                        let mut vec = Vec::with_capacity(value.len());
                                                        let mut #not_unique_field_vec_lower_case_token_stream = Vec::with_capacity(value.len());
                                                        for element in value {
                                                            match vec.contains(&element) {
                                                                true => {
                                                                    #not_unique_field_vec_lower_case_token_stream.push(element);
                                                                }
                                                                false => {
                                                                    vec.push(element);
                                                                }
                                                            }
                                                        }
                                                        #not_unique_field_vec_lower_case_token_stream
                                                    };
                                                    let error = #prepare_and_execute_query_error_token_stream::#not_unique_field_vec_vec_pascal_token_stream {
                                                        #not_unique_field_vec_lower_case_token_stream,
                                                        #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                                    };
                                                    #error_log_call_token_stream
                                                    return #try_delete_many_with_body_response_variants_token_stream::from(error);
                                                }
                                            }
                                        },
                                        None => None
                                    };
                                })
                            },
                        });
                        quote::quote!{
                            #filter_unique_parameters_primary_key_token_stream
                            #(#filter_unique_parameters_other_columns_token_stream)*
                        }
                    };
                    let query_string_token_stream = {
                        let additional_parameters_modification_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                            true => None,
                            false => {
                                let field_ident = field.ident.clone()
                                    .unwrap_or_else(|| {
                                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                    });
                                let field_handle_token_stream = {
                                    let field_handle_stringified = format!("{field_ident}_handle");
                                    field_handle_stringified.parse::<proc_macro2::TokenStream>()
                                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                };
                                let handle_token_stream = {
                                    let handle_stringified = format!("\"{field_ident} = ${{increment}}\"");
                                    handle_stringified.parse::<proc_macro2::TokenStream>()
                                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                };
                                Some(quote::quote!{
                                    if let Some(value) = &#field_handle_token_stream {
                                        match #crate_server_postgres_bind_query_bind_query_try_increment_token_stream(
                                            value,
                                            &mut increment
                                        ) {
                                            Ok(_) => {
                                                let handle = format!(#handle_token_stream);
                                                match additional_parameters.is_empty() {
                                                    true => {
                                                        additional_parameters.push_str(&handle);
                                                    },
                                                    false => {
                                                        additional_parameters.push_str(&format!(" AND {handle}"));
                                                    },
                                                }
                                            },
                                            Err(e) => {
                                                return #try_delete_many_with_body_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                            },
                                        }
                                    }
                                })
                            },
                        });
                        let additional_parameters_id_modification_token_stream = {
                            let handle_token_stream = {
                                let handle_stringified = format!("\" {id_field_ident} {in_name_stringified} ({{}})\"");
                                handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            let additional_parameters_and_token_stream = {
                                let additional_parameters_and_stringified = format!("\" {and_name_stringified}\"");
                                additional_parameters_and_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {additional_parameters_and_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            quote::quote!{
                                if let Some(#id_field_ident) = &#parameters_lower_case_token_stream.#payload_lower_case_token_stream.#id_field_ident {
                                    if let false = additional_parameters.is_empty() {
                                        additional_parameters.push_str(#additional_parameters_and_token_stream);
                                    }
                                    additional_parameters.push_str(&format!(
                                        #handle_token_stream,
                                        {
                                            let mut additional_parameters = std::string::String::default();
                                            for element in #id_field_ident {
                                                match #crate_server_postgres_bind_query_bind_query_try_increment_token_stream(
                                                    element,
                                                    &mut increment,
                                                ) {
                                                    Ok(_) => {
                                                        additional_parameters.push_str(&format!("${increment},"));
                                                    }
                                                    Err(e) => {
                                                        return #try_delete_many_with_body_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                                    }
                                                }
                                            }
                                            additional_parameters.pop();
                                            additional_parameters
                                        }
                                    ));
                                }
                            }
                        };
                        let handle_token_stream = {
                            let handle_stringified = format!("\"{delete_name_stringified} {from_name_stringified} {table_name_stringified} {where_name_stringified} {{}}\"");
                            handle_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{
                            format!(
                                #handle_token_stream,
                                {
                                    #increment_initialization_token_stream
                                    let mut additional_parameters = std::string::String::default();
                                    #(#additional_parameters_modification_token_stream)*
                                    #additional_parameters_id_modification_token_stream
                                    additional_parameters
                                }
                            )
                        }
                    };
                    let binded_query_token_stream = {
                        let binded_query_modifications_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                            true => None,
                            false => {
                                let field_ident = field.ident.clone()
                                    .unwrap_or_else(|| {
                                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                    });
                                let field_handle_token_stream = {
                                    let field_handle_stringified = format!("{field_ident}_handle");
                                    field_handle_stringified.parse::<proc_macro2::TokenStream>()
                                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                };
                                Some(quote::quote!{
                                    if let Some(value) = #field_handle_token_stream {
                                        query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(value, query);
                                    }
                                })
                            },
                        });
                        let binded_query_id_modifications_token_stream = quote::quote!{
                            if let Some(#id_field_ident) = #parameters_lower_case_token_stream.#payload_lower_case_token_stream.#id_field_ident {
                                for element in #id_field_ident {
                                    query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(element, query);
                                }
                            }
                        };
                        quote::quote!{
                            let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                            #(#binded_query_modifications_token_stream)*
                            #binded_query_id_modifications_token_stream
                            query
                        }
                    };
                    let generate_postgres_execute_query_token_stream = crate::generate_postgres_execute_query::generate_postgres_execute_query(
                        &query_string_name_token_stream,
                        &query_string_token_stream,
                        &binded_query_name_token_stream,
                        &binded_query_token_stream,
                        &acquire_pool_and_connection_token_stream,
                        &pg_connection_token_stream,
                        &try_delete_many_with_body_response_variants_token_stream,
                        &desirable_token_stream,
                        &from_log_and_return_error_token_stream,
                    );
                    quote::quote!{
                        #filter_unique_parameters_token_stream
                        #generate_postgres_execute_query_token_stream
                    }
                };
                quote::quote!{
                    #check_for_none_token_stream
                    match (#(#parameters_match_token_stream),*) {
                        (#(#parameters_match_primary_key_some_other_none_token_stream),*) => {
                            #generate_postgres_transaction_token_stream
                        }
                        _ => {
                            #generate_postgres_execute_query_token_stream
                        }
                    }
                }
            };
            // println!("{prepare_and_execute_query_token_stream}");
            quote::quote!{
                pub async fn #delete_many_with_body_lower_case_token_stream<'a>(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#delete_many_with_body_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #delete_many_with_body_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #delete_many_with_body_payload_camel_case_token_stream,
                            #try_delete_many_with_body_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #prepare_and_execute_query_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #try_delete_many_with_body_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{delete_many_with_body_token_stream}");
    let delete_many_token_stream = {
        let delete_many_name_camel_case_stringified = "DeleteMany";
        let delete_many_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&delete_many_name_camel_case_stringified.to_string());
        let delete_many_parameters_camel_case_token_stream = {
            let delete_many_parameters_camel_case_stringified = format!("{delete_many_name_camel_case_stringified}{parameters_camel_case_stringified}");
            delete_many_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_many_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let delete_many_query_camel_case_token_stream = {
            let delete_many_query_camel_case_stringified = format!("{delete_many_name_camel_case_stringified}{query_camel_case_stringified}");
            delete_many_query_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_many_query_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let delete_many_query_for_url_encoding_camel_case_token_stream = {
            let delete_many_query_for_url_encoding_camel_case_stringified = format!("{delete_many_name_camel_case_stringified}{query_camel_case_stringified}{for_url_encoding_camel_case_stringified}");
            delete_many_query_for_url_encoding_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_many_query_for_url_encoding_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_delete_many_error_named_camel_case_token_stream = {
            let try_delete_many_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{delete_many_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_delete_many_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_many_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_delete_many_response_variants_token_stream = {
            let try_delete_many_response_variants_stringified = format!("{try_camel_case_stringified}{delete_many_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_delete_many_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_many_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let prepare_and_execute_query_error_token_stream = {
            let error_path_stringified = format!("{try_camel_case_stringified}{delete_many_name_camel_case_stringified}");
            error_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_deserialize_token_stream
                pub struct #delete_many_parameters_camel_case_token_stream {
                    pub #query_lower_case_token_stream: #delete_many_query_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let query_token_stream = {
            let query_id_field_token_stream = quote::quote!{
                pub #id_field_ident: Option<Vec<std::string::String>>,//crate::server::postgres::bigserial_ids::BigserialIds
            };
            let fields_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    Some(quote::quote!{
                        pub #field_ident: Option<crate::server::routes::helpers::strings_deserialized_from_string_splitted_by_comma::StringsDeserializedFromStringSplittedByComma>
                    })
                },
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #delete_many_query_camel_case_token_stream {
                    #query_id_field_token_stream
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{query_token_stream}");
        let query_for_url_encoding_token_stream = {
            let fields_for_url_encoding_with_excluded_id_token_stream = fields_named.iter().map(|field|{
                let field_ident = field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    pub #field_ident: Option<std::string::String>
                }
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #delete_many_query_for_url_encoding_camel_case_token_stream {
                    #(#fields_for_url_encoding_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{query_for_url_encoding_token_stream}");
        let impl_std_convert_from_delete_many_query_for_delete_many_query_for_url_encoding_token_stream = {
            let impl_std_convert_from_delete_many_query_for_delete_many_query_for_url_encoding_id_token_stream = quote::quote!{
                let #id_field_ident = value.#id_field_ident.map(|value| {
                    #crate_common_serde_urlencoded_serde_urlencoded_parameter_serde_urlencoded_parameter_token_stream(value)
                });
            };
            let impl_std_convert_from_delete_many_query_for_delete_many_query_for_url_encoding_others_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    Some(quote::quote!{
                        let #field_ident = value.#field_ident.map(|value| {
                            #crate_common_serde_urlencoded_serde_urlencoded_parameter_serde_urlencoded_parameter_token_stream(value)
                        });
                    })
                },
            });
            let fields_idents_token_stream = fields_named.iter().map(|field|{
                let field_ident = field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{#field_ident}
            });
            quote::quote!{
                impl std::convert::From<#delete_many_query_camel_case_token_stream> for #delete_many_query_for_url_encoding_camel_case_token_stream {
                    fn from(value: #delete_many_query_camel_case_token_stream) -> Self {
                        #impl_std_convert_from_delete_many_query_for_delete_many_query_for_url_encoding_id_token_stream
                        #(#impl_std_convert_from_delete_many_query_for_delete_many_query_for_url_encoding_others_token_stream)*
                        Self { #(#fields_idents_token_stream),* }
                    }
                }
            }
        };
        // println!("{impl_std_convert_from_delete_many_query_for_delete_many_query_for_url_encoding_token_stream}");
        let try_delete_many_error_named_token_stream = {
            let try_delete_many_request_error_camel_case_token_stream = {
                let try_delete_many_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{delete_many_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_delete_many_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_many_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_delete_many_error_named_camel_case_token_stream {
                    #query_encode_variant_token_stream,
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_delete_many_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{try_delete_many_error_named_token_stream}");
        let http_request_token_stream = {
            let try_delete_many_lower_case_token_stream = {
                let try_delete_many_lower_case_stringified = format!("{try_lower_case_stringified}_{delete_many_name_lower_case_stringified}");
                try_delete_many_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_many_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{delete_many_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let url_handle_token_stream = {
                let url_handle_stringified = format!("\"{{}}/{table_name_stringified}?{{}}\"");//todo where
                url_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {url_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_delete_many_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #delete_many_parameters_camel_case_token_stream,
                ) -> Result<(), #try_delete_many_error_named_camel_case_token_stream> {
                    let encoded_query = match serde_urlencoded::to_string(#delete_many_query_for_url_encoding_camel_case_token_stream::from(#parameters_lower_case_token_stream.query)) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_delete_many_error_named_camel_case_token_stream::#query_encode_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        #url_handle_token_stream,
                        #server_location_name_token_stream,
                        encoded_query
                    );
                    // println!("{}", url);
                    match #tvfrr_extraction_logic_token_stream(
                        #reqwest_client_new_token_stream
                        .delete(&url)
                        #project_commit_header_addition_token_stream
                        .send(),
                    )
                    .await
                    {
                        Ok(value) => Ok(value),
                        Err(e) => Err(#try_delete_many_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let delete_many_lower_case_token_stream = delete_many_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_many_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let prepare_and_execute_query_token_stream = {
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &prepare_and_execute_query_error_token_stream,
                    &error_log_call_token_stream,
                    &try_delete_many_response_variants_token_stream,
                );
                // let prepare_and_execute_query_response_variants_token_stream = &try_delete_many_response_variants_token_stream;
                let query_part = crate::check_for_none::QueryPart::QueryParameters;
                let check_for_none_token_stream = crate::check_for_none::check_for_none(
                    &fields_named,
                    &id_field,
                    &proc_macro_name_ident_stringified,
                    dot_space,
                    &try_delete_many_response_variants_token_stream,
                    query_part,
                    false
                );
                // println!("{check_for_none_token_stream}");
                let parameters_match_token_stream = fields_named.iter().map(|field| {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    quote::quote!{
                        &#parameters_lower_case_token_stream.#query_lower_case_token_stream.#field_ident
                    }
                });
                let parameters_match_primary_key_some_other_none_token_stream = fields_named.iter().map(|field| {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    match field_ident == id_field_ident {
                        true => quote::quote!{Some(#id_field_ident)},
                        false => quote::quote!{None}
                    }
                });
                let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                    &from_log_and_return_error_token_stream,
                    &pg_connection_token_stream
                );
                let generate_postgres_transaction_token_stream = {
                    let filter_unique_parameters_token_stream = {
                        let filter_unique_parameters_primary_key_token_stream = quote::quote!{
                            let #not_unique_primary_keys_name_token_stream = {
                                let mut vec = Vec::with_capacity(#id_field_ident.len());
                                let mut #not_unique_primary_keys_name_token_stream = Vec::with_capacity(#id_field_ident.len());
                                for element in #id_field_ident {
                                    let handle = element;
                                    match vec.contains(&handle) {
                                        true => {
                                            #not_unique_primary_keys_name_token_stream.push(element.clone());
                                        }
                                        false => {
                                            vec.push(element);
                                        }
                                    }
                                }
                                #not_unique_primary_keys_name_token_stream
                            };
                            if let false = #not_unique_primary_keys_name_token_stream.is_empty() {
                                let error = #prepare_and_execute_query_error_token_stream::#not_unique_primery_key_token_stream;
                                #error_log_call_token_stream
                                return #try_delete_many_response_variants_token_stream::from(error);
                            }
                        };
                        quote::quote!{
                            #filter_unique_parameters_primary_key_token_stream
                        }
                    };
                    let expected_updated_primary_keys_token_stream = {
                        quote::quote!{
                            #id_field_ident
                            .iter()
                            .map(|element| element.clone()) //todo - maybe its not a good idea to remove .clone here coz in macro dont know what type
                            .collect::<Vec<#id_field_type>>()
                        }
                    };
                    let query_string_primary_key_some_other_none_token_stream = {
                        let handle_stringified = format!("\"{delete_name_stringified} {from_name_stringified} {table_name_stringified} {where_name_stringified} {id_field_ident} {in_name_stringified} ({select_name_stringified} {unnest_name_stringified}($1)){returning_id_stringified}\"");
                        handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    let binded_query_primary_key_some_other_none_token_stream = quote::quote!{
                        let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                        query = query.bind(
                            #id_field_ident.clone()//todo remove .clone
                            .into_iter()
                            .map(|element| element.clone())
                            .collect::<Vec<#id_field_type>>()
                        );
                        query
                    };
                    let generate_postgres_transaction_token_stream = crate::generate_postgres_transaction::generate_postgres_transaction(
                        &expected_updated_primary_keys_token_stream,
                        &query_string_name_token_stream,
                        &query_string_primary_key_some_other_none_token_stream,
                        &binded_query_primary_key_some_other_none_token_stream,
                        &acquire_pool_and_connection_token_stream,
                        &use_sqlx_acquire_token_stream,
                        &pg_connection_token_stream,
                        &begin_token_stream,
                        &binded_query_name_token_stream,
                        &use_futures_try_stream_ext_token_stream,
                        &query_and_rollback_failed_token_stream,
                        &primary_key_try_from_sqlx_row_name_token_stream,
                        &from_log_and_return_error_token_stream,
                        &rollback_error_name_token_stream,
                        &primary_key_from_row_and_failed_rollback_token_stream,
                        &non_existing_primary_keys_name_token_stream,
                        &expected_updated_primary_keys_name_token_stream,
                        &primary_key_vec_name_token_stream,
                        &rollback_token_stream,
                        &non_existing_primary_keys_token_stream,
                        &non_existing_primary_keys_and_failed_rollback_token_stream,
                        &postgres_transaction_token_stream,
                        &commit_token_stream,
                        &try_delete_many_response_variants_token_stream,
                        &desirable_token_stream,
                        &prepare_and_execute_query_error_token_stream,
                        &commit_failed_token_stream,
                        &error_log_call_token_stream,
                    );
                    quote::quote!{
                        #filter_unique_parameters_token_stream
                        #generate_postgres_transaction_token_stream
                    }
                };
                let generate_postgres_execute_query_token_stream = {
                    let filter_unique_parameters_token_stream = {
                        let filter_unique_parameters_primary_key_token_stream = quote::quote!{
                            if let Some(#id_field_ident) = &#parameters_lower_case_token_stream.#query_lower_case_token_stream.#id_field_ident {
                                let #not_unique_primary_keys_name_token_stream = {
                                    let mut vec = Vec::with_capacity(#id_field_ident.len());
                                    let mut #not_unique_primary_keys_name_token_stream = Vec::with_capacity(#id_field_ident.len());
                                    for element in #id_field_ident {
                                        let handle = element;
                                        match vec.contains(&handle) {
                                            true => {
                                                #not_unique_primary_keys_name_token_stream.push(element.clone());
                                            }
                                            false => {
                                                vec.push(element);
                                            }
                                        }
                                    }
                                    #not_unique_primary_keys_name_token_stream
                                };
                                if let false = #not_unique_primary_keys_name_token_stream.is_empty() {
                                    let error = #prepare_and_execute_query_error_token_stream::#not_unique_primery_key_token_stream;
                                    #error_log_call_token_stream
                                    return #try_delete_many_response_variants_token_stream::from(error);
                                }
                            }
                        };
                        //todo make standart for query and body
                        let filter_unique_parameters_other_columns_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                            true => None,
                            false => {
                                let field_ident = field.ident.clone()
                                    .unwrap_or_else(|| {
                                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                    });
                                let field_handle_token_stream = {
                                    let field_handle_stringified = format!("{field_ident}_handle");
                                    field_handle_stringified.parse::<proc_macro2::TokenStream>()
                                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                };
                                let not_unique_field_vec_lower_case_token_stream = {
                                    let not_unique_field_vec_lower_case_stringified = format!("not_unique_{field_ident}_vec");
                                    not_unique_field_vec_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {not_unique_field_vec_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                };
                                let not_unique_field_vec_vec_pascal_token_stream = {
                                    let not_unique_field_vec_pascal_stringified = format!(
                                        "NotUnique{}Vec",
                                        {
                                            use convert_case::Casing;
                                            field_ident.to_string().to_case(convert_case::Case::Pascal)
                                        }
                                    );
                                    not_unique_field_vec_pascal_stringified.parse::<proc_macro2::TokenStream>()
                                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {not_unique_field_vec_pascal_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                };
                                Some(quote::quote!{
                                    let #field_handle_token_stream = match #parameters_lower_case_token_stream.#query_lower_case_token_stream.#field_ident {
                                        Some(value) => {
                                            let is_unique = {
                                                let mut vec = Vec::with_capacity(value.0.len());
                                                let mut is_unique = true;
                                                for element in &value.0 {
                                                    match vec.contains(&element) {
                                                        true => {
                                                            is_unique = false;
                                                            break;
                                                        }
                                                        false => {
                                                            vec.push(element);
                                                        }
                                                    }
                                                }
                                                is_unique
                                            };
                                            match is_unique {
                                                true => Some(value),
                                                false => {
                                                    let #not_unique_field_vec_lower_case_token_stream = {
                                                        let mut vec = Vec::with_capacity(value.0.len());
                                                        let mut #not_unique_field_vec_lower_case_token_stream = Vec::with_capacity(value.0.len());
                                                        for element in value.0 {
                                                            match vec.contains(&element) {
                                                                true => {
                                                                    #not_unique_field_vec_lower_case_token_stream.push(element);
                                                                }
                                                                false => {
                                                                    vec.push(element);
                                                                }
                                                            }
                                                        }
                                                        #not_unique_field_vec_lower_case_token_stream
                                                    };
                                                    let error = #prepare_and_execute_query_error_token_stream::#not_unique_field_vec_vec_pascal_token_stream {
                                                        #not_unique_field_vec_lower_case_token_stream,
                                                        #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                                    };
                                                    #error_log_call_token_stream
                                                    return #try_delete_many_response_variants_token_stream::from(error);
                                                }
                                            }
                                        }
                                        None => None,
                                    };
                                })
                            },
                        });
                        quote::quote!{
                            #filter_unique_parameters_primary_key_token_stream
                            #(#filter_unique_parameters_other_columns_token_stream)*
                        }
                    };
                    let query_string_token_stream = {
                        let additional_parameters_modification_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                            true => None,
                            false => {
                                let field_ident = field.ident.clone()
                                    .unwrap_or_else(|| {
                                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                    });
                                let handle_token_stream = {
                                    let handle_stringified = format!("\"{field_ident} = ${{increment}}\"");
                                    handle_stringified.parse::<proc_macro2::TokenStream>()
                                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                };
                                let field_handle_token_stream = {
                                    let field_handle_stringified = format!("{field_ident}_handle");
                                    field_handle_stringified.parse::<proc_macro2::TokenStream>()
                                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                };
                                Some(quote::quote!{
                                    if let Some(value) = &#field_handle_token_stream {
                                        for _ in &value.0 {
                                            match increment.checked_add(1) {
                                                Some(incr) => {
                                                    increment = incr;
                                                    let handle = format!(#handle_token_stream);
                                                    match additional_parameters.is_empty() {
                                                        true => {
                                                            additional_parameters.push_str(&handle);
                                                        }
                                                        false => {
                                                            additional_parameters.push_str(&format!(" or {handle}"));//todo
                                                        }
                                                    }
                                                },
                                                None => {
                                                    return #try_delete_many_response_variants_token_stream::BindQuery {
                                                        checked_add: crate::server::postgres::bind_query::TryGenerateBindIncrementsErrorNamed::CheckedAdd { //todo remove it? refactor it?
                                                            checked_add: std::string::String::from("checked_add is None"), 
                                                            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream, 
                                                        }.into_serialize_deserialize_version(),
                                                        #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                                    };
                                                },
                                            }
                                        }
                                    }
                                })
                            },
                        });
                        let additional_parameters_id_modification_token_stream = {
                            let handle_token_stream = {
                                let handle_stringified = format!("\" {id_field_ident} {in_name_stringified} ({{}})\"");
                                handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            let additional_parameters_empty_handle_token_stream = {
                                let additional_parameters_empty_handle_stringified = format!("\" {and_name_stringified}\"");
                                additional_parameters_empty_handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {additional_parameters_empty_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            quote::quote!{
                                if let Some(id) = &#parameters_lower_case_token_stream.#query_lower_case_token_stream.#id_field_ident {
                                    if let false = additional_parameters.is_empty() {
                                        additional_parameters.push_str(#additional_parameters_empty_handle_token_stream);//todo
                                    }
                                    additional_parameters.push_str(&format!(
                                        #handle_token_stream,
                                        {
                                            let mut additional_parameters = std::string::String::default(); 
                                            for element in #id_field_ident {
                                                match #crate_server_postgres_bind_query_bind_query_try_increment_token_stream(element, &mut increment) {
                                                    Ok(_) => {
                                                        additional_parameters.push_str(&format!("${increment},"));
                                                    } 
                                                    Err(e) => {
                                                        return #try_delete_many_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                                    }
                                                }
                                            } 
                                            additional_parameters.pop(); 
                                            additional_parameters
                                        }
                                    ));
                                }
                            }
                        };
                        let handle_token_stream = {
                            let handle_stringified = format!("\"{delete_name_stringified} {from_name_stringified} {table_name_stringified} {where_name_stringified} {{}}\"");
                            handle_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{
                            format!(
                                #handle_token_stream,
                                {
                                    #increment_initialization_token_stream
                                    let mut additional_parameters = std::string::String::default();
                                    #(#additional_parameters_modification_token_stream)*
                                    #additional_parameters_id_modification_token_stream
                                    additional_parameters
                                }
                            )
                        }
                    };
                    let binded_query_token_stream = {
                        let binded_query_modifications_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                            true => None,
                            false => {
                                let field_ident = field.ident.clone()
                                    .unwrap_or_else(|| {
                                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                    });
                                let field_handle_token_stream = {
                                    let field_handle_stringified = format!("{field_ident}_handle");
                                    field_handle_stringified.parse::<proc_macro2::TokenStream>()
                                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                };
                                Some(quote::quote!{
                                    if let Some(value) = #field_handle_token_stream {
                                        query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(value, query);
                                    }
                                })
                            },
                        });
                        let binded_query_id_modifications_token_stream = quote::quote!{
                            if let Some(#id_field_ident) = #parameters_lower_case_token_stream.#query_lower_case_token_stream.#id_field_ident {
                                for element in #id_field_ident {
                                    query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(element, query);
                                }
                            }
                        };
                        quote::quote!{
                            let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                            #(#binded_query_modifications_token_stream)*
                            #binded_query_id_modifications_token_stream
                            query
                        }
                    };
                    let generate_postgres_execute_query_token_stream = crate::generate_postgres_execute_query::generate_postgres_execute_query(
                        &query_string_name_token_stream,
                        &query_string_token_stream,
                        &binded_query_name_token_stream,
                        &binded_query_token_stream,
                        &acquire_pool_and_connection_token_stream,
                        &pg_connection_token_stream,
                        &try_delete_many_response_variants_token_stream,
                        &desirable_token_stream,
                        &from_log_and_return_error_token_stream,
                    );
                    quote::quote!{
                        #filter_unique_parameters_token_stream
                        #generate_postgres_execute_query_token_stream
                    }
                };
                quote::quote!{
                    #check_for_none_token_stream
                    match (#(#parameters_match_token_stream),*) {
                        (#(#parameters_match_primary_key_some_other_none_token_stream),*) => {
                            #generate_postgres_transaction_token_stream
                        }
                        _ => {
                            #generate_postgres_execute_query_token_stream
                        }
                    }
                }
            };
            quote::quote!{
                pub async fn #delete_many_lower_case_token_stream<'a>(
                    #query_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_query_token_stream<#delete_many_query_camel_case_token_stream>,
                        #axum_extract_rejection_query_rejection_token_stream,
                    >,
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #delete_many_parameters_camel_case_token_stream {
                        #query_lower_case_token_stream: match #crate_server_routes_helpers_query_extractor_error_query_value_result_extractor_token_stream::<
                            #delete_many_query_camel_case_token_stream,
                            #try_delete_many_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#query_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #prepare_and_execute_query_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #query_token_stream
            #query_for_url_encoding_token_stream
            #impl_std_convert_from_delete_many_query_for_delete_many_query_for_url_encoding_token_stream
            #try_delete_many_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{delete_many_token_stream}");
    let gen = quote::quote! {
        #table_name_declaration_token_stream
        #struct_options_token_stream
        #from_ident_for_ident_options_token_stream
        #(#structs_variants_token_stream)*
        #(#structs_variants_impl_from_token_stream)*
        #column_token_stream
        #column_select_token_stream
        #primary_key_try_from_sqlx_row_token_stream
        //
        #primary_key_uuid_wrapper_try_from_sqlx_row_token_stream
        //
        #order_by_wrapper_token_stream
        #impl_crate_common_serde_urlencoded_serde_urlencoded_parameter_for_ident_order_by_wrapper_token_stream
        #deserialize_ident_order_by_token_stream
        #allow_methods_token_stream
        #ident_column_read_permission_token_stream

        #create_many_token_stream
        #create_one_token_stream
        #read_one_token_stream
        #read_many_with_body_token_stream
        #read_many_token_stream
        #update_one_token_stream
        #update_many_token_stream
        #delete_one_token_stream
        // #delete_many_with_body_token_stream
        // #delete_many_token_stream
    };
    // if ident == "" {
    //    println!("{gen}");
    // }
    gen.into()
}

// enum Operation {
//     // CreateMany,
//     // Create,
//     DeleteOne,
//     // DeleteManyWithBody,
//     // DeleteMany,
//     // ReadOne,
//     // ReadManyWithBody,
//     // ReadMany,
//     UpdateOne,
//     // UpdateMany
// }

// impl std::fmt::Display for Operation {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match self {
//             // Self::CreateMany => write!(f, "create_many"),
//             // Self::Create => write!(f, "create"),
//             Self::DeleteOne => write!(f, "delete_one"),
//             // Self::DeleteManyWithBody => write!(f, "delete_many_with_body"),
//             // Self::DeleteMany => write!(f, "delete"),
//             // Self::ReadOne => write!(f, "read_one"),
//             // Self::ReadManyWithBody => write!(f, "read_many_with_body"),
//             // Self::ReadMany => write!(f, "read"),
//             Self::UpdateOne => write!(f, "update_one"),
//             // Self::UpdateMany => write!(f, "update"),
//         }
//     }
// }


// fn generate_create_or_replace_function_token_stream(
//     ident_lower_case_stringified: &std::string::String,
//     operation: &crate::Operation,
//     proc_macro_name_ident_stringified: &std::string::String,
//     fields_named: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
//     id_field: &syn::Field,
//     payload_lower_case_token_stream: &proc_macro2::TokenStream
// ) -> proc_macro2::TokenStream {
//     let create_or_replace_function_name_original_token_stream = {
//         let create_or_replace_function_name_original_stringified =
//             format!("\"{ident_lower_case_stringified}_{operation}\"");
//         create_or_replace_function_name_original_stringified.parse::<proc_macro2::TokenStream>()
//         .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_or_replace_function_name_original_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
//     };
//     let create_or_replace_function_name_additions_token_stream = fields_named.iter().filter_map(|field|match field == id_field {
//         true => None,
//         false => {
//             let field_ident = field.ident.clone()
//                 .unwrap_or_else(|| {
//                     panic!("{proc_macro_name_ident_stringified} field.ident is None")
//                 });
//             let format_value_token_stream = {
//                 let format_value_stringified = format!("\"_{field_ident}\"");
//                 format_value_stringified.parse::<proc_macro2::TokenStream>()
//                 .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {format_value_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
//             };
//             Some(quote::quote!{
//                 if self.#payload_lower_case_token_stream.#field_ident.is_some() {
//                     value.push_str(&format!(#format_value_token_stream));
//                 }
//             })
//         },
//     });
//     quote::quote! {
//         let mut value = std::string::String::from(#create_or_replace_function_name_original_token_stream);
//         #(#create_or_replace_function_name_additions_token_stream)*
//         value
//     }
// }

// `DO` blocks cannot use bound parameters.  If you need to pass in values then you can create a temporary function and call that instead, though it's a bit more of a hassle.

// #[derive(strum_macros::Display)]//strum_macros::EnumIter, 
// enum Attribute {
//     GeneratePostgresqlCrudPrimaryKey,
// }

// impl Attribute {
//     pub fn to_str(&self) -> &str {
//         match self {
//             Attribute::GeneratePostgresqlCrudPrimaryKey => "generate_postgresql_crud_primary_key",
//         }
//     }
//     pub fn attribute_view(&self) -> String {
//         self.to_str().to_string()
//     }
// }
