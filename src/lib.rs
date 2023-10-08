mod column_names_factorial;
mod check_for_none;
mod acquire_pool_and_connection;
mod from_log_and_return_error;
//todo unnest allows to put multiple data(arrays) in sqlx compile time query(brookzerker twitch\youtube tutorial)
#[proc_macro_derive(
    GeneratePostgresqlCrud,
    attributes(
        generate_postgresql_crud_id,
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
    // let ident_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&ident.to_string());
    let proc_macro_name_ident_stringified = format!("{proc_macro_name} {ident}");
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
        let id_attr_name = "generate_postgresql_crud_id";
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
                impl crate::common::serde_urlencoded::SerdeUrlencodedParameter for #column_select_ident_token_stream {
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
                let write_ident_token_stream = column_variant.iter().map(|field|{
                    let field_ident = field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
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
            let sqlx_decode_decode_and_sqlx_types_type_token_stream = fields_named.iter().map(|field|{
                let field_type = &field.ty;
                quote::quote! {
                    Option<#field_type>: #sqlx_decode_decode_database_token_stream,
                    Option<#field_type>: #sqlx_types_type_database_token_stream,
                }
            });
            quote::quote! {
                impl #column_select_ident_token_stream {
                    fn #options_try_from_sqlx_row_name_token_stream<'a, R: sqlx::Row>(
                        &self,
                        row: &'a R,
                    ) -> sqlx::Result<#struct_options_ident_token_stream>
                    where
                        #std_primitive_str_sqlx_column_index_token_stream
                        #(#sqlx_decode_decode_and_sqlx_types_type_token_stream)*
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
    let query_encode_token_stream = quote::quote!{QueryEncode};
    let url_encoding_token_stream = quote::quote!{url_encoding};
    let serde_urlencoded_ser_error_token_stream = quote::quote!{serde_urlencoded::ser::Error};
    let serde_json_to_string_token_stream = quote::quote!{serde_json::to_string};
    let into_url_encoding_version_name_token_stream = quote::quote!{into_url_encoding_version}; 
    let url_encoding_camel_case_stringified = "UrlEncoding";
    let for_url_encoding_camel_case_stringified = format!("For{url_encoding_camel_case_stringified}");
    let payload_element_camel_case_stringified = format!("{payload_camel_case_stringified}Element");
    let prepare_and_execute_query_name_token_stream = quote::quote!{prepare_and_execute_query};
    let try_camel_case_stringified = "Try";
    let try_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&try_camel_case_stringified.to_string());
    let response_variants_camel_case_stringified = "ResponseVariants";
    let error_named_camel_case_stringified = "ErrorNamed";
    let tvfrr_extraction_logic_lower_case_stringified = "tvfrr_extraction_logic";
    let request_error_camel_case_stringified = "RequestError";
    let returning_stringified = "returning";
    let returning_id_stringified = format!(" {returning_stringified} {id_field_ident}");
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
    let error_named_derive_token_stream = quote::quote!{#[derive(Debug, thiserror::Error, error_occurence::ErrorOccurence)]};
    let parameters_derive_token_stream = quote::quote!{#[derive(Debug, serde::Deserialize)]};
    let path_derive_token_stream = quote::quote!{#[derive(Debug, serde::Deserialize)]};
    let query_derive_token_stream = quote::quote!{#[derive(Debug, serde::Serialize, serde::Deserialize)]};
    let payload_derive_token_stream = quote::quote!{#[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]};
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
    let crate_common_serde_urlencoded_serde_urlencoded_parameter_serde_urlencoded_parameter_token_stream = quote::quote!{crate::common::serde_urlencoded::SerdeUrlencodedParameter::serde_urlencoded_parameter};
    let fields_named_len = fields_named.len();
    let dot_space = ", ";
    // let pg_temp_stringified = "pg_temp";
    let pg_connection_token_stream = quote::quote!{pg_connection};
    let desirable_token_stream = quote::quote!{Desirable};
    let query_string_name_token_stream = quote::quote!{query_string};
    let binded_query_name_token_stream = quote::quote!{binded_query};
    let postgres_transaction_token_stream = quote::quote!{postgres_transaction_token_stream};
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
    let increment_initialization_token_stream = quote::quote!{let mut increment: u64 = 0;};
    let crate_server_postgres_constants_stringified = "crate::server::postgres::constants::";
    let crate_server_postgres_constants_update_name_token_stream = {
        let crate_server_postgres_constants_update_name_stringified = format!("{crate_server_postgres_constants_stringified}UPDATE_NAME");
        crate_server_postgres_constants_update_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_update_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_as_name_token_stream = {
        let crate_server_postgres_constants_as_name_stringified = format!("{crate_server_postgres_constants_stringified}AS_NAME");
        crate_server_postgres_constants_as_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_as_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_set_name_token_stream = {
        let crate_server_postgres_constants_set_name_stringified = format!("{crate_server_postgres_constants_stringified}SET_NAME");
        crate_server_postgres_constants_set_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_set_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_from_name_token_stream = {
        let crate_server_postgres_constants_from_name_stringified = format!("{crate_server_postgres_constants_stringified}FROM_NAME");
        crate_server_postgres_constants_from_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_from_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_insert_name_token_stream = {
        let crate_server_postgres_constants_insert_name_stringified = format!("{crate_server_postgres_constants_stringified}INSERT_NAME");
        crate_server_postgres_constants_insert_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_insert_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_into_name_token_stream = {
        let crate_server_postgres_constants_into_name_stringified = format!("{crate_server_postgres_constants_stringified}INTO_NAME");
        crate_server_postgres_constants_into_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_into_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_values_name_token_stream = {
        let crate_server_postgres_constants_values_name_stringified = format!("{crate_server_postgres_constants_stringified}VALUES_NAME");
        crate_server_postgres_constants_values_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_values_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_delete_name_token_stream = {
        let crate_server_postgres_constants_delete_name_stringified = format!("{crate_server_postgres_constants_stringified}DELETE_NAME");
        crate_server_postgres_constants_delete_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_delete_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_where_name_token_stream = {
        let crate_server_postgres_constants_where_name_stringified = format!("{crate_server_postgres_constants_stringified}WHERE_NAME");
        crate_server_postgres_constants_where_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_where_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_and_name_token_stream = {
        let crate_server_postgres_constants_and_name_stringified = format!("{crate_server_postgres_constants_stringified}AND_NAME");
        crate_server_postgres_constants_and_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_and_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_any_name_token_stream = {
        let crate_server_postgres_constants_any_name_stringified = format!("{crate_server_postgres_constants_stringified}ANY_NAME");
        crate_server_postgres_constants_any_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_any_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_array_name_token_stream = {
        let crate_server_postgres_constants_array_name_stringified = format!("{crate_server_postgres_constants_stringified}ARRAY_NAME");
        crate_server_postgres_constants_array_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_array_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_select_name_token_stream = {
        let crate_server_postgres_constants_select_name_stringified = format!("{crate_server_postgres_constants_stringified}SELECT_NAME");
        crate_server_postgres_constants_select_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_select_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_order_by_name_token_stream = {
        let crate_server_postgres_constants_order_by_name_stringified = format!("{crate_server_postgres_constants_stringified}ORDER_BY_NAME");
        crate_server_postgres_constants_order_by_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_order_by_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_limit_name_token_stream = {
        let crate_server_postgres_constants_limit_name_stringified = format!("{crate_server_postgres_constants_stringified}LIMIT_NAME");
        crate_server_postgres_constants_limit_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_limit_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_constants_offset_name_token_stream = {
        let crate_server_postgres_constants_offset_name_stringified = format!("{crate_server_postgres_constants_stringified}OFFSET_NAME");
        crate_server_postgres_constants_offset_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_constants_offset_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let create_batch_token_stream = {
        let create_batch_name_camel_case_stringified = "CreateBatch";
        let create_batch_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&create_batch_name_camel_case_stringified.to_string());
        let create_batch_parameters_camel_case_token_stream = {
            let create_batch_parameters_camel_case_stringified = format!("{create_batch_name_camel_case_stringified}{parameters_camel_case_stringified}");
            create_batch_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_batch_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let create_batch_payload_element_camel_case_token_stream = {
            let create_batch_payload_element_camel_case_stringified = format!("{create_batch_name_camel_case_stringified}{payload_element_camel_case_stringified}");
            create_batch_payload_element_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_batch_payload_element_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let create_batch_payload_camel_case_token_stream = quote::quote!{Vec<#create_batch_payload_element_camel_case_token_stream>};
        let try_create_batch_error_named_camel_case_token_stream = {
            let try_create_batch_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{create_batch_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_create_batch_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_batch_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_create_batch_response_variants_token_stream = {
            let try_create_batch_response_variants_stringified = format!("{try_camel_case_stringified}{create_batch_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_create_batch_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_batch_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
                pub struct #create_batch_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #create_batch_payload_camel_case_token_stream,
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
                #payload_derive_token_stream
                pub struct #create_batch_payload_element_camel_case_token_stream {
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let prepare_and_execute_query_token_stream = {
            let prepare_and_execute_query_error_token_stream = {
                let error_path_stringified = format!("{try_camel_case_stringified}{create_batch_name_camel_case_stringified}");
                error_path_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &try_create_batch_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
            );
            let query_string_token_stream = {
                let query_token_stream = {
                    let column_names = {
                        let fields_named_filtered = fields_named.iter().filter(|field|*field != &id_field).collect::<Vec<&syn::Field>>();
                        let fields_named_len = fields_named_filtered.len();
                        fields_named_filtered.iter().enumerate().fold(std::string::String::default(), |mut acc, (index, field)| {
                            let field_ident = field.ident.clone().unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                            let incremented_index = index + 1;
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
                    let query_stringified = format!("\"{{}} {{}} {{}}({column_names}) {{}} {{}}\"");
                    query_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let element_bind_increments_modificate_token_stream = {
                    let fields_named_filtered = fields_named.iter().filter(|field|*field != &id_field).collect::<Vec<&syn::Field>>();
                    let field_named_filtered_len = fields_named_filtered.len();
                    fields_named_filtered.iter().enumerate().map(|(index, field)|{
                        let field_ident = field.ident.clone()
                            .unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                        let possible_dot_space_format = match (index + 1) == field_named_filtered_len {
                            true => quote::quote!{"{value}"},
                            false => quote::quote!{"{value}, "},
                        };
                        quote::quote!{
                            match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(&element.#field_ident, &mut increment) {
                                Ok(value) => {
                                    element_bind_increments.push_str(&format!(#possible_dot_space_format));
                                },
                                Err(e) => {
                                    return #try_create_batch_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                },
                            }
                        }
                    }).collect::<Vec<proc_macro2::TokenStream>>()
                };
                quote::quote!{
                    format!(
                        #query_token_stream,
                        #crate_server_postgres_constants_insert_name_token_stream,
                        #crate_server_postgres_constants_into_name_token_stream,
                        ROUTE_NAME,
                        #crate_server_postgres_constants_values_name_token_stream,
                        {
                            #increment_initialization_token_stream
                            let mut bind_increments = std::string::String::default();
                            for element in &self.#payload_lower_case_token_stream {
                                bind_increments.push_str(&format!(
                                    "({}), ",
                                    {
                                        let mut element_bind_increments = std::string::String::default();
                                        #(#element_bind_increments_modificate_token_stream)*
                                        element_bind_increments
                                    }
                                ));
                            }
                            bind_increments.pop();
                            bind_increments.pop();
                            bind_increments
                        }
                    )
                }
            };
            let binded_query_token_stream = {
                let bind_value_to_query_modificate_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                    true => None,
                    false => {
                        let field_ident = field.ident.clone()
                            .unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                        Some(quote::quote!{
                            query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(element.#field_ident, query);  
                        })
                    },
                });
                quote::quote!{
                    let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                    for element in self.#payload_lower_case_token_stream {
                        #(#bind_value_to_query_modificate_token_stream)*
                    }
                    query
                }
            };
            quote::quote!{
                impl #create_batch_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        #app_info_state_name_token_stream: &#app_info_state_path,
                    ) -> #try_create_batch_response_variants_token_stream
                    {
                        let #query_string_name_token_stream = #query_string_token_stream;
                        // println!("{query_string}");
                        let #binded_query_name_token_stream = {
                            #binded_query_token_stream
                        };
                        #acquire_pool_and_connection_token_stream
                        match #binded_query_name_token_stream
                            .execute(#pg_connection_token_stream.as_mut())
                            .await
                        {
                            Ok(_) => {
                                //todo - is need to return rows affected?
                                #try_create_batch_response_variants_token_stream::#desirable_token_stream(())
                            }
                            Err(e) => {
                                #from_log_and_return_error_token_stream
                            }
                        }
                    }
                }
            }
        };
        // println!("{prepare_and_execute_query_token_stream}");
        let try_create_batch_error_named_token_stream = {
            let try_create_batch_request_error_camel_case_token_stream = {
                let try_create_batch_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{create_batch_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_create_batch_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_batch_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_create_batch_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_create_batch_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_create_batch_error_named_token_stream}");
        let http_request_token_stream = {
            let try_create_batch_lower_case_token_stream = {
                let try_create_batch_lower_case_stringified = format!("{try_lower_case_stringified}_{create_batch_name_lower_case_stringified}");
                try_create_batch_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_batch_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{create_batch_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_create_batch_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #create_batch_parameters_camel_case_token_stream,
                ) -> Result<(), #try_create_batch_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_create_batch_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        "{}/api/{}",
                        #server_location_name_token_stream,
                        ROUTE_NAME
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
                        Err(e) => Err(#try_create_batch_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let create_batch_lower_case_token_stream = create_batch_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_batch_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            quote::quote!{
                pub async fn #create_batch_lower_case_token_stream(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#create_batch_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #create_batch_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #create_batch_payload_camel_case_token_stream,
                            #try_create_batch_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    #parameters_lower_case_token_stream.#prepare_and_execute_query_name_token_stream(&#app_info_state_name_token_stream).await
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #prepare_and_execute_query_token_stream
            #try_create_batch_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{create_batch_token_stream}");
    let create_token_stream = {
        let create_name_camel_case_stringified = "Create";
        let create_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&create_name_camel_case_stringified.to_string());
        let create_parameters_camel_case_token_stream = {
            let create_parameters_camel_case_stringified = format!("{create_name_camel_case_stringified}{parameters_camel_case_stringified}");
            create_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let create_payload_camel_case_token_stream = {
            let create_payload_camel_case_stringified = format!("{create_name_camel_case_stringified}{payload_camel_case_stringified}");
            create_payload_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_payload_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_create_error_named_camel_case_token_stream = {
            let try_create_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{create_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_create_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_create_response_variants_token_stream = {
            let try_create_response_variants_stringified = format!("{try_camel_case_stringified}{create_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_create_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
                pub struct #create_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #create_payload_camel_case_token_stream,
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
                #payload_derive_token_stream
                pub struct #create_payload_camel_case_token_stream {
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let prepare_and_execute_query_token_stream = {
            let prepare_and_execute_query_error_token_stream = {
                let error_path_stringified = format!("{try_camel_case_stringified}{create_name_camel_case_stringified}");
                error_path_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &try_create_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
            );
            let query_string_token_stream = {
                let query_token_stream = {
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
                            let incremented_index = index + 1;
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
                    let query_stringified = format!("\"{{}} {{}} {{}}({column_names}) {{}} ({column_increments})\"");
                    query_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                quote::quote!{
                    format!(
                        #query_token_stream,
                        #crate_server_postgres_constants_insert_name_token_stream,
                        #crate_server_postgres_constants_into_name_token_stream,
                        ROUTE_NAME,
                        #crate_server_postgres_constants_values_name_token_stream
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
                        Some(quote::quote!{
                            query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(self.#payload_lower_case_token_stream.#field_ident, query);
                        })
                    },
                });
                quote::quote!{
                    let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                    #(#binded_query_modifications_token_stream)*
                    query
                }
            };
            quote::quote!{
                impl #create_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        #app_info_state_name_token_stream: &#app_info_state_path,
                    ) -> #try_create_response_variants_token_stream
                    {
                        let #query_string_name_token_stream = #query_string_token_stream;
                        // println!("{query_string}");
                        let #binded_query_name_token_stream = {
                            #binded_query_token_stream
                        };
                        #acquire_pool_and_connection_token_stream
                        match #binded_query_name_token_stream
                            .execute(#pg_connection_token_stream.as_mut())
                            .await
                        {
                            Ok(_) => {
                                //todo - is need to return rows affected?
                                #try_create_response_variants_token_stream::#desirable_token_stream(())
                            }
                            Err(e) => {
                                #from_log_and_return_error_token_stream
                            }
                        }
                    }
                }
            }
        };
        // println!("{prepare_and_execute_query_token_stream}");
        let try_create_error_named_token_stream = {
            let try_create_request_error_camel_case_token_stream = {
                let try_create_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{create_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_create_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_create_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_create_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_create_error_named_token_stream}");
        let http_request_token_stream = {
            let try_create_lower_case_token_stream = {
                let try_create_lower_case_stringified = format!("{try_lower_case_stringified}_{create_name_lower_case_stringified}");
                try_create_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_create_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{create_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_create_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #create_parameters_camel_case_token_stream,
                ) -> Result<(), #try_create_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_create_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        "{}/api/{}",
                        #server_location_name_token_stream,
                        ROUTE_NAME
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
                        Err(e) => Err(#try_create_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let create_lower_case_token_stream = create_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            quote::quote!{
                pub async fn #create_lower_case_token_stream(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#create_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #create_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #create_payload_camel_case_token_stream,
                            #try_create_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    #parameters_lower_case_token_stream.#prepare_and_execute_query_name_token_stream(&#app_info_state_name_token_stream).await
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #prepare_and_execute_query_token_stream
            #try_create_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{create_token_stream}");
    let delete_by_id_token_stream = {
        let delete_by_id_name_camel_case_stringified = "DeleteById";
        let delete_by_id_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&delete_by_id_name_camel_case_stringified.to_string());
        let delete_by_id_parameters_camel_case_token_stream = {
            let delete_by_id_parameters_camel_case_stringified = format!("{delete_by_id_name_camel_case_stringified}{parameters_camel_case_stringified}");
            delete_by_id_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_by_id_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let delete_by_id_path_camel_case_token_stream = {
            let delete_by_id_path_camel_case_stringified = format!("{delete_by_id_name_camel_case_stringified}{path_camel_case_stringified}");
            delete_by_id_path_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_by_id_path_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_delete_by_id_error_named_camel_case_token_stream = {
            let try_delete_by_id_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{delete_by_id_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_delete_by_id_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_by_id_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_delete_by_id_response_variants_token_stream = {
            let try_delete_by_id_response_variants_stringified = format!("{try_camel_case_stringified}{delete_by_id_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_delete_by_id_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_by_id_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #parameters_derive_token_stream
                pub struct #delete_by_id_parameters_camel_case_token_stream {
                    pub #path_lower_case_token_stream: #delete_by_id_path_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let path_token_stream = {
            quote::quote!{
                #path_derive_token_stream
                pub struct #delete_by_id_path_camel_case_token_stream {
                    pub #id_field_ident: crate::server::postgres::bigserial::Bigserial,//#id_field_type
                }
            }
        };
        // println!("{path_token_stream}");
        let prepare_and_execute_query_token_stream = {
            let prepare_and_execute_query_error_token_stream = {
                let error_path_stringified = format!("{try_camel_case_stringified}{delete_by_id_name_camel_case_stringified}");
                error_path_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &try_delete_by_id_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
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
                quote::quote!{
                    let mut query = format!(
                        "{} {} {} {} ",
                        #crate_server_postgres_constants_delete_name_token_stream,
                        #crate_server_postgres_constants_from_name_token_stream,
                        ROUTE_NAME,
                        #crate_server_postgres_constants_where_name_token_stream,
                    );
                    #additional_parameters_id_modification_token_stream
                    query.push_str(&format!(#returning_id_quotes_token_stream));
                    query
                }
            };
            let binded_query_token_stream = {
                let binded_query_modifications_token_stream = quote::quote!{
                    query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(self.#path_lower_case_token_stream.#id_field_ident, query);
                };
                quote::quote!{
                    let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                    #binded_query_modifications_token_stream
                    query
                }
            };
            quote::quote!{
                impl #delete_by_id_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        #app_info_state_name_token_stream: &#app_info_state_path,
                    ) -> #try_delete_by_id_response_variants_token_stream
                    {
                        let #query_string_name_token_stream = {
                            #query_string_token_stream
                        };
                        let #binded_query_name_token_stream = {
                            #binded_query_token_stream
                        };
                        #acquire_pool_and_connection_token_stream
                        match #binded_query_name_token_stream
                            .fetch_one(#pg_connection_token_stream.as_mut())
                            .await
                        {
                            Ok(row) => #try_delete_by_id_response_variants_token_stream::#desirable_token_stream(()),//todo - () as variable token stream
                            Err(e) => {
                                #from_log_and_return_error_token_stream;
                            }
                        }
                    }
                }
            }
        };
        // println!("{prepare_and_execute_query_token_stream}");
        let try_delete_by_id_error_named_token_stream = {
            let try_delete_by_id_request_error_camel_case_token_stream = {
                let try_delete_by_id_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{delete_by_id_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_delete_by_id_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_by_id_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_delete_by_id_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_delete_by_id_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{try_delete_by_id_error_named_token_stream}");
        let http_request_token_stream = {
            let try_delete_by_id_lower_case_token_stream = {
                let try_delete_by_id_lower_case_stringified = format!("{try_lower_case_stringified}_{delete_by_id_name_lower_case_stringified}");
                try_delete_by_id_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_by_id_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{delete_by_id_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_delete_by_id_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #delete_by_id_parameters_camel_case_token_stream,
                ) -> Result<(), #try_delete_by_id_error_named_camel_case_token_stream> {
                    let url = format!(
                        "{}/api/{}/{}",
                        #server_location_name_token_stream,
                        ROUTE_NAME,
                        #parameters_lower_case_token_stream.#path_lower_case_token_stream.id
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
                        Err(e) => Err(#try_delete_by_id_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let delete_by_id_lower_case_token_stream = delete_by_id_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_by_id_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            quote::quote!{
                pub async fn #delete_by_id_lower_case_token_stream<'a>(
                    #path_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_path_token_stream<#delete_by_id_path_camel_case_token_stream>,
                        #axum_extract_rejection_path_rejection_token_stream,
                    >,
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #delete_by_id_parameters_camel_case_token_stream {
                        #path_lower_case_token_stream: match #crate_server_routes_helpers_path_extractor_error_path_value_result_extractor_token_stream::<
                            #delete_by_id_path_camel_case_token_stream,
                            #try_delete_by_id_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#path_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    #parameters_lower_case_token_stream.#prepare_and_execute_query_name_token_stream(&#app_info_state_name_token_stream).await
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #path_token_stream
            #prepare_and_execute_query_token_stream
            #try_delete_by_id_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{delete_by_id_token_stream}");
    let delete_with_body_token_stream = {
        let delete_with_body_name_camel_case_stringified = "DeleteWithBody";
        let delete_with_body_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&delete_with_body_name_camel_case_stringified.to_string());
        let delete_with_body_parameters_camel_case_token_stream = {
            let delete_with_body_parameters_camel_case_stringified = format!("{delete_with_body_name_camel_case_stringified}{parameters_camel_case_stringified}");
            delete_with_body_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_with_body_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let delete_with_body_payload_camel_case_token_stream = {
            let delete_with_body_payload_camel_case_stringified = format!("{delete_with_body_name_camel_case_stringified}{payload_camel_case_stringified}");
            delete_with_body_payload_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_with_body_payload_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_delete_with_body_error_named_camel_case_token_stream = {
            let try_delete_with_body_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{delete_with_body_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_delete_with_body_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_with_body_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_delete_with_body_response_variants_token_stream = {
            let try_delete_with_body_response_variants_stringified = format!("{try_camel_case_stringified}{delete_with_body_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_delete_with_body_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_with_body_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #parameters_derive_token_stream
                pub struct #delete_with_body_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #delete_with_body_payload_camel_case_token_stream,
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
                #payload_derive_token_stream
                pub struct #delete_with_body_payload_camel_case_token_stream {
                    pub #id_field_ident: Option<Vec<crate::server::postgres::bigserial::Bigserial>>,//todo
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let prepare_and_execute_query_token_stream = {
            let prepare_and_execute_query_error_token_stream = {
                let error_path_stringified = format!("{try_camel_case_stringified}{delete_with_body_name_camel_case_stringified}");
                error_path_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let check_for_none_token_stream_excluding_primary_key = crate::check_for_none::check_for_none(
                &fields_named,
                &id_field,
                &proc_macro_name_ident_stringified,
                dot_space,
                &try_delete_with_body_response_variants_token_stream,
                crate::check_for_none::QueryPart::Payload,
                true
            );
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &try_delete_with_body_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
            );
            let query_string_token_stream = {
                let additional_parameters_id_modification_token_stream = {
                    let query_part_token_stream = {
                        let query_part_stringified = format!("\"{{prefix}} {id_field_ident} = {{}}({{}}[{{}}])\"");
                        query_part_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_part_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    quote::quote!{
                        if let Some(value) = &self.#payload_lower_case_token_stream.#id_field_ident {
                            let prefix = match additional_parameters.is_empty() {
                                true => #crate_server_postgres_constants_where_name_token_stream.to_string(),
                                false => format!(" {}", #crate_server_postgres_constants_and_name_token_stream),
                            };
                            let bind_increments = {
                                let mut bind_increments = std::string::String::default();
                                for element in value {
                                    match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(
                                        element,
                                        &mut increment
                                    ) {
                                        Ok(value) => {
                                            bind_increments.push_str(&format!("{value}, "));
                                        },
                                        Err(e) => {
                                            return #try_delete_with_body_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                        },
                                    }
                                }
                                bind_increments.pop();
                                bind_increments.pop();
                                bind_increments
                            };
                            additional_parameters.push_str(&format!(
                                #query_part_token_stream,
                                #crate_server_postgres_constants_any_name_token_stream,
                                #crate_server_postgres_constants_array_name_token_stream,
                                bind_increments
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
                        Some(quote::quote!{
                            if let Some(value) = &self.#payload_lower_case_token_stream.#field_ident {
                                let prefix = match additional_parameters.is_empty() {
                                    true => #crate_server_postgres_constants_where_name_token_stream.to_string(),
                                    false => format!(" {}", #crate_server_postgres_constants_and_name_token_stream),
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
                                                        bind_increments.push_str(&format!("{handle}"));
                                                    },
                                                    false => {
                                                        bind_increments.push_str(&format!("{} {handle}", element.conjuctive_operator));
                                                    },
                                                }
                                            },
                                            Err(e) => {
                                                return #try_delete_with_body_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
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
                quote::quote!{
                    format!(
                        "{} {} {} {}",
                        #crate_server_postgres_constants_delete_name_token_stream,
                        #crate_server_postgres_constants_from_name_token_stream,
                        ROUTE_NAME,
                        {
                            #increment_initialization_token_stream
                            let mut additional_parameters = std::string::String::default();
                            #additional_parameters_id_modification_token_stream
                            #(#additional_parameters_modification_token_stream)*
                            additional_parameters
                        }
                    )
                }
            };
            let binded_query_token_stream = {
                let binded_query_modifications_token_stream = fields_named.iter().map(|field|{
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    quote::quote!{
                        if let Some(values) = self.#payload_lower_case_token_stream.#field_ident {
                            for value in values {
                                query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                                    value, query,
                                );
                            }
                        }
                    }
                });
                quote::quote!{
                    let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                    #(#binded_query_modifications_token_stream)*
                    query
                }
            };
            quote::quote!{
                impl #delete_with_body_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        #app_info_state_name_token_stream: &#app_info_state_path,
                    ) -> #try_delete_with_body_response_variants_token_stream
                    {
                        #check_for_none_token_stream_excluding_primary_key
                        let #query_string_name_token_stream = #query_string_token_stream;
                        // println!("{query_string}");
                        let #binded_query_name_token_stream = {
                            #binded_query_token_stream
                        };
                        #acquire_pool_and_connection_token_stream
                        match #binded_query_name_token_stream
                            .execute(#pg_connection_token_stream.as_mut())
                            .await
                        {
                            Ok(_) => {
                                //todo - is need to return rows affected?
                                #try_delete_with_body_response_variants_token_stream::#desirable_token_stream(())
                            }
                            Err(e) => {
                                #from_log_and_return_error_token_stream
                            }
                        }
                    }
                }
            }
        };
        // println!("{prepare_and_execute_query_token_stream}");
        let try_delete_with_body_error_named_token_stream = {
            let try_delete_with_body_request_error_camel_case_token_stream = {
                let try_delete_with_body_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{delete_with_body_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_delete_with_body_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_with_body_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_delete_with_body_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_delete_with_body_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_delete_with_body_error_named_token_stream}");
        let http_request_token_stream = {
            let try_delete_with_body_lower_case_token_stream = {
                let try_delete_with_body_lower_case_stringified = format!("{try_lower_case_stringified}_{delete_with_body_name_lower_case_stringified}");
                try_delete_with_body_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_with_body_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{delete_with_body_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_delete_with_body_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #delete_with_body_parameters_camel_case_token_stream,
                ) -> Result<(), #try_delete_with_body_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_delete_with_body_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        "{}/api/{}/search",
                        #server_location_name_token_stream,
                        ROUTE_NAME
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
                        Err(e) => Err(#try_delete_with_body_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let delete_with_body_lower_case_token_stream = delete_with_body_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_with_body_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            quote::quote!{
                pub async fn #delete_with_body_lower_case_token_stream<'a>(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#delete_with_body_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #delete_with_body_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #delete_with_body_payload_camel_case_token_stream,
                            #try_delete_with_body_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    #parameters_lower_case_token_stream.#prepare_and_execute_query_name_token_stream(&#app_info_state_name_token_stream).await
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #prepare_and_execute_query_token_stream
            #try_delete_with_body_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{delete_with_body_token_stream}");
    let delete_token_stream = {
        let delete_name_camel_case_stringified = "Delete";
        let delete_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&delete_name_camel_case_stringified.to_string());
        let delete_parameters_camel_case_token_stream = {
            let delete_parameters_camel_case_stringified = format!("{delete_name_camel_case_stringified}{parameters_camel_case_stringified}");
            delete_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let delete_query_camel_case_token_stream = {
            let delete_query_camel_case_stringified = format!("{delete_name_camel_case_stringified}{query_camel_case_stringified}");
            delete_query_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_query_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let delete_query_for_url_encoding_camel_case_token_stream = {
            let delete_query_for_url_encoding_camel_case_stringified = format!("{delete_name_camel_case_stringified}{query_camel_case_stringified}{for_url_encoding_camel_case_stringified}");
            delete_query_for_url_encoding_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_query_for_url_encoding_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_delete_error_named_camel_case_token_stream = {
            let try_delete_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{delete_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_delete_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_delete_response_variants_token_stream = {
            let try_delete_response_variants_stringified = format!("{try_camel_case_stringified}{delete_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_delete_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        //
        let prepare_and_execute_query_error_token_stream = {
            let error_path_stringified = format!("{try_camel_case_stringified}{delete_name_camel_case_stringified}");
            error_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        //
        let parameters_token_stream = {
            quote::quote!{
                #parameters_derive_token_stream
                pub struct #delete_parameters_camel_case_token_stream {
                    pub #query_lower_case_token_stream: #delete_query_camel_case_token_stream,
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
                    let field_type = &field.ty;
                    Some(quote::quote!{
                        pub #field_ident: Option<#field_type>
                    })
                },
            });
            quote::quote!{
                #query_derive_token_stream
                pub struct #delete_query_camel_case_token_stream {
                    #(#fields_with_excluded_id_token_stream),*
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
                        pub #field_ident: Option<std::string::String>
                    })
                },
            });
            quote::quote!{
                #[derive(Debug, serde::Serialize, serde::Deserialize)]
                struct #delete_query_for_url_encoding_camel_case_token_stream {
                    #(#fields_for_url_encoding_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{query_for_url_encoding_token_stream}");
        let into_url_encoding_version_token_stream = {
            let fields_into_url_encoding_version_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    Some(quote::quote!{
                        let #field_ident = self.#field_ident.map(|value| {
                            #crate_common_serde_urlencoded_serde_urlencoded_parameter_serde_urlencoded_parameter_token_stream(
                                value,
                            )
                        });
                    })
                },
            });
            let fields_into_url_encoding_version_constract_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
                true => None,
                false => {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    Some(quote::quote!{
                        #field_ident
                    })
                },
            });
            quote::quote!{
                impl #delete_query_camel_case_token_stream {
                    fn #into_url_encoding_version_name_token_stream(self) -> #delete_query_for_url_encoding_camel_case_token_stream {
                        #(#fields_into_url_encoding_version_with_excluded_id_token_stream)*
                        #delete_query_for_url_encoding_camel_case_token_stream {
                            #(#fields_into_url_encoding_version_constract_with_excluded_id_token_stream),*
                        }
                    }
                }
            }
        };
        // println!("{into_url_encoding_version_token_stream}");
        let prepare_and_execute_query_token_stream = {
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &try_delete_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
            );
            let prepare_and_execute_query_response_variants_token_stream = &try_delete_response_variants_token_stream;
            let query_part = crate::check_for_none::QueryPart::QueryParameters;
            let check_for_none_token_stream = crate::check_for_none::check_for_none(
                &fields_named,
                &id_field,
                &proc_macro_name_ident_stringified,
                dot_space,
                &try_delete_response_variants_token_stream,
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
                    &self.#query_lower_case_token_stream.#field_ident
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
            let expected_updated_primary_keys_name_token_stream = quote::quote!{expected_updated_primary_keys};
            let expected_updated_primary_keys_token_stream = {
                quote::quote!{
                    #id_field_ident
                        .iter()
                        .map(|element| element.to_inner().clone()) //todo - maybe its not a good idea to remove .clone here coz in macro dont know what type
                        .collect::<Vec<#id_field_type>>()
                }
            };
            let query_string_primary_key_some_other_none_token_stream = {
                let handle_token_stream = {
                    let handle_stringified = format!("\"{{}} {{}} {{}} {{}} {id_field_ident} {{}} ({{}}) {returning_id_stringified}\"");
                    handle_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                quote::quote!{
                    format!(
                        #handle_token_stream,
                        crate::server::postgres::constants::DELETE_NAME,
                        crate::server::postgres::constants::FROM_NAME,
                        ROUTE_NAME,
                        crate::server::postgres::constants::WHERE_NAME,
                        crate::server::postgres::constants::IN_NAME,
                        {
                            #increment_initialization_token_stream
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
                                        return #try_delete_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                    }
                                }
                            }
                            additional_parameters.pop();
                            additional_parameters
                        }
                    )
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
                        Some(quote::quote!{
                            if let Some(value) = &self.#query_lower_case_token_stream.#field_ident {
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
                                        return #try_delete_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                    },
                                }
                            }
                        })
                    },
                });
                quote::quote!{
                    format!(
                        "{} {} {} {} {}",
                        #crate_server_postgres_constants_delete_name_token_stream,
                        #crate_server_postgres_constants_from_name_token_stream,
                        ROUTE_NAME,
                        #crate_server_postgres_constants_where_name_token_stream,
                        {
                            #increment_initialization_token_stream
                            let mut additional_parameters = std::string::String::default();
                            #(#additional_parameters_modification_token_stream)*
                            additional_parameters
                        }
                    )
                }
            };
            let binded_query_primary_key_some_other_none_token_stream = {
                quote::quote!{
                    let #binded_query_name_token_stream = {
                        let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                        for element in #id_field_ident {
                            query = query.bind(element.clone().into_inner());//todo think about - is clone needed?
                        }
                        query
                    };
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
                        Some(quote::quote!{
                            if let Some(value) = self.#query_lower_case_token_stream.#field_ident {
                                query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(value, query);
                            }
                        })
                    },
                });
                quote::quote!{
                    let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                    #(#binded_query_modifications_token_stream)*
                    query
                }
            };
            quote::quote!{
                // impl #delete_parameters_camel_case_token_stream {
                //     pub async fn #prepare_and_execute_query_name_token_stream(
                //         self,
                //         #app_info_state_name_token_stream: &#app_info_state_path,
                //     ) -> #try_delete_response_variants_token_stream
                //     {
                //         #check_for_none_token_stream
                //         let #query_string_name_token_stream = #query_string_token_stream;
                //         // println!("{query_string}");
                //         let #binded_query_name_token_stream = {
                //             #binded_query_token_stream
                //         };
                //         #acquire_pool_and_connection_token_stream
                //         match #binded_query_name_token_stream
                //             .execute(#pg_connection_token_stream.as_mut())
                //             .await
                //         {
                //             Ok(_) => {
                //                 //todo - is need to return rows affected?
                //                 #try_delete_response_variants_token_stream::#desirable_token_stream(())
                //             }
                //             Err(e) => {
                //                 #from_log_and_return_error_token_stream
                //             }
                //         }
                //     }
                // }
                impl #delete_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        #app_info_state_name_token_stream: &#app_info_state_path,
                    ) -> #try_delete_response_variants_token_stream {
                        #check_for_none_token_stream
                        match (#(#parameters_match_token_stream),*) {
                            (#(#parameters_match_primary_key_some_other_none_token_stream),*) => {
                                let #expected_updated_primary_keys_name_token_stream = {
                                    #expected_updated_primary_keys_token_stream
                                };
                                let #query_string_name_token_stream = {
                                    #query_string_primary_key_some_other_none_token_stream
                                };
                                println!("{}", #query_string_name_token_stream);
                                #binded_query_primary_key_some_other_none_token_stream
                                #acquire_pool_and_connection_token_stream
                                let mut #postgres_transaction_token_stream = match {
                                    use sqlx::Acquire;
                                    #pg_connection_token_stream.#begin_token_stream()
                                }
                                .await
                                {
                                    Ok(value) => value,
                                    Err(e) => {
                                        #from_log_and_return_error_token_stream
                                    }
                                };
                                match #binded_query_name_token_stream.fetch_all(#postgres_transaction_token_stream.as_mut()).await {//todo fetch instead of fetch_all?
                                    Ok(updated_rows) => {
                                        let typed_updated_rows = {
                                            let mut typed_updated_rows = Vec::with_capacity(updated_rows.len());
                                            for updated_row in updated_rows {
                                                match #primary_key_try_from_sqlx_row_name_token_stream(&updated_row) {
                                                    Ok(updated_row_primary_key) => {
                                                        typed_updated_rows.push(updated_row_primary_key);
                                                    }
                                                    Err(e) => match #postgres_transaction_token_stream.#rollback_token_stream().await {
                                                        Ok(_) => {
                                                            #from_log_and_return_error_token_stream
                                                        }
                                                        Err(rollback_error) => {
                                                            //todo  BIG QUESTION - WHAT TO DO IF ROLLBACK FAILED? INFINITE LOOP TRYING TO ROLLBACK?
                                                            let error = #prepare_and_execute_query_error_token_stream::PrimaryKeyFromRowAndFailedRollback {
                                                                primary_key_from_row: e,
                                                                rollback_error,
                                                                #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                                            };
                                                            #error_log_call_token_stream
                                                            return #try_delete_response_variants_token_stream::from(error);
                                                        }
                                                    },
                                                }
                                            }
                                            typed_updated_rows
                                        };
                                        {
                                            let non_existing_primary_keys = {
                                                let mut non_existing_primary_keys =
                                                    Vec::with_capacity(#expected_updated_primary_keys_name_token_stream.len());
                                                for element in #expected_updated_primary_keys_name_token_stream {
                                                    if let false =
                                                        typed_updated_rows.contains(&element)
                                                    {
                                                        non_existing_primary_keys
                                                            .push(element);
                                                    }
                                                }
                                                non_existing_primary_keys
                                            };
                                            if let false = non_existing_primary_keys.is_empty() {
                                                match #postgres_transaction_token_stream.#rollback_token_stream().await {
                                                    Ok(_) => {
                                                        let error = #prepare_and_execute_query_error_token_stream::NonExistingPrimaryKeys {
                                                            non_existing_primary_keys,
                                                            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream, //todo how to show log from proc_macro
                                                        };
                                                        #error_log_call_token_stream
                                                        return #try_delete_response_variants_token_stream::from(error);
                                                    }
                                                    Err(e) => {
                                                        let error = #prepare_and_execute_query_error_token_stream::NonExistingPrimaryKeysAndFailedRollback {
                                                                non_existing_primary_keys,
                                                                rollback_error: e,
                                                                #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream, //todo how to show log from proc_macro
                                                            };
                                                        #error_log_call_token_stream
                                                        return #try_delete_response_variants_token_stream::from(error);
                                                    }
                                                }
                                            }
                                        }
                                        match #postgres_transaction_token_stream.#commit_token_stream().await {
                                            Ok(_) => #try_delete_response_variants_token_stream::#desirable_token_stream(()),
                                            Err(e) => {
                                                //todo  BIG QUESTION - WHAT TO DO IF COMMIT FAILED? INFINITE LOOP TRYING TO COMMIT?
                                                let error = #prepare_and_execute_query_error_token_stream::CommitFailed {
                                                    commit_error: e,
                                                    #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                                };
                                                #error_log_call_token_stream
                                                #try_delete_response_variants_token_stream::from(error) //todo - few variants or return ResponseVariants::from - with return ; and not
                                            }
                                        }
                                    }
                                    Err(e) => match #postgres_transaction_token_stream.#rollback_token_stream().await {
                                        Ok(_) => {
                                            let error = #prepare_and_execute_query_error_token_stream::from(e);
                                            #error_log_call_token_stream
                                            #try_delete_response_variants_token_stream::from(error)
                                        }
                                        Err(rollback_error) => {
                                            //todo  BIG QUESTION - WHAT TO DO IF ROLLBACK FAILED? INFINITE LOOP TRYING TO ROLLBACK?
                                            let error = #prepare_and_execute_query_error_token_stream::DeleteAndRollbackFailed {
                                                delete_error: e,
                                                rollback_error,
                                                #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                            };
                                            #error_log_call_token_stream
                                            #try_delete_response_variants_token_stream::from(error)
                                        }
                                    },
                                }
                            }
                            _ => {
                                let #query_string_name_token_stream = format!(
                                    "{} {} {} {} {}",
                                    crate::server::postgres::constants::DELETE_NAME,
                                    crate::server::postgres::constants::FROM_NAME,
                                    ROUTE_NAME,
                                    crate::server::postgres::constants::WHERE_NAME,
                                    {
                                        format!(
                                            "{}",
                                            {
                                                #increment_initialization_token_stream
                                                let mut additional_parameters = std::string::String::default();
                                                if let Some(value) = &self.#query_lower_case_token_stream.name {
                                                    match #crate_server_postgres_bind_query_bind_query_try_increment_token_stream(
                                                        value,
                                                        &mut increment,
                                                    ) {
                                                        Ok(_) => {
                                                            let handle = format!("name = ${increment}");
                                                            match additional_parameters.is_empty() {
                                                                true => {
                                                                    additional_parameters.push_str(&handle);
                                                                }
                                                                false => {
                                                                    additional_parameters.push_str(&format!(
                                                                        " {} {handle}",
                                                                        crate::server::postgres::constants::AND_NAME,
                                                                    ));
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            return #try_delete_response_variants_token_stream::BindQuery {
                                                                checked_add: e.into_serialize_deserialize_version(),
                                                                #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                                            };
                                                        }
                                                    }
                                                }
                                                if let Some(value) = &self.query.color {
                                                    match crate::server::postgres::bind_query::BindQuery::try_increment(
                                                        value,
                                                        &mut increment,
                                                    ) {
                                                        Ok(_) => {
                                                            let handle = format!("color = ${increment}");
                                                            match additional_parameters.is_empty() {
                                                                true => {
                                                                    additional_parameters.push_str(&handle);
                                                                }
                                                                false => {
                                                                    additional_parameters.push_str(&format!(
                                                                        " {} {handle}",
                                                                        crate::server::postgres::constants::AND_NAME,
                                                                    ));
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            return #try_delete_response_variants_token_stream::BindQuery {
                                                                checked_add: e.into_serialize_deserialize_version(),
                                                                #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                                            };
                                                        }
                                                    }
                                                }
                                                if let Some(id_vec) = &self.query.id {
                                                    if let false = additional_parameters.is_empty() {
                                                        additional_parameters.push_str(&format!(
                                                            " {}",
                                                            crate::server::postgres::constants::AND_NAME,
                                                        ));
                                                    }
                                                    additional_parameters.push_str(&format!(
                                                        " id {} ({})",
                                                        crate::server::postgres::constants::IN_NAME,
                                                        {
                                                            let mut additional_parameters = std::string::String::default();
                                                            for element in id_vec {
                                                                match crate::server::postgres::bind_query::BindQuery::try_increment(
                                                                    element,
                                                                    &mut increment,
                                                                ) {
                                                                    Ok(_) => {
                                                                        additional_parameters.push_str(&format!("${increment},"));
                                                                    }
                                                                    Err(e) => {
                                                                        return #try_delete_response_variants_token_stream::BindQuery {
                                                                            checked_add: e.into_serialize_deserialize_version(),
                                                                            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                                                        };
                                                                    }
                                                                }
                                                            }
                                                            additional_parameters.pop();
                                                            additional_parameters
                                                        }
                                                    ));
                                                }
                                                additional_parameters
                                            },
                                        )
                                    }
                                );
                                println!("{}", #query_string_name_token_stream);
                                let #binded_query_name_token_stream = {
                                    let mut query = sqlx::query::<sqlx::Postgres>(&#query_string_name_token_stream);
                                    if let Some(value) = self.query.name {
                                        query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                                            value, query,
                                        );
                                    }
                                    if let Some(value) = self.query.color {
                                        query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                                            value, query,
                                        );
                                    }
                                    if let Some(id_vec) = self.query.id {
                                        for element in id_vec {
                                            query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(element, query);
                                        }
                                    }
                                    query
                                };
                                #acquire_pool_and_connection_token_stream
                                match #binded_query_name_token_stream.execute(#pg_connection_token_stream.as_mut()).await {
                                    Ok(_) => #try_delete_response_variants_token_stream::#desirable_token_stream(()),
                                    Err(e) => {
                                        #from_log_and_return_error_token_stream
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
        // println!("{prepare_and_execute_query_token_stream}");
        let try_delete_error_named_token_stream = {
            let try_delete_request_error_camel_case_token_stream = {
                let try_delete_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{delete_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_delete_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_delete_error_named_camel_case_token_stream {
                    #query_encode_variant_token_stream,
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_delete_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{try_delete_error_named_token_stream}");
        let http_request_token_stream = {
            let try_delete_lower_case_token_stream = {
                let try_delete_lower_case_stringified = format!("{try_lower_case_stringified}_{delete_name_lower_case_stringified}");
                try_delete_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_delete_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{delete_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_delete_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #delete_parameters_camel_case_token_stream,
                ) -> Result<(), #try_delete_error_named_camel_case_token_stream> {
                    let encoded_query = match serde_urlencoded::to_string(#delete_query_for_url_encoding_camel_case_token_stream::from(#parameters_lower_case_token_stream.query)) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_delete_error_named_camel_case_token_stream::#query_encode_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        "{}/api/{}?{}",
                        #server_location_name_token_stream,
                        ROUTE_NAME,
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
                        Err(e) => Err(#try_delete_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
                //
                // pub async fn try_delete<'a>(
                //     server_location: &str,
                //     parameters: DeleteParameters,
                // ) -> Result<(), TryDeleteErrorNamed> {
                //     let encoded_query = match serde_urlencoded::to_string(DeleteQueryForUrlEncoding::from(parameters.query)) {
                //         Ok(value) => value,
                //         Err(e) => {
                //             return Err(TryDeleteErrorNamed::QueryEncode {
                //                 url_encoding: e,
                //                 code_occurence: crate::code_occurence_tufa_common!(),
                //             });
                //         }
                //     };
                //     let url = format!("{}/api/{}?{}", server_location, ROUTE_NAME, encoded_query);
                //     match tvfrr_extraction_logic_try_delete(
                //         reqwest::Client::new()
                //         .delete(&url)
                //         .header(
                //             crate::common::git::project_git_info::PROJECT_COMMIT,
                //             crate::global_variables::compile_time::project_git_info::PROJECT_GIT_INFO.project_commit,
                //         )
                //         .send(),
                //     )
                //     .await
                //     {
                //         Ok(value) => Ok(value),
                //         Err(e) => Err(TryDeleteErrorNamed::RequestError {
                //             request_error: e,
                //             code_occurence: crate::code_occurence_tufa_common!(),
                //         }),
                //     }
                // }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let delete_lower_case_token_stream = delete_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let delete_query_try_from_url_encoding_camel_case_token_stream = {
                let delete_query_try_from_url_encoding_camel_case_stringified = format!("{delete_name_camel_case_stringified}{query_camel_case_stringified}{try_camel_case_stringified}From{url_encoding_camel_case_stringified}");
                delete_query_try_from_url_encoding_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {delete_query_try_from_url_encoding_camel_case_stringified} {}",     proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #delete_lower_case_token_stream<'a>(
                    #query_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_query_token_stream<#delete_query_for_url_encoding_camel_case_token_stream>,
                        #axum_extract_rejection_query_rejection_token_stream,
                    >,
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #delete_parameters_camel_case_token_stream {
                        #query_lower_case_token_stream: match #crate_server_routes_helpers_query_extractor_error_query_value_result_extractor_token_stream::<
                            #delete_query_for_url_encoding_camel_case_token_stream,
                            #try_delete_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#query_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => match #delete_query_camel_case_token_stream::try_from(value) {
                                Ok(value) => value,
                                Err(e) => {
                                    let error = #prepare_and_execute_query_error_token_stream::#delete_query_try_from_url_encoding_camel_case_token_stream {
                                        checked_add: e,
                                        #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                    };
                                    #error_log_call_token_stream
                                    return #try_delete_response_variants_token_stream::from(error);
                                }
                            },
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    #parameters_lower_case_token_stream.#prepare_and_execute_query_name_token_stream(&#app_info_state_name_token_stream).await
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            // #query_token_stream
            // #query_for_url_encoding_token_stream
            // #into_url_encoding_version_token_stream
            #prepare_and_execute_query_token_stream
            #try_delete_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{delete_token_stream}");
    let read_by_id_token_stream = {
        let read_by_id_name_camel_case_stringified = "ReadById";
        let read_by_id_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&read_by_id_name_camel_case_stringified.to_string());
        let read_by_id_parameters_camel_case_token_stream = {
            let read_by_id_parameters_camel_case_stringified = format!("{read_by_id_name_camel_case_stringified}{parameters_camel_case_stringified}");
            read_by_id_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_by_id_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let read_by_id_path_camel_case_token_stream = {
            let read_by_id_path_camel_case_stringified = format!("{read_by_id_name_camel_case_stringified}{path_camel_case_stringified}");
            read_by_id_path_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_by_id_path_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE)) 
        };
        let read_by_id_query_camel_case_token_stream = {
            let read_by_id_query_camel_case_stringified = format!("{read_by_id_name_camel_case_stringified}{query_camel_case_stringified}");
            read_by_id_query_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_by_id_query_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let read_by_id_query_for_url_encoding_camel_case_token_stream = {
            let read_by_id_query_for_url_encoding_camel_case_stringified = format!("{read_by_id_name_camel_case_stringified}{query_camel_case_stringified}{for_url_encoding_camel_case_stringified}");
            read_by_id_query_for_url_encoding_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_by_id_query_for_url_encoding_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_read_by_id_error_named_camel_case_token_stream = {
            let try_read_by_id_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{read_by_id_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_read_by_id_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_by_id_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_read_by_id_response_variants_token_stream = {
            let try_read_by_id_response_variants_stringified = format!("{try_camel_case_stringified}{read_by_id_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_read_by_id_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_by_id_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #parameters_derive_token_stream
                pub struct #read_by_id_parameters_camel_case_token_stream {
                    pub #path_lower_case_token_stream: #read_by_id_path_camel_case_token_stream,
                    pub #query_lower_case_token_stream: #read_by_id_query_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let path_token_stream = {
            quote::quote!{
                #path_derive_token_stream
                pub struct #read_by_id_path_camel_case_token_stream {
                    pub #id_field_ident: crate::server::postgres::bigserial::Bigserial//#id_field_type,
                }
            }
        };
        // println!("{path_token_stream}");
        let query_token_stream = {
            quote::quote!{
                #query_derive_token_stream
                pub struct #read_by_id_query_camel_case_token_stream {
                    pub #select_token_stream: Option<#column_select_ident_token_stream>,
                }
            }
        };
        // println!("{query_token_stream}");
        let query_for_url_encoding_token_stream = {
            quote::quote!{
                #[derive(Debug, serde::Serialize, serde::Deserialize)]
                struct #read_by_id_query_for_url_encoding_camel_case_token_stream {
                    #select_token_stream: Option<std::string::String>,
                } 
            }
        };
        // println!("{query_for_url_encoding_token_stream}");
        let into_url_encoding_version_token_stream = {
            quote::quote!{
                impl #read_by_id_query_camel_case_token_stream {
                    fn #into_url_encoding_version_name_token_stream(self) -> #read_by_id_query_for_url_encoding_camel_case_token_stream {
                        let #select_token_stream = self.#select_token_stream.map(|value| {
                            #crate_common_serde_urlencoded_serde_urlencoded_parameter_serde_urlencoded_parameter_token_stream(
                                value,
                            )
                        });
                        #read_by_id_query_for_url_encoding_camel_case_token_stream {
                            #select_token_stream
                        }
                    }
                }
            }
        };
        // println!("{into_url_encoding_version_token_stream}");
        let prepare_and_execute_query_token_stream = {
            let prepare_and_execute_query_error_token_stream = {
                let error_path_stringified = format!("{try_camel_case_stringified}{read_by_id_name_camel_case_stringified}");
                error_path_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &try_read_by_id_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
            );
            let query_string_token_stream = {
                let query_token_stream = {
                    let query_stringified = format!("\"{{}} {{}} {{}} {{}} {{}} {id_field_ident} = $1\"");
                    query_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                quote::quote!{
                    format!(
                        #query_token_stream,
                        #crate_server_postgres_constants_select_name_token_stream,
                        crate::server::postgres::generate_query::GenerateQuery::generate_query(&#select_token_stream),
                        #crate_server_postgres_constants_from_name_token_stream,
                        ROUTE_NAME,
                        #crate_server_postgres_constants_where_name_token_stream,
                    )
                }
            };
            let binded_query_token_stream = {
                let binded_query_modifications_token_stream = quote::quote!{
                    query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                        self.#path_lower_case_token_stream.#id_field_ident, query,
                    );
                };
                quote::quote!{
                    let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                    #binded_query_modifications_token_stream
                    query
                }
            };
            quote::quote!{
                impl #read_by_id_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        #app_info_state_name_token_stream: &#app_info_state_path,
                    ) -> #try_read_by_id_response_variants_token_stream
                    {
                        let #select_token_stream = self.#query_lower_case_token_stream.#select_token_stream.unwrap_or_default();
                        let #query_string_name_token_stream = #query_string_token_stream;
                        // println!("{query_string}");
                        let #binded_query_name_token_stream = {
                            #binded_query_token_stream
                        };
                        #acquire_pool_and_connection_token_stream
                        match #binded_query_name_token_stream.fetch_one(#pg_connection_token_stream.as_mut()).await {
                            Ok(row) => match #select_token_stream.#options_try_from_sqlx_row_name_token_stream(&row) {
                                Ok(value) => #try_read_by_id_response_variants_token_stream::#desirable_token_stream(value),
                                Err(e) => {
                                    #from_log_and_return_error_token_stream
                                },
                            },
                            Err(e) => {
                                #from_log_and_return_error_token_stream
                            },
                        }
                    }
                }
            }
        };
        // println!("{prepare_and_execute_query_token_stream}");
        let try_read_by_id_error_named_token_stream = {
            let try_read_by_id_request_error_camel_case_token_stream = {
                let try_read_by_id_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{read_by_id_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_read_by_id_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_by_id_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_read_by_id_error_named_camel_case_token_stream {
                    #query_encode_variant_token_stream,
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_read_by_id_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{try_read_by_id_error_named_token_stream}");
        let http_request_token_stream = {
            let try_read_by_id_lower_case_token_stream = {
                let try_read_by_id_lower_case_stringified = format!("{try_lower_case_stringified}_{read_by_id_name_lower_case_stringified}");
                try_read_by_id_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_by_id_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{read_by_id_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_read_by_id_lower_case_token_stream(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #read_by_id_parameters_camel_case_token_stream,
                ) -> Result<
                    #struct_options_ident_token_stream,
                    #try_read_by_id_error_named_camel_case_token_stream,
                > {
                    let encoded_query = match serde_urlencoded::to_string(#parameters_lower_case_token_stream.#query_lower_case_token_stream.#into_url_encoding_version_name_token_stream()) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_read_by_id_error_named_camel_case_token_stream::#query_encode_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        "{}/api/{}/{}?{}",
                        #server_location_name_token_stream,
                        ROUTE_NAME,
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
                        Err(e) => Err(#try_read_by_id_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let read_by_id_lower_case_token_stream = read_by_id_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_by_id_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            quote::quote!{
                pub async fn #read_by_id_lower_case_token_stream(
                    #path_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_path_token_stream<#read_by_id_path_camel_case_token_stream>,
                        #axum_extract_rejection_path_rejection_token_stream,
                    >,
                    #query_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_query_token_stream<#read_by_id_query_camel_case_token_stream>,
                        #axum_extract_rejection_query_rejection_token_stream,
                    >,
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #read_by_id_parameters_camel_case_token_stream {
                        #path_lower_case_token_stream: match #crate_server_routes_helpers_path_extractor_error_path_value_result_extractor_token_stream::<
                            #read_by_id_path_camel_case_token_stream,
                            #try_read_by_id_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#path_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                        #query_lower_case_token_stream: match #crate_server_routes_helpers_query_extractor_error_query_value_result_extractor_token_stream::<
                            #read_by_id_query_camel_case_token_stream,
                            #try_read_by_id_response_variants_token_stream,
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
                    #parameters_lower_case_token_stream.#prepare_and_execute_query_name_token_stream(&#app_info_state_name_token_stream).await
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #path_token_stream
            #query_token_stream
            #query_for_url_encoding_token_stream
            #into_url_encoding_version_token_stream
            #prepare_and_execute_query_token_stream
            #try_read_by_id_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{read_by_id_token_stream}");
    let read_with_body_token_stream = {
        let read_with_body_name_camel_case_stringified = "ReadWithBody";
        let read_with_body_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&read_with_body_name_camel_case_stringified.to_string());
        let read_with_body_parameters_camel_case_token_stream = {
            let read_with_body_parameters_camel_case_stringified = format!("{read_with_body_name_camel_case_stringified}{parameters_camel_case_stringified}");
            read_with_body_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_with_body_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let read_with_body_payload_camel_case_token_stream = {
            let read_with_body_payload_camel_case_stringified = format!("{read_with_body_name_camel_case_stringified}{payload_camel_case_stringified}");
            read_with_body_payload_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_with_body_payload_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_read_with_body_error_named_camel_case_token_stream = {
            let try_read_with_body_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{read_with_body_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_read_with_body_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_with_body_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_read_with_body_response_variants_token_stream = {
            let try_read_with_body_response_variants_stringified = format!("{try_camel_case_stringified}{read_with_body_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_read_with_body_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_with_body_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
                pub struct #read_with_body_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #read_with_body_payload_camel_case_token_stream,
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
                #payload_derive_token_stream
                pub struct #read_with_body_payload_camel_case_token_stream {
                    pub #select_token_stream: #column_select_ident_token_stream,
                    pub #id_field_ident: Option<Vec<crate::server::postgres::bigserial::Bigserial>>,
                    #(#fields_with_excluded_id_token_stream)*
                    pub #order_by_token_stream: crate::server::postgres::order_by::OrderBy<#column_ident_token_stream>,
                    pub limit: crate::server::postgres::postgres_bigint::PostgresBigint,
                    pub offset: crate::server::postgres::postgres_bigint::PostgresBigint,
                }
            }
        };
        // println!("{payload_token_stream}");
        let prepare_and_execute_query_token_stream = {
            let prepare_and_execute_query_error_token_stream = {
                let error_path_stringified = format!("{try_camel_case_stringified}{read_with_body_name_camel_case_stringified}");
                error_path_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &try_read_with_body_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
            );
            let query_string_token_stream = {
                let additional_parameters_id_modification_token_stream = {
                    let query_part_token_stream = {
                        let query_part_stringified = format!("\"{{prefix}} {id_field_ident} = {{}}({{}}[{{}}])\"");
                        query_part_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_part_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    quote::quote!{
                        if let Some(value) = &self.#payload_lower_case_token_stream.#id_field_ident {
                            let prefix = match additional_parameters.is_empty() {
                                true => #crate_server_postgres_constants_where_name_token_stream.to_string(),
                                false => format!(" {}", #crate_server_postgres_constants_and_name_token_stream),
                            };
                            let bind_increments = {
                                let mut bind_increments = std::string::String::default();
                                for element in value {
                                    match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(
                                        element,
                                        &mut increment
                                    ) {
                                        Ok(value) => {
                                            bind_increments.push_str(&format!("{value}, "));
                                        },
                                        Err(e) => {
                                            return #try_read_with_body_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                        },
                                    }
                                }
                                if let false = bind_increments.is_empty() {
                                    bind_increments.pop();
                                    bind_increments.pop();
                                }
                                bind_increments
                            };
                            additional_parameters.push_str(&format!(
                                #query_part_token_stream,
                                #crate_server_postgres_constants_any_name_token_stream,
                                #crate_server_postgres_constants_array_name_token_stream,
                                bind_increments
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
                        Some(quote::quote!{
                            if let Some(value) = &self.#payload_lower_case_token_stream.#field_ident {
                                let prefix = match additional_parameters.is_empty() {
                                    true => #crate_server_postgres_constants_where_name_token_stream.to_string(),
                                    false => format!(" {}", #crate_server_postgres_constants_and_name_token_stream),
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
                                                return #try_read_with_body_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
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
                quote::quote!{
                    format!(
                        "{} {} {} {} {}",
                        #crate_server_postgres_constants_select_name_token_stream,
                        crate::server::postgres::generate_query::GenerateQuery::generate_query(
                            &self.#payload_lower_case_token_stream.#select_token_stream
                        ),
                        #crate_server_postgres_constants_from_name_token_stream,
                        ROUTE_NAME,
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
                                let value = &self.#payload_lower_case_token_stream.#order_by_token_stream;
                                let order_stringified = match &value.order {
                                    Some(order) => order.to_string(),
                                    None => crate::server::postgres::order::Order::default().to_string(),
                                };
                                additional_parameters.push_str(&format!(
                                    "{prefix}{} {} {order_stringified}",
                                    #crate_server_postgres_constants_order_by_name_token_stream,
                                    value.column
                                ));
                            }
                            {
                                let prefix = match additional_parameters.is_empty() {
                                    true => "",
                                    false => " ",
                                };
                                let value = match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(
                                    &self.#payload_lower_case_token_stream.limit,
                                    &mut increment
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return #try_read_with_body_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                    },
                                };
                                additional_parameters.push_str(&format!(
                                    "{prefix}{} {value}",
                                    #crate_server_postgres_constants_limit_name_token_stream,
                                ));
                            }
                            {
                                let prefix = match additional_parameters.is_empty() {
                                    true => "",
                                    false => " ",
                                };
                                let value = match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(
                                    &self.#payload_lower_case_token_stream.offset,
                                    &mut increment
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return #try_read_with_body_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                    },
                                };
                                additional_parameters.push_str(&format!(
                                    "{prefix}{} {value}",
                                    #crate_server_postgres_constants_offset_name_token_stream,
                                ));
                            }
                            additional_parameters
                        }
                    )
                }
            };
            let binded_query_token_stream = {
                let binded_query_modifications_token_stream = fields_named.iter().map(|field|{
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    quote::quote!{
                        if let Some(values) = self.#payload_lower_case_token_stream.#field_ident {
                            for value in values {
                                query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                                    value, query,
                                );
                            }
                        }
                    }
                });
                quote::quote!{
                    let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                    #(#binded_query_modifications_token_stream)*
                    query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                        self.#payload_lower_case_token_stream.limit,
                        query,
                    );
                    query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                        self.#payload_lower_case_token_stream.offset,
                        query,
                    );
                    query
                }
            };
            quote::quote!{
                impl #read_with_body_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        #app_info_state_name_token_stream: &#app_info_state_path,
                    ) -> #try_read_with_body_response_variants_token_stream
                    {
                        let #query_string_name_token_stream = #query_string_token_stream;
                        // println!("{query_string}");
                        let #binded_query_name_token_stream = {
                            #binded_query_token_stream
                        };
                        let vec_values = {
                            #acquire_pool_and_connection_token_stream
                            let mut rows = #binded_query_name_token_stream.fetch(#pg_connection_token_stream.as_mut());
                            let mut vec_values = Vec::new();
                            while let Some(row) = {
                                match {
                                    use futures::TryStreamExt;
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
                                match self.#payload_lower_case_token_stream.#select_token_stream.#options_try_from_sqlx_row_name_token_stream(&row) {
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
                        #try_read_with_body_response_variants_token_stream::#desirable_token_stream(vec_values)
                    }
                }
            }
        };
        // println!("{prepare_and_execute_query_token_stream}");
        let try_read_with_body_error_named_token_stream = {
            let try_read_with_body_request_error_camel_case_token_stream = {
                let try_read_with_body_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{read_with_body_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_read_with_body_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_with_body_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_read_with_body_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_read_with_body_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_read_with_body_error_named_token_stream}");
        let http_request_token_stream = {
            let try_read_with_body_lower_case_token_stream = {
                let try_read_with_body_lower_case_stringified = format!("{try_lower_case_stringified}_{read_with_body_name_lower_case_stringified}");
                try_read_with_body_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_with_body_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{read_with_body_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_read_with_body_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #read_with_body_parameters_camel_case_token_stream,
                ) -> Result<
                    Vec<#struct_options_ident_token_stream>,
                    #try_read_with_body_error_named_camel_case_token_stream,
                > {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_read_with_body_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        "{}/api/{}/search",
                        #server_location_name_token_stream,
                        ROUTE_NAME
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
                        Err(e) => Err(#try_read_with_body_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let read_with_body_lower_case_token_stream = read_with_body_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_with_body_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            quote::quote!{
                pub async fn #read_with_body_lower_case_token_stream(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#read_with_body_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #read_with_body_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #read_with_body_payload_camel_case_token_stream,
                            #try_read_with_body_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    #parameters_lower_case_token_stream.#prepare_and_execute_query_name_token_stream(&#app_info_state_name_token_stream).await
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #prepare_and_execute_query_token_stream
            #try_read_with_body_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{read_with_body_token_stream}");
    let read_token_stream = {
        let read_name_camel_case_stringified = "Read";
        let read_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&read_name_camel_case_stringified.to_string());
        let read_parameters_camel_case_token_stream = {
            let read_parameters_camel_case_stringified = format!("{read_name_camel_case_stringified}{parameters_camel_case_stringified}");
            read_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let read_query_camel_case_token_stream = {
            let read_query_camel_case_stringified = format!("{read_name_camel_case_stringified}{query_camel_case_stringified}");
            read_query_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_query_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE)) 
        };
        let read_query_for_url_encoding_camel_case_token_stream = {
            let read_query_for_url_encoding_camel_case_stringified = format!("{read_name_camel_case_stringified}{query_camel_case_stringified}{for_url_encoding_camel_case_stringified}");
            read_query_for_url_encoding_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_query_for_url_encoding_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE)) 
        };
        let try_read_error_named_camel_case_token_stream = {
            let try_read_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{read_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_read_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_read_response_variants_token_stream = {
            let try_read_response_variants_stringified = format!("{try_camel_case_stringified}{read_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_read_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #parameters_derive_token_stream
                pub struct #read_parameters_camel_case_token_stream {
                    pub #query_lower_case_token_stream: #read_query_camel_case_token_stream,
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
                #query_derive_token_stream
                pub struct #read_query_camel_case_token_stream {
                    pub #select_token_stream: Option<#column_select_ident_token_stream>,
                    pub #id_field_ident: Option<crate::server::postgres::bigserial_ids::BigserialIds>,
                    #(#fields_with_excluded_id_token_stream)*
                    pub #order_by_token_stream: Option<CatOrderByWrapper>,//todo
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
                #[derive(Debug, serde::Serialize, serde::Deserialize)]
                struct #read_query_for_url_encoding_camel_case_token_stream {
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
                impl #read_query_camel_case_token_stream {
                    fn #into_url_encoding_version_name_token_stream(self) -> #read_query_for_url_encoding_camel_case_token_stream {
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
                        #read_query_for_url_encoding_camel_case_token_stream {
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
        let prepare_and_execute_query_token_stream = {
            let prepare_and_execute_query_error_token_stream = {
                let error_path_stringified = format!("{try_camel_case_stringified}{read_name_camel_case_stringified}");
                error_path_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &try_read_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
            );
            let query_string_token_stream = {
                let additional_parameters_modification_token_stream = fields_named.iter().map(|field| {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    let handle_token_stream = {
                        let handle_stringified = format!("\"{{prefix}} {field_ident} = {{}}({{}}[{{value}}])\"");
                        handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    quote::quote!{
                        if let Some(value) = &self.#query_lower_case_token_stream.#field_ident {
                            let prefix = match additional_parameters.is_empty() {
                                true => #crate_server_postgres_constants_where_name_token_stream.to_string(),
                                false => format!(" {}", #crate_server_postgres_constants_and_name_token_stream),
                            };
                            let value = match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(
                                value,
                                &mut increment
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return #try_read_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                },
                            };
                            additional_parameters.push_str(&format!(
                                #handle_token_stream,
                                #crate_server_postgres_constants_any_name_token_stream,
                                #crate_server_postgres_constants_array_name_token_stream,
                            ));
                        }
                    }
                });
                quote::quote!{
                    format!(
                        "{} {} {} {} {}",
                        #crate_server_postgres_constants_select_name_token_stream,
                        crate::server::postgres::generate_query::GenerateQuery::generate_query(&#select_token_stream),
                        #crate_server_postgres_constants_from_name_token_stream,
                        ROUTE_NAME,
                        {
                            #increment_initialization_token_stream
                            let mut additional_parameters = std::string::String::default();
                            #(#additional_parameters_modification_token_stream)*
                            if let Some(value) = &self.#query_lower_case_token_stream.#order_by_token_stream {
                                let prefix = match additional_parameters.is_empty() {
                                    true => "",
                                    false => " ",
                                };
                                let order_stringified = match &value.0.order {
                                    Some(order) => order.to_string(),
                                    None => crate::server::postgres::order::Order::default().to_string(),
                                };
                                additional_parameters.push_str(&format!(
                                    "{prefix}{} {} {order_stringified}",
                                    #crate_server_postgres_constants_order_by_name_token_stream,
                                    value.0.column
                                ));
                            }
                            {
                                let prefix = match additional_parameters.is_empty() {
                                    true => "",
                                    false => " ",
                                };
                                let value = match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(
                                    &self.#query_lower_case_token_stream.limit,
                                    &mut increment
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return #try_read_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                    },
                                };
                                additional_parameters.push_str(&format!(
                                    "{prefix}{} {value}",
                                    #crate_server_postgres_constants_limit_name_token_stream,
                                ));
                            }
                            if let Some(value) = &self.#query_lower_case_token_stream.offset {
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
                                        return #try_read_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                    },
                                };
                                additional_parameters.push_str(&format!(
                                    "{prefix}{} {value}",
                                    #crate_server_postgres_constants_offset_name_token_stream,
                                ));
                            }
                            additional_parameters
                        }
                    )
                }
            };
            let binded_query_token_stream = {
                let binded_query_modifications_token_stream = fields_named.iter().map(|field|{
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    quote::quote!{
                        if let Some(value) = self.#query_lower_case_token_stream.#field_ident {
                            query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                                value, query,
                            );
                        }
                    }
                });
                quote::quote!{
                    let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                    #(#binded_query_modifications_token_stream)*
                    query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                        self.#query_lower_case_token_stream.limit,
                        query,
                    );
                    if let Some(value) = self.#query_lower_case_token_stream.offset {
                        query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                            value, query,
                        );
                    }
                    query
                }
            };
            quote::quote!{
                impl #read_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self, //impl crate::server::routes::helpers::bind_sqlx_query::BindSqlxQuer + crate::server::postgres::generate_query::GenerateQuery
                        #app_info_state_name_token_stream: &#app_info_state_path,
                    ) -> #try_read_response_variants_token_stream
                    {
                        let #select_token_stream = #column_select_ident_token_stream::from(self.#query_lower_case_token_stream.#select_token_stream.clone());
                        let #query_string_name_token_stream = #query_string_token_stream;
                        // println!("{query_string}");
                        let #binded_query_name_token_stream = {
                            #binded_query_token_stream
                        };
                        let vec_values = {
                            #acquire_pool_and_connection_token_stream
                            let mut rows = #binded_query_name_token_stream.fetch(#pg_connection_token_stream.as_mut());
                            let mut vec_values = Vec::new();
                            while let Some(row) = {
                                match {
                                    use futures::TryStreamExt;
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
                        #try_read_response_variants_token_stream::#desirable_token_stream(vec_values)
                    }
                }
            }
        };
        // println!("{prepare_and_execute_query_token_stream}");
        let try_read_error_named_token_stream = {
            let try_read_request_error_camel_case_token_stream = {
                let try_read_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{read_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_read_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_read_error_named_camel_case_token_stream {
                    #query_encode_variant_token_stream,
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_read_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{try_read_error_named_token_stream}");
        let http_request_token_stream = {
            let try_read_lower_case_token_stream = {
                let try_read_lower_case_stringified = format!("{try_lower_case_stringified}_{read_name_lower_case_stringified}");
                try_read_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_read_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{read_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_read_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #read_parameters_camel_case_token_stream,
                ) -> Result<
                    Vec<#struct_options_ident_token_stream>,
                    #try_read_error_named_camel_case_token_stream,
                > {
                    let encoded_query = match serde_urlencoded::to_string(#parameters_lower_case_token_stream.#query_lower_case_token_stream.#into_url_encoding_version_name_token_stream()) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_read_error_named_camel_case_token_stream::#query_encode_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        "{}/api/{}?{}",
                        #server_location_name_token_stream,
                        ROUTE_NAME,
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
                        Err(e) => Err(#try_read_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let read_lower_case_token_stream = read_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {read_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            quote::quote!{
                pub async fn #read_lower_case_token_stream(
                    #query_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_query_token_stream<#read_query_camel_case_token_stream>,
                        #axum_extract_rejection_query_rejection_token_stream,
                    >,
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #read_parameters_camel_case_token_stream {
                        #query_lower_case_token_stream: match #crate_server_routes_helpers_query_extractor_error_query_value_result_extractor_token_stream::<
                            #read_query_camel_case_token_stream,
                            #try_read_response_variants_token_stream,
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
                    #parameters_lower_case_token_stream.#prepare_and_execute_query_name_token_stream(&#app_info_state_name_token_stream).await
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #query_token_stream
            #query_for_url_encoding_token_stream
            #into_url_encoding_version_token_stream
            #prepare_and_execute_query_token_stream
            #try_read_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{read_token_stream}");
    let update_by_id_token_stream = {//todo WHY ITS RETURN SUCCESS EVEN IF ROW DOES NOT EXISTS?
        let update_by_id_name_camel_case_stringified = "UpdateById";
        let update_by_id_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&update_by_id_name_camel_case_stringified.to_string());
        let update_by_id_parameters_camel_case_token_stream = {
            let update_by_id_parameters_camel_case_stringified = format!("{update_by_id_name_camel_case_stringified}{parameters_camel_case_stringified}");
            update_by_id_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_by_id_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_by_id_path_camel_case_token_stream = {
            let update_by_id_path_camel_case_stringified = format!("{update_by_id_name_camel_case_stringified}{path_camel_case_stringified}");
            update_by_id_path_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_by_id_path_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_by_id_payload_camel_case_token_stream = {
            let update_by_id_payload_camel_case_stringified = format!("{update_by_id_name_camel_case_stringified}{payload_camel_case_stringified}");
            update_by_id_payload_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_by_id_payload_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_update_by_id_error_named_camel_case_token_stream = {
            let try_update_by_id_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{update_by_id_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_update_by_id_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_by_id_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_update_by_id_response_variants_token_stream = {
            let try_update_by_id_response_variants_stringified = format!("{try_camel_case_stringified}{update_by_id_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_update_by_id_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_by_id_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #parameters_derive_token_stream
                pub struct #update_by_id_parameters_camel_case_token_stream {
                    pub #path_lower_case_token_stream: #update_by_id_path_camel_case_token_stream,
                    pub #payload_lower_case_token_stream: #update_by_id_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let path_token_stream = {
            quote::quote!{
                #path_derive_token_stream
                pub struct #update_by_id_path_camel_case_token_stream {
                    pub #id_field_ident: crate::server::postgres::bigserial::Bigserial,//#id_field_type
                }
            }
        };
        // println!("{path_token_stream}");
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
                #payload_derive_token_stream
                pub struct #update_by_id_payload_camel_case_token_stream {
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let prepare_and_execute_query_token_stream = {
            let prepare_and_execute_query_error_token_stream = {
                let error_path_stringified = format!("{try_camel_case_stringified}{update_by_id_name_camel_case_stringified}");
                error_path_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let check_for_none_token_stream_excluding_primary_key = crate::check_for_none::check_for_none(
                &fields_named,
                &id_field,
                &proc_macro_name_ident_stringified,
                dot_space,
                &try_update_by_id_response_variants_token_stream,
                crate::check_for_none::QueryPart::Payload,
                true
            );
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &try_update_by_id_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
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
                            let possible_dot_space = match (index + 1) == fields_named_len {
                                true => "",
                                false => dot_space,
                            };
                            let handle_stringified = format!("\"{field_ident} = ${{increment}}{possible_dot_space}\"");
                            handle_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{
                            if let Some(value) = &self.#payload_lower_case_token_stream.#field_ident {
                                match #crate_server_postgres_bind_query_bind_query_try_increment_token_stream(value, &mut increment) {
                                    Ok(_) => {
                                        query.push_str(&format!(#handle_token_stream));//add dot_space for all elements except last
                                    },
                                    Err(e) => {
                                        return #try_update_by_id_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
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
                        match #crate_server_postgres_bind_query_bind_query_try_increment_token_stream(&self.#path_lower_case_token_stream.#id_field_ident, &mut increment) {
                            Ok(_) => {
                                query.push_str(&format!(#query_part_token_stream));
                            },
                            Err(e) => {
                                return #try_update_by_id_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                            },
                        }
                    }
                };
                quote::quote!{
                    #increment_initialization_token_stream
                    let mut query = format!(
                        "{} {} {} ",
                        #crate_server_postgres_constants_update_name_token_stream,
                        ROUTE_NAME,
                        #crate_server_postgres_constants_set_name_token_stream,
                    );
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
                            if let Some(value) = self.#payload_lower_case_token_stream.#field_ident {
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
                        self.#path_lower_case_token_stream.#id_field_ident,
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
            quote::quote!{
                impl #update_by_id_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        #app_info_state_name_token_stream: &#app_info_state_path,
                    ) -> #try_update_by_id_response_variants_token_stream
                    {
                        #check_for_none_token_stream_excluding_primary_key
                        let #query_string_name_token_stream = {
                            #query_string_token_stream
                        };
                        let #binded_query_name_token_stream = {
                            #binded_query_token_stream
                        };
                        #acquire_pool_and_connection_token_stream
                        match #binded_query_name_token_stream
                            .fetch_one(#pg_connection_token_stream.as_mut())
                            .await
                        {
                            Ok(_) => #try_update_by_id_response_variants_token_stream::#desirable_token_stream(()),//todo () type token_stream
                            Err(e) => {
                                #from_log_and_return_error_token_stream;
                            }
                        }
                    }
                }
            }
        };
        // println!("{prepare_and_execute_query_token_stream}");
        let try_update_by_id_error_named_token_stream = {
            let try_update_by_id_request_error_camel_case_token_stream = {
                let try_update_by_id_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{update_by_id_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_update_by_id_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_by_id_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_update_by_id_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_update_by_id_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_update_by_id_error_named_token_stream}");
        let http_request_token_stream = {
            let try_update_by_id_lower_case_token_stream = {
                let try_update_by_id_lower_case_stringified = format!("{try_lower_case_stringified}_{update_by_id_name_lower_case_stringified}");
                try_update_by_id_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_by_id_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{update_by_id_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_update_by_id_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #update_by_id_parameters_camel_case_token_stream,
                ) -> Result<(), #try_update_by_id_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_update_by_id_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        "{}/api/{}/{}",
                        #server_location_name_token_stream,
                        ROUTE_NAME,
                        #parameters_lower_case_token_stream.#path_lower_case_token_stream.id.to_inner()
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
                        Err(e) => Err(#try_update_by_id_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let update_by_id_lower_case_token_stream = update_by_id_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_by_id_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            quote::quote!{
                pub async fn #update_by_id_lower_case_token_stream<'a>(
                    #path_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_path_token_stream<#update_by_id_path_camel_case_token_stream>,
                        #axum_extract_rejection_path_rejection_token_stream,
                    >,
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#update_by_id_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #update_by_id_parameters_camel_case_token_stream {
                        #path_lower_case_token_stream: match #crate_server_routes_helpers_path_extractor_error_path_value_result_extractor_token_stream::<
                            #update_by_id_path_camel_case_token_stream,
                            #try_update_by_id_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#path_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #update_by_id_payload_camel_case_token_stream,
                            #try_update_by_id_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    #parameters_lower_case_token_stream.#prepare_and_execute_query_name_token_stream(&#app_info_state_name_token_stream).await
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #path_token_stream
            #payload_token_stream
            #prepare_and_execute_query_token_stream
            #try_update_by_id_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{update_by_id_token_stream}");
    let update_token_stream = {
        let update_name_camel_case_stringified = "Update";
        let update_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&update_name_camel_case_stringified.to_string());
        let update_parameters_camel_case_token_stream = {
            let update_parameters_camel_case_stringified = format!("{update_name_camel_case_stringified}{parameters_camel_case_stringified}");
            update_parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_payload_element_camel_case_token_stream = {
            let update_payload_element_camel_case_stringified = format!("{update_name_camel_case_stringified}{payload_element_camel_case_stringified}");
            update_payload_element_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_payload_element_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let update_payload_camel_case_token_stream = quote::quote!{Vec<#update_payload_element_camel_case_token_stream>};
        let try_update_error_named_camel_case_token_stream = {
            let try_update_error_named_camel_case_stringified = format!("{try_camel_case_stringified}{update_name_camel_case_stringified}{error_named_camel_case_stringified}");
            try_update_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_update_response_variants_token_stream = {
            let try_update_response_variants_stringified = format!("{try_camel_case_stringified}{update_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_update_response_variants_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let parameters_token_stream = {
            quote::quote!{
                #parameters_derive_token_stream
                pub struct #update_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #update_payload_camel_case_token_stream,
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
                #payload_derive_token_stream
                pub struct #update_payload_element_camel_case_token_stream {
                    pub #id_field_ident: crate::server::postgres::bigserial::Bigserial,
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let prepare_and_execute_query_token_stream = {
            //todo - check if primary keys are unique in the input array
            let prepare_and_execute_query_error_token_stream = {
                let error_path_stringified = format!("{try_camel_case_stringified}{update_name_camel_case_stringified}");
                error_path_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &try_update_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
            );
            let query_string_token_stream = {
                let query_token_stream = {
                    let column_names = fields_named.iter().enumerate().fold(std::string::String::default(), |mut acc, (index, field)| {
                        let field_ident = field.ident.clone()
                            .unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                        let possible_dot_space = match (index + 1) == fields_named_len {
                            true => "",
                            false => dot_space,
                        };
                        acc.push_str(&format!("{field_ident}{possible_dot_space}"));
                        acc
                    });
                    let declarations = {
                        let fields_named_filtered = fields_named.iter().filter(|field|*field != &id_field).collect::<Vec<&syn::Field>>();
                        let fields_named_len = fields_named_filtered.len();
                        fields_named_filtered.iter().enumerate().fold(std::string::String::default(), |mut acc, (index, field)| {
                            let field_ident = field.ident.clone().unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                            let possible_dot_space = match (index + 1) == fields_named_len {
                                true => "",
                                false => dot_space,
                            };
                            acc.push_str(&format!("{field_ident} = data.{field_ident}{possible_dot_space}"));
                            acc
                        })
                    };
                    let query_stringified = format!("\"{{}} {{}} {{}} t {{}} {declarations} {{}} (values {{values}}) as data({column_names}) where t.{id_field_ident} = data.{id_field_ident} {returning_stringified} data.{id_field_ident}\"");
                    query_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let additional_parameters_modification_token_stream = fields_named.iter().map(|field| {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    quote::quote!{
                        match #crate_server_postgres_bind_query_bind_query_try_generate_bind_increments_token_stream(
                            &element.#field_ident,
                            &mut increment
                        ) {
                            Ok(value) => {
                                element_value.push_str(&format!("{value}, "));
                            },
                            Err(e) => {
                                return #try_update_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                            },
                        };
                    }
                });
                quote::quote!{
                    #increment_initialization_token_stream
                    let mut values = std::string::String::default();
                    for element in &self.#payload_lower_case_token_stream {
                        values.push_str(&format!(
                            "({}), ",
                            {
                                let mut element_value = std::string::String::default();
                                #(#additional_parameters_modification_token_stream)*
                                element_value.pop();//todo - remove it 
                                element_value.pop();
                                element_value
                            }
                        ));
                    }
                    values.pop();
                    values.pop();
                    format!(
                        #query_token_stream,
                        #crate_server_postgres_constants_update_name_token_stream,
                        ROUTE_NAME,
                        #crate_server_postgres_constants_as_name_token_stream,
                        #crate_server_postgres_constants_set_name_token_stream,
                        #crate_server_postgres_constants_from_name_token_stream,
                    )
                }
            };
            let binded_query_token_stream = {
                let binded_query_modifications_token_stream = fields_named.iter().map(|field|{
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    quote::quote!{
                        query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                            element.#field_ident,
                            query,
                        ); 
                    }
                });
                quote::quote!{
                    let mut query = #sqlx_query_sqlx_postgres_token_stream(&#query_string_name_token_stream);
                    for element in self.#payload_lower_case_token_stream {
                        #(#binded_query_modifications_token_stream)*
                    }
                    query
                }
            };
            quote::quote!{
                impl #update_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        #app_info_state_name_token_stream: &#app_info_state_path,
                    ) -> #try_update_response_variants_token_stream {
                        let #query_string_name_token_stream = {
                            #query_string_token_stream
                        };
                        let expected_updated_primary_keys = self
                            .#payload_lower_case_token_stream
                            .iter()
                            .map(|element| element.#id_field_ident.to_inner().clone()) //todo - maybe its not a good idea to remove .clone here coz in macro dont know what type
                            .collect::<Vec<#id_field_type>>();
                        let #binded_query_name_token_stream = {
                            #binded_query_token_stream
                        };
                        #acquire_pool_and_connection_token_stream
                        let mut #postgres_transaction_token_stream = match {
                            use sqlx::Acquire;
                            #pg_connection_token_stream.#begin_token_stream()
                        }
                        .await
                        {
                            Ok(value) => value,
                            Err(e) => {
                                #from_log_and_return_error_token_stream;
                            }
                        };
                        match #binded_query_name_token_stream.fetch_all(#postgres_transaction_token_stream.as_mut()).await {//todo maybe instead use fetch while try_next ? 
                            Ok(updated_rows) => {
                                let typed_updated_rows = {
                                    let mut typed_updated_rows = Vec::with_capacity(updated_rows.len());
                                    for updated_row in updated_rows {
                                        match #primary_key_try_from_sqlx_row_name_token_stream(&updated_row) {
                                            Ok(updated_row_primary_key) => {
                                                typed_updated_rows.push(updated_row_primary_key);
                                            }
                                            Err(e) => match #postgres_transaction_token_stream.#rollback_token_stream().await {
                                                Ok(_) => {
                                                    #from_log_and_return_error_token_stream;
                                                }
                                                Err(rollback_error) => {
                                                    //todo  BIG QUESTION - WHAT TO DO IF ROLLBACK FAILED? INFINITE LOOP TRYING TO ROLLBACK?
                                                    let error = #prepare_and_execute_query_error_token_stream::PrimaryKeyFromRowAndFailedRollback {
                                                        primary_key_from_row: e,
                                                        rollback_error,
                                                        #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                                    };
                                                    #error_log_call_token_stream
                                                    return #try_update_response_variants_token_stream::from(error);
                                                }
                                            },
                                        }
                                    }
                                    typed_updated_rows
                                };
                                {
                                    let non_existing_primary_keys = {
                                        let mut non_existing_primary_keys =
                                            Vec::with_capacity(expected_updated_primary_keys.len());
                                        for expected_updated_primary_key in expected_updated_primary_keys {
                                            if let false = typed_updated_rows.contains(&expected_updated_primary_key) {
                                                non_existing_primary_keys.push(expected_updated_primary_key);
                                            }
                                        }
                                        non_existing_primary_keys
                                    };
                                    if let false = non_existing_primary_keys.is_empty() {
                                        match #postgres_transaction_token_stream.#rollback_token_stream().await {
                                            Ok(_) => {
                                                let error = #prepare_and_execute_query_error_token_stream::NonExistingPrimaryKeys {
                                                    non_existing_primary_keys,
                                                    #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream, //todo how to show log from proc_macro
                                                };
                                                #error_log_call_token_stream
                                                return #try_update_response_variants_token_stream::from(error);
                                            }
                                            Err(e) => {
                                                let error = #prepare_and_execute_query_error_token_stream::NonExistingPrimaryKeysAndFailedRollback {
                                                    non_existing_primary_keys,
                                                    rollback_error: e,
                                                    #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream, //todo how to show log from proc_macro
                                                };
                                                #error_log_call_token_stream
                                                return #try_update_response_variants_token_stream::from(error);
                                            }
                                        }
                                    }
                                }
                                // println!("{:#?}", expected_updated_primary_keys);
                                match #postgres_transaction_token_stream.#commit_token_stream().await {
                                    Ok(_) => #try_update_response_variants_token_stream::#desirable_token_stream(()),
                                    Err(e) => {
                                        //todo  BIG QUESTION - WHAT TO DO IF COMMIT FAILED? INFINITE LOOP TRYING TO COMMIT?
                                        let error = #prepare_and_execute_query_error_token_stream::CommitFailed {
                                            commit_error: e,
                                            #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                        };
                                        #error_log_call_token_stream
                                        return #try_update_response_variants_token_stream::from(error); //todo - few variants or return ResponseVariants::from - with return ; and not
                                    }
                                }
                            }
                            Err(e) => match #postgres_transaction_token_stream.#rollback_token_stream().await {
                                Ok(_) => {
                                    #from_log_and_return_error_token_stream;
                                }
                                Err(rollback_error) => {
                                    //todo  BIG QUESTION - WHAT TO DO IF ROLLBACK FAILED? INFINITE LOOP TRYING TO ROLLBACK?
                                    let error = #prepare_and_execute_query_error_token_stream::UpdateAndRollbackFailed {
                                        update_error: e,
                                        rollback_error,
                                        #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream,
                                    };
                                    #error_log_call_token_stream
                                    return #try_update_response_variants_token_stream::from(error);
                                }
                            },
                        }
                    }
                }
            }
        };
        // println!("{prepare_and_execute_query_token_stream}");
        let try_update_error_named_token_stream = {
            let try_update_request_error_camel_case_token_stream = {
                let try_update_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{update_name_camel_case_stringified}{request_error_camel_case_stringified}");
                try_update_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_update_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_update_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_update_error_named_token_stream}");
        let http_request_token_stream = {
            let try_update_lower_case_token_stream = {
                let try_update_lower_case_stringified = format!("{try_lower_case_stringified}_{update_name_lower_case_stringified}");
                try_update_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_update_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{update_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_update_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #update_parameters_camel_case_token_stream,
                ) -> Result<(), #try_update_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_update_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
                        }
                    };
                    let url = format!(
                        "{}/api/{}/",
                        #server_location_name_token_stream,
                        ROUTE_NAME,
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
                        Err(e) => Err(#try_update_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let update_lower_case_token_stream = update_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {update_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            quote::quote!{
                pub async fn #update_lower_case_token_stream<'a>(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#update_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #update_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #update_payload_camel_case_token_stream,
                            #try_update_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => value,
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    #parameters_lower_case_token_stream.#prepare_and_execute_query_name_token_stream(&#app_info_state_name_token_stream).await
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #prepare_and_execute_query_token_stream
            #try_update_error_named_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // println!("{update_token_stream}");
    let gen = quote::quote! {
        #struct_options_token_stream
        #from_ident_for_ident_options_token_stream
        #(#structs_variants_token_stream)*
        #(#structs_variants_impl_from_token_stream)*
        #column_token_stream
        #column_select_token_stream
        #primary_key_try_from_sqlx_row_token_stream

        #create_batch_token_stream
        #create_token_stream
        #delete_by_id_token_stream
        #delete_with_body_token_stream
        #delete_token_stream
        #read_by_id_token_stream
        #read_with_body_token_stream
        #read_token_stream
        #update_by_id_token_stream
        #update_token_stream
    };
    // if ident == "" {
    //     println!("{gen}");
    // }
    gen.into()
}

enum Operation {
    // CreateBatch,
    // Create,
    DeleteById,
    // DeleteWithBody,//+
    // Delete,//+
    // ReadById,
    // ReadWithBody,//+
    // Read,//+
    UpdateById,
    // Update//+
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // Self::CreateBatch => write!(f, "create_batch"),
            // Self::Create => write!(f, "create"),
            Self::DeleteById => write!(f, "delete_by_id"),
            // Self::DeleteWithBody => write!(f, "delete_with_body"),
            // Self::Delete => write!(f, "delete"),
            // Self::ReadById => write!(f, "read_by_id"),
            // Self::ReadWithBody => write!(f, "read_with_body"),
            // Self::Read => write!(f, "read"),
            Self::UpdateById => write!(f, "update_by_id"),
            // Self::Update => write!(f, "update"),
        }
    }
}


fn generate_create_or_replace_function_token_stream(
    ident_lower_case_stringified: &std::string::String,
    operation: &crate::Operation,
    proc_macro_name_ident_stringified: &std::string::String,
    fields_named: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    id_field: &syn::Field,
    payload_lower_case_token_stream: &proc_macro2::TokenStream
) -> proc_macro2::TokenStream {
    let create_or_replace_function_name_original_token_stream = {
        let create_or_replace_function_name_original_stringified =
            format!("\"{ident_lower_case_stringified}_{operation}\"");
        create_or_replace_function_name_original_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_or_replace_function_name_original_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let create_or_replace_function_name_additions_token_stream = fields_named.iter().filter_map(|field|match field == id_field {
        true => None,
        false => {
            let field_ident = field.ident.clone()
                .unwrap_or_else(|| {
                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                });
            let format_value_token_stream = {
                let format_value_stringified = format!("\"_{field_ident}\"");
                format_value_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {format_value_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            Some(quote::quote!{
                if self.#payload_lower_case_token_stream.#field_ident.is_some() {
                    value.push_str(&format!(#format_value_token_stream));
                }
            })
        },
    });
    quote::quote! {
        let mut value = std::string::String::from(#create_or_replace_function_name_original_token_stream);
        #(#create_or_replace_function_name_additions_token_stream)*
        value
    }
}

// `DO` blocks cannot use bound parameters.  If you need to pass in values then you can create a temporary function and call that instead, though it's a bit more of a hassle.

// #[derive(strum_macros::Display)]//strum_macros::EnumIter, 
// enum Attribute {
//     GeneratePostgresqlCrudId,
// }

// impl Attribute {
//     pub fn to_str(&self) -> &str {
//         match self {
//             Attribute::GeneratePostgresqlCrudId => "generate_postgresql_crud_id",
//         }
//     }
//     pub fn attribute_view(&self) -> String {
//         self.to_str().to_string()
//     }
// }
