#[proc_macro_derive(
    GeneratePostgresqlCrud,
    attributes(
        generate_postgresql_crud_id,
    )
)]
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
    // println!("{id_field:#?}");
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
        column_names_factorial(
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
    let column_token_stream = {
        let column_ident_token_stream = {
            let column_ident_stringified = format!("{ident}Column");
            column_ident_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {column_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
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
            quote::quote! {
                impl #column_select_ident_token_stream {
                    fn options_try_from_sqlx_row<'a, R: sqlx::Row>(
                        &self,
                        row: &'a R,
                    ) -> sqlx::Result<#struct_options_ident_token_stream>
                    where
                        &'a std::primitive::str: sqlx::ColumnIndex<R>,
                        Option<i64>: sqlx::decode::Decode<'a, R::Database>,
                        Option<i64>: sqlx::types::Type<R::Database>,
                        Option<String>: sqlx::decode::Decode<'a, R::Database>,
                        Option<String>: sqlx::types::Type<R::Database>,
                        Option<String>: sqlx::decode::Decode<'a, R::Database>,
                        Option<String>: sqlx::types::Type<R::Database>,
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
        quote::quote! {
            #column_select_struct_token_stream
            #generate_query_token_stream
            #impl_default_token_stream
            #from_option_self_token_stream
            #serde_urlencoded_parameter_token_stream
            #options_try_from_sqlx_row_token_stream
        }
    };
    let parameters_camel_case_stringified = "Parameters";
    // let parameters_camel_case_token_stream = parameters_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    //     .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {parameters_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let path_camel_case_stringified = "Path";
    // let path_camel_case_token_stream = path_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    //     .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {path_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let query_camel_case_stringified = "Query";
    // let query_camel_case_token_stream = query_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    //     .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE)); 
    let payload_camel_case_stringified = "Payload";
    // let payload_camel_case_token_stream = payload_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    //     .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {payload_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE)); 
    let for_url_encoding_camel_case_stringified = "ForUrlEncoding";
    let payload_element_camel_case_stringified = format!("{payload_camel_case_stringified}Element");
    let prepare_and_execute_query_token_stream = quote::quote!{prepare_and_execute_query};
    let try_camel_case_stringified = "Try";
    let response_variants_camel_case_stringified = "ResponseVariants";
    let path_to_crud = "crate::repositories_types::tufa_server::routes::api::cats::";
    let app_info_state_path = quote::quote!{crate::repositories_types::tufa_server::routes::api::cats::DynArcGetConfigGetPostgresPoolSendSync};
    let error_log_call_token_stream = quote::quote!{
        crate::common::error_logs_logic::error_log::ErrorLog::error_log(
            &error,
            app_info_state.as_ref(),
        );
    };
    let fields_named_len = fields_named.len();
    let dot_space = ", ";
    
    // let path_lower_case_token_stream= quote::quote!{path};
    // let query_lower_case_token_stream= quote::quote!{query};
    // let payload_lower_case_token_stream= quote::quote!{payload};
    // let select_lower_case_token_stream= quote::quote!{select};
    let create_batch_token_stream = {
        let create_batch_name_camel_case_stringified = "CreateBatch";
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
        let create_batch_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&create_batch_name_camel_case_stringified);
        let prepare_and_execute_query_response_variants_token_stream = {
            let try_response_variants_path_stringified = format!("{path_to_crud}{create_batch_name_lower_case_stringified}::{try_camel_case_stringified}{create_batch_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_response_variants_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_response_variants_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let prepare_and_execute_query_error_token_stream = {
            let error_path_stringified = format!("{path_to_crud}{create_batch_name_lower_case_stringified}::{try_camel_case_stringified}{create_batch_name_camel_case_stringified}");
            error_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
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
            let query_stringified = format!("\"{{}} {{}} {{}}({column_names}) {{}} {{bind_increments}}\"");
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
                    match crate::server::postgres::bind_query::BindQuery::try_generate_bind_increments(&element.#field_ident, &mut increment) {
                        Ok(value) => {
                            element_bind_increments.push_str(&format!(#possible_dot_space_format));
                        },
                        Err(e) => {
                            return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                                checked_add: e.into_serialize_deserialize_version(), 
                                code_occurence: crate::code_occurence_tufa_common!(),
                            };
                        },
                    }
                }
            }).collect::<Vec<proc_macro2::TokenStream>>()
        };
        let bind_value_to_query_modificate_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
            true => None,
            false => {
                let field_ident = field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                Some(quote::quote!{
                    query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(element.#field_ident, query);  
                })
            },
        });
        quote::quote!{
            #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
            pub struct #create_batch_parameters_camel_case_token_stream {
                pub payload: Vec<#create_batch_payload_element_camel_case_token_stream>,
            }
            #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
            pub struct #create_batch_payload_element_camel_case_token_stream {
                #(#fields_with_excluded_id_token_stream),*
            }
            impl #create_batch_parameters_camel_case_token_stream {
                pub async fn #prepare_and_execute_query_token_stream(
                    self,
                    app_info_state: &#app_info_state_path,
                ) -> #prepare_and_execute_query_response_variants_token_stream
                {
                    let bind_increments = {
                        let mut increment: u64 = 0;
                        let mut bind_increments = std::string::String::default();
                        for element in &self.payload {
                            let element_bind_increments = {
                                let mut element_bind_increments = std::string::String::default();
                                #(#element_bind_increments_modificate_token_stream)*
                                element_bind_increments
                            };
                            bind_increments.push_str(&format!("({element_bind_increments}), "));
                        }
                        bind_increments.pop();
                        bind_increments.pop();
                        bind_increments
                    };
                    let query_string = format!(
                        #query_token_stream,
                        crate::server::postgres::constants::INSERT_NAME,
                        crate::server::postgres::constants::INTO_NAME,
                        crate::repositories_types::tufa_server::routes::api::cats::CATS,
                        crate::server::postgres::constants::VALUES_NAME
                    );
                    println!("{query_string}");
                    let binded_query = {
                        let mut query = sqlx::query::<sqlx::Postgres>(&query_string);
                        for element in self.payload {
                            #(#bind_value_to_query_modificate_token_stream)*
                        }
                        query
                    };
                    match binded_query
                        .execute(app_info_state.get_postgres_pool())
                        .await
                    {
                        Ok(_) => {
                            //todo - is need to return rows affected?
                            #prepare_and_execute_query_response_variants_token_stream::Desirable(())
                        }
                        Err(e) => {
                            let error = #prepare_and_execute_query_error_token_stream::from(e);
                            #error_log_call_token_stream
                            #prepare_and_execute_query_response_variants_token_stream::from(error)
                        }
                    }
                }
            }
        }
    };
    let create_token_stream = {
        let create_name_camel_case_stringified = "Create";
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
        let create_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&create_name_camel_case_stringified);
        let prepare_and_execute_query_response_variants_token_stream = {
            let try_response_variants_path_stringified = format!("{path_to_crud}{create_name_lower_case_stringified}::{try_camel_case_stringified}{create_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_response_variants_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_response_variants_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let prepare_and_execute_query_error_token_stream = {
            let error_path_stringified = format!("{path_to_crud}{create_name_lower_case_stringified}::{try_camel_case_stringified}{create_name_camel_case_stringified}");
            error_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
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
        let binded_query_modifications_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
            true => None,
            false => {
                let field_ident = field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                Some(quote::quote!{
                    query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(self.payload.#field_ident, query);
                })
            },
        });
        quote::quote!{
            #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
            pub struct #create_parameters_camel_case_token_stream {
                pub payload: #create_payload_camel_case_token_stream,
            }
            #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
            pub struct #create_payload_camel_case_token_stream {
                #(#fields_with_excluded_id_token_stream),*
            }
            impl #create_parameters_camel_case_token_stream {
                pub async fn #prepare_and_execute_query_token_stream(
                    self,
                    app_info_state: &#app_info_state_path,
                ) -> #prepare_and_execute_query_response_variants_token_stream
                {
                    let query_string = format!(
                        #query_token_stream,
                        crate::server::postgres::constants::INSERT_NAME,
                        crate::server::postgres::constants::INTO_NAME,
                        crate::repositories_types::tufa_server::routes::api::cats::CATS,
                        crate::server::postgres::constants::VALUES_NAME
                    );
                    println!("{query_string}");
                    let binded_query = {
                        let mut query = sqlx::query::<sqlx::Postgres>(&query_string);
                        #(#binded_query_modifications_token_stream)*
                        query
                    };
                    match binded_query
                        .execute(app_info_state.get_postgres_pool())
                        .await
                    {
                        Ok(_) => {
                            //todo - is need to return rows affected?
                            #prepare_and_execute_query_response_variants_token_stream::Desirable(())
                        }
                        Err(e) => {
                            let error = #prepare_and_execute_query_error_token_stream::from(e);
                            #error_log_call_token_stream
                            #prepare_and_execute_query_response_variants_token_stream::from(error)
                        }
                    }
                }
            }
        }
    };
    let delete_by_id_token_stream = {
        let delete_by_id_name_camel_case_stringified = "DeleteById";
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
        let id_field_ident = id_field.ident.clone()
            .unwrap_or_else(|| {
                panic!("{proc_macro_name_ident_stringified} id_field.ident is None")
            });
        let delete_by_id_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&delete_by_id_name_camel_case_stringified);
        let prepare_and_execute_query_response_variants_token_stream = {
            let try_response_variants_path_stringified = format!("{path_to_crud}{delete_by_id_name_lower_case_stringified}::{try_camel_case_stringified}{delete_by_id_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_response_variants_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_response_variants_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let prepare_and_execute_query_error_token_stream = {
            let error_path_stringified = format!("{path_to_crud}{delete_by_id_name_lower_case_stringified}::{try_camel_case_stringified}{delete_by_id_name_camel_case_stringified}");
            error_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let query_token_stream = {
            let query_stringified = format!("\"{{}} {{}} {{}} {{}} {id_field_ident} = $1\"");
            query_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let binded_query_modifications_token_stream = quote::quote!{
            query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(self.path.#id_field_ident, query);
        };
        quote::quote!{
            #[derive(Debug, serde::Deserialize)]
            pub struct #delete_by_id_parameters_camel_case_token_stream {
                pub path: #delete_by_id_path_camel_case_token_stream,
            }
            #[derive(Debug, serde::Deserialize)]
            pub struct #delete_by_id_path_camel_case_token_stream {
                pub #id_field_ident: crate::server::postgres::bigserial::Bigserial,//#id_field_type
            }
            impl #delete_by_id_parameters_camel_case_token_stream {
                pub async fn #prepare_and_execute_query_token_stream(
                    self,
                    app_info_state: &#app_info_state_path,
                ) -> #prepare_and_execute_query_response_variants_token_stream
                {
                    let query_string = format!(
                        #query_token_stream,
                        crate::server::postgres::constants::DELETE_NAME,
                        crate::server::postgres::constants::FROM_NAME,
                        crate::repositories_types::tufa_server::routes::api::cats::CATS,
                        crate::server::postgres::constants::WHERE_NAME
                    );
                    println!("{query_string}");
                    let binded_query = {
                        let mut query = sqlx::query::<sqlx::Postgres>(&query_string);
                        #binded_query_modifications_token_stream
                        query
                    };
                    match binded_query
                        .execute(app_info_state.get_postgres_pool())
                        .await
                    {
                        Ok(_) => {
                            //todo - is need to return rows affected?
                            #prepare_and_execute_query_response_variants_token_stream::Desirable(())
                        }
                        Err(e) => {
                            let error = #prepare_and_execute_query_error_token_stream::from(e);
                            #error_log_call_token_stream
                            #prepare_and_execute_query_response_variants_token_stream::from(error)
                        }
                    }
                }
            }
        }
    };
    let delete_with_body_token_stream = {
        let delete_with_body_name_camel_case_stringified = "DeleteWithBody";
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
        let id_field_ident = id_field.ident.clone()
            .unwrap_or_else(|| {
                panic!("{proc_macro_name_ident_stringified} id_field.ident is None")
            });
        let fields_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
            true => None,
            false => {
                let field_ident = field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                Some(quote::quote!{
                    pub #field_ident: Option<Vec<crate::server::postgres::regex_filter::RegexFilter>>
                })
            },
        });
        let delete_with_body_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&delete_with_body_name_camel_case_stringified);
        let prepare_and_execute_query_response_variants_token_stream = {
            let try_response_variants_path_stringified = format!("{path_to_crud}{delete_with_body_name_lower_case_stringified}::{try_camel_case_stringified}{delete_with_body_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_response_variants_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_response_variants_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let prepare_and_execute_query_error_token_stream = {
            let error_path_stringified = format!("{path_to_crud}{delete_with_body_name_lower_case_stringified}::{try_camel_case_stringified}{delete_with_body_name_camel_case_stringified}");
            error_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let check_for_all_none_token_stream = {
            let (mut none_elements, mut match_elements) = fields_named.iter().fold(
                (
                    std::string::String::default(),
                    std::string::String::default()
                ), |mut acc, field| {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    acc.0.push_str(&format!("None{dot_space}"));
                    acc.1.push_str(&format!("&self.payload.{field_ident}{dot_space}"));
                    acc
                });
            none_elements.pop();
            none_elements.pop();
            match_elements.pop();
            match_elements.pop();
            let none_elements_token_stream = none_elements.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {none_elements} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let match_elements_token_stream = match_elements.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {match_elements} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            quote::quote!{
                if let (#none_elements_token_stream) = (#match_elements_token_stream) {
                    return #prepare_and_execute_query_response_variants_token_stream::NoParameters { 
                        no_parameters: std::string::String::from("no parameters"), 
                        code_occurence: crate::code_occurence_tufa_common!(),
                    };
                }
            }
        };
        let additional_parameters_id_modification_token_stream = {
            let query_part_token_stream = {
                let query_part_stringified = format!("\"{{prefix}} {id_field_ident} = {{}}({{}}[{{}}])\"");
                query_part_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_part_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                if let Some(value) = &self.payload.#id_field_ident {
                    let prefix = match additional_parameters.is_empty() {
                        true => crate::server::postgres::constants::WHERE_NAME.to_string(),
                        false => format!(" {}", crate::server::postgres::constants::AND_NAME),
                    };
                    let bind_increments = {
                        let mut bind_increments = std::string::String::default();
                        for element in value {
                            match crate::server::postgres::bind_query::BindQuery::try_generate_bind_increments(
                                element,
                                &mut increment
                            ) {
                                Ok(bind_increments_handle) => {
                                    bind_increments.push_str(&format!("{bind_increments_handle}, "));
                                },
                                Err(e) => {
                                    return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                                        checked_add: e.into_serialize_deserialize_version(), 
                                        code_occurence: crate::code_occurence_tufa_common!(),
                                    };
                                },
                            }
                        }
                        bind_increments.pop();
                        bind_increments.pop();
                        bind_increments
                    };
                    additional_parameters.push_str(&format!(
                        #query_part_token_stream,
                        crate::server::postgres::constants::ANY_NAME,
                        crate::server::postgres::constants::ARRAY_NAME,
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
                    let handle_stringified = format!("\"{field_ident} ~ {{bind_increments_handle}} \"");
                    handle_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                Some(quote::quote!{
                    if let Some(value) = &self.payload.#field_ident {
                        let prefix = match additional_parameters.is_empty() {
                            true => crate::server::postgres::constants::WHERE_NAME.to_string(),
                            false => format!(" {}", crate::server::postgres::constants::AND_NAME),
                        };
                        let bind_increments = {
                            let mut bind_increments = std::string::String::default();
                            for (index, element) in value.iter().enumerate() {
                                match crate::server::postgres::bind_query::BindQuery::try_generate_bind_increments(
                                    element,
                                    &mut increment
                                ) {
                                    Ok(bind_increments_handle) => {
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
                                        return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                                            checked_add: e.into_serialize_deserialize_version(), 
                                            code_occurence: crate::code_occurence_tufa_common!(),
                                        };
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
        let binded_query_modifications_token_stream = fields_named.iter().map(|field|{
            let field_ident = field.ident.clone()
                .unwrap_or_else(|| {
                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                });
            quote::quote!{
                if let Some(values) = self.payload.#field_ident {
                    for value in values {
                        query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                            value, query,
                        );
                    }
                }
            }
        });
        quote::quote!{
            #[derive(Debug, serde::Deserialize)]
            pub struct #delete_with_body_parameters_camel_case_token_stream {
                pub payload: #delete_with_body_payload_camel_case_token_stream,
            }
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            pub struct #delete_with_body_payload_camel_case_token_stream {
                pub #id_field_ident: Option<Vec<crate::server::postgres::bigserial::Bigserial>>,
                #(#fields_with_excluded_id_token_stream),*
            }
            impl #delete_with_body_parameters_camel_case_token_stream {
                pub async fn #prepare_and_execute_query_token_stream(
                    self,
                    app_info_state: &#app_info_state_path,
                ) -> #prepare_and_execute_query_response_variants_token_stream
                {
                    #check_for_all_none_token_stream
                    let query_string = {
                        let additional_parameters = {
                            let mut additional_parameters = std::string::String::default();
                            let mut increment: u64 = 0;
                            #additional_parameters_id_modification_token_stream
                            #(#additional_parameters_modification_token_stream)*
                            additional_parameters
                        };
                        format!(
                            "{} {} {} {additional_parameters}",
                            crate::server::postgres::constants::DELETE_NAME,
                            crate::server::postgres::constants::FROM_NAME,
                            crate::repositories_types::tufa_server::routes::api::cats::CATS
                        )
                    };
                    println!("{query_string}");
                    let binded_query = {
                        let mut query = sqlx::query::<sqlx::Postgres>(&query_string);
                        #(#binded_query_modifications_token_stream)*
                        query
                    };
                    match binded_query
                        .execute(app_info_state.get_postgres_pool())
                        .await
                    {
                        Ok(_) => {
                            //todo - is need to return rows affected?
                            #prepare_and_execute_query_response_variants_token_stream::Desirable(())
                        }
                        Err(e) => {
                            let error = #prepare_and_execute_query_error_token_stream::from(e);
                            #error_log_call_token_stream
                            #prepare_and_execute_query_response_variants_token_stream::from(error)
                        }
                    }
                }
            }
        }
    };
    let delete_token_stream = {
        let delete_name_camel_case_stringified = "Delete";
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
        let fields_into_url_encoding_version_with_excluded_id_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
            true => None,
            false => {
                let field_ident = field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                Some(quote::quote!{
                    let #field_ident = self.#field_ident.map(|value| {
                        crate::common::serde_urlencoded::SerdeUrlencodedParameter::serde_urlencoded_parameter(
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
        let delete_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&delete_name_camel_case_stringified);
        let prepare_and_execute_query_response_variants_token_stream = {
            let try_response_variants_path_stringified = format!("{path_to_crud}{delete_name_lower_case_stringified}::{try_camel_case_stringified}{delete_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_response_variants_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_response_variants_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let prepare_and_execute_query_error_token_stream = {
            let error_path_stringified = format!("{path_to_crud}{delete_name_lower_case_stringified}::{try_camel_case_stringified}{delete_name_camel_case_stringified}");
            error_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let check_for_all_none_token_stream = {
            let (mut none_elements, mut match_elements) = fields_named.iter().filter(|field|*field != &id_field).fold(
                (
                    std::string::String::default(),
                    std::string::String::default()
                ), |mut acc, field| {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    acc.0.push_str(&format!("None{dot_space}"));
                    acc.1.push_str(&format!("&self.query.{field_ident}{dot_space}"));
                    acc
                });
            none_elements.pop();
            none_elements.pop();
            match_elements.pop();
            match_elements.pop();
            let none_elements_token_stream = none_elements.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {none_elements} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let match_elements_token_stream = match_elements.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {match_elements} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            quote::quote!{
                if let (#none_elements_token_stream) = (#match_elements_token_stream) {
                    return #prepare_and_execute_query_response_variants_token_stream::NoParameters { 
                        no_parameters: std::string::String::from("no parameters"), 
                        code_occurence: crate::code_occurence_tufa_common!(),
                    };
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
                    let handle_stringified = format!("\"{field_ident} = ${{increment}}\"");
                    handle_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                Some(quote::quote!{
                    if let Some(value) = &self.query.#field_ident {
                        match crate::server::postgres::bind_query::BindQuery::try_increment(
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
                                return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                                    checked_add: e.into_serialize_deserialize_version(), 
                                    code_occurence: crate::code_occurence_tufa_common!() 
                                };
                            },
                        }
                    }
                })
            },
        });
        let binded_query_modifications_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
            true => None,
            false => {
                let field_ident = field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                Some(quote::quote!{
                    if let Some(value) = self.query.#field_ident {
                        query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(value, query);
                    }
                })
            },
        });
        quote::quote!{
            #[derive(Debug, serde::Deserialize)]
            pub struct #delete_parameters_camel_case_token_stream {
                pub query: #delete_query_camel_case_token_stream,
            }
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            pub struct #delete_query_camel_case_token_stream {
                #(#fields_with_excluded_id_token_stream),*
            }
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            struct #delete_query_for_url_encoding_camel_case_token_stream {
                #(#fields_for_url_encoding_with_excluded_id_token_stream),*
            }
            impl #delete_query_camel_case_token_stream {
                fn into_url_encoding_version(self) -> #delete_query_for_url_encoding_camel_case_token_stream {
                    #(#fields_into_url_encoding_version_with_excluded_id_token_stream)*
                    #delete_query_for_url_encoding_camel_case_token_stream {
                        #(#fields_into_url_encoding_version_constract_with_excluded_id_token_stream),*
                    }
                }
            }
            impl #delete_parameters_camel_case_token_stream {
                pub async fn #prepare_and_execute_query_token_stream(
                    self,
                    app_info_state: &#app_info_state_path,
                ) -> #prepare_and_execute_query_response_variants_token_stream
                {
                    #check_for_all_none_token_stream
                    let query_string = {
                        let additional_parameters = {
                            let mut additional_parameters = std::string::String::default();
                            let mut increment: u64 = 0;
                            
                            #(#additional_parameters_modification_token_stream)*
                            additional_parameters
                        };
                        format!(
                            "{} {} {} {} {additional_parameters}",
                            crate::server::postgres::constants::DELETE_NAME,
                            crate::server::postgres::constants::FROM_NAME,
                            crate::repositories_types::tufa_server::routes::api::cats::CATS,
                            crate::server::postgres::constants::WHERE_NAME
                        )
                    };
                    println!("{query_string}");
                    let binded_query = {
                        let mut query = sqlx::query::<sqlx::Postgres>(&query_string);
                        #(#binded_query_modifications_token_stream)*
                        query
                    };
                    match binded_query
                        .execute(app_info_state.get_postgres_pool())
                        .await
                    {
                        Ok(_) => {
                            //todo - is need to return rows affected?
                            #prepare_and_execute_query_response_variants_token_stream::Desirable(())
                        }
                        Err(e) => {
                            let error = #prepare_and_execute_query_error_token_stream::from(e);
                            #error_log_call_token_stream
                            #prepare_and_execute_query_response_variants_token_stream::from(error)
                        }
                    }
                }
            }
        }
    };
    let read_by_id_token_stream = {
        let read_by_id_name_camel_case_stringified = "ReadById";
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
        let id_field_ident = id_field.ident.clone()
            .unwrap_or_else(|| {
                panic!("{proc_macro_name_ident_stringified} id_field.ident is None")
            });
        let read_by_id_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&read_by_id_name_camel_case_stringified);
        let prepare_and_execute_query_response_variants_token_stream = {
            let try_response_variants_path_stringified = format!("{path_to_crud}{read_by_id_name_lower_case_stringified}::{try_camel_case_stringified}{read_by_id_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_response_variants_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_response_variants_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let prepare_and_execute_query_error_token_stream = {
            let error_path_stringified = format!("{path_to_crud}{read_by_id_name_lower_case_stringified}::{try_camel_case_stringified}{read_by_id_name_camel_case_stringified}");
            error_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let query_token_stream = {
            let query_stringified = format!("\"{{}} {{}} {{}} {{}} {{}} {id_field_ident} = $1\"");
            query_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let binded_query_modifications_token_stream = quote::quote!{
            query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                self.path.#id_field_ident, query,
            );
        };
        quote::quote!{
            #[derive(Debug, serde::Deserialize)]
            pub struct #read_by_id_parameters_camel_case_token_stream {
                pub path: #read_by_id_path_camel_case_token_stream,
                pub query: #read_by_id_query_camel_case_token_stream,
            }
            #[derive(Debug, serde::Deserialize)]
            pub struct #read_by_id_path_camel_case_token_stream {
                pub #id_field_ident: crate::server::postgres::bigserial::Bigserial//#id_field_type,
            }
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            pub struct #read_by_id_query_camel_case_token_stream {
                pub select: Option<#column_select_ident_token_stream>,
            }
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            struct #read_by_id_query_for_url_encoding_camel_case_token_stream {
                select: Option<std::string::String>,
            }
            impl #read_by_id_query_camel_case_token_stream {
                fn into_url_encoding_version(self) -> #read_by_id_query_for_url_encoding_camel_case_token_stream {
                    let select = self.select.map(|value| {
                        crate::common::serde_urlencoded::SerdeUrlencodedParameter::serde_urlencoded_parameter(
                            value,
                        )
                    });
                    #read_by_id_query_for_url_encoding_camel_case_token_stream {
                        select
                    }
                }
            }
            impl #read_by_id_parameters_camel_case_token_stream {
                pub async fn #prepare_and_execute_query_token_stream(
                    self,
                    app_info_state: &#app_info_state_path,
                ) -> #prepare_and_execute_query_response_variants_token_stream
                {
                    let select = self.query.select.unwrap_or_default();
                    let query_string = format!(
                        #query_token_stream,
                        crate::server::postgres::constants::SELECT_NAME,
                        crate::server::postgres::generate_query::GenerateQuery::generate_query(&select),
                         crate::server::postgres::constants::FROM_NAME,
                        crate::repositories_types::tufa_server::routes::api::cats::CATS,
                        crate::server::postgres::constants::WHERE_NAME,
                    );
                    println!("{query_string}");
                    let binded_query = {
                        let mut query = sqlx::query::<sqlx::Postgres>(&query_string);
                        #binded_query_modifications_token_stream
                        query
                    };
                    match binded_query.fetch_one(app_info_state.get_postgres_pool()).await {
                        Ok(row) => match select.options_try_from_sqlx_row(&row) {
                            Ok(value) => #prepare_and_execute_query_response_variants_token_stream::Desirable(value),
                            Err(e) => {
                                let error = #prepare_and_execute_query_error_token_stream::from(e);
                                #error_log_call_token_stream
                                #prepare_and_execute_query_response_variants_token_stream::from(error)
                            },
                        },
                        Err(e) => {
                            let error = #prepare_and_execute_query_error_token_stream::from(e);
                            #error_log_call_token_stream
                            #prepare_and_execute_query_response_variants_token_stream::from(error)
                        },
                    }
                }
            }
        }
    };
    let read_with_body_token_stream = {
        let read_with_body_name_camel_case_stringified = "ReadWithBody";
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
        let id_field_ident = id_field.ident.clone()
            .unwrap_or_else(|| {
                panic!("{proc_macro_name_ident_stringified} id_field.ident is None")
            });
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
        let read_with_body_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&read_with_body_name_camel_case_stringified);
        let prepare_and_execute_query_response_variants_token_stream = {
            let try_response_variants_path_stringified = format!("{path_to_crud}{read_with_body_name_lower_case_stringified}::{try_camel_case_stringified}{read_with_body_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_response_variants_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_response_variants_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let prepare_and_execute_query_error_token_stream = {
            let error_path_stringified = format!("{path_to_crud}{read_with_body_name_lower_case_stringified}::{try_camel_case_stringified}{read_with_body_name_camel_case_stringified}");
            error_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let additional_parameters_id_modification_token_stream = {
            let query_part_token_stream = {
                let query_part_stringified = format!("\"{{prefix}} {id_field_ident} = {{}}({{}}[{{}}])\"");
                query_part_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_part_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                if let Some(value) = &self.payload.#id_field_ident {
                    let prefix = match additional_parameters.is_empty() {
                        true => crate::server::postgres::constants::WHERE_NAME.to_string(),
                        false => format!(" {}", crate::server::postgres::constants::AND_NAME),
                    };
                    let bind_increments = {
                        let mut bind_increments = std::string::String::default();
                        for element in value {
                            match crate::server::postgres::bind_query::BindQuery::try_generate_bind_increments(
                                element,
                                &mut increment
                            ) {
                                Ok(bind_increments_handle) => {
                                    bind_increments.push_str(&format!("{bind_increments_handle}, "));
                                },
                                Err(e) => {
                                    return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                                        checked_add: e.into_serialize_deserialize_version(), 
                                        code_occurence: crate::code_occurence_tufa_common!(),
                                    };
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
                        crate::server::postgres::constants::ANY_NAME,
                        crate::server::postgres::constants::ARRAY_NAME,
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
                    let handle_stringified = format!("\"{field_ident} ~ {{bind_increments_handle}} \"");
                    handle_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                Some(quote::quote!{
                    if let Some(value) = &self.payload.#field_ident {
                        let prefix = match additional_parameters.is_empty() {
                            true => crate::server::postgres::constants::WHERE_NAME.to_string(),
                            false => format!(" {}", crate::server::postgres::constants::AND_NAME),
                        };
                        let bind_increments = {
                            let mut bind_increments = std::string::String::default();
                            for (index, element) in value.iter().enumerate() {
                                match crate::server::postgres::bind_query::BindQuery::try_generate_bind_increments(
                                    element,
                                    &mut increment
                                ) {
                                    Ok(bind_increments_handle) => {
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
                                        return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                                            checked_add: e.into_serialize_deserialize_version(), 
                                            code_occurence: crate::code_occurence_tufa_common!(),
                                        };
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
        let binded_query_modifications_token_stream = fields_named.iter().map(|field|{
            let field_ident = field.ident.clone()
                .unwrap_or_else(|| {
                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                });
            quote::quote!{
                if let Some(values) = self.payload.#field_ident {
                    for value in values {
                        query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                            value, query,
                        );
                    }
                }
            }
        });
        quote::quote!{
            #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
            pub struct #read_with_body_parameters_camel_case_token_stream {
                pub payload: #read_with_body_payload_camel_case_token_stream,
            }
            #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
            pub struct #read_with_body_payload_camel_case_token_stream {
                pub select: #column_select_ident_token_stream,
                pub #id_field_ident: Option<Vec<crate::server::postgres::bigserial::Bigserial>>,
                #(#fields_with_excluded_id_token_stream)*
                pub order_by: crate::server::postgres::order_by::OrderBy<CatColumn>,
                pub limit: crate::server::postgres::postgres_number::PostgresNumber,
                pub offset: crate::server::postgres::postgres_number::PostgresNumber,
            }
            impl #read_with_body_parameters_camel_case_token_stream {
                pub async fn #prepare_and_execute_query_token_stream(
                    self,
                    app_info_state: &#app_info_state_path,
                ) -> #prepare_and_execute_query_response_variants_token_stream
                {
                    let query_string = {
                        let mut query = std::string::String::default();
                        {
                            query.push_str(&format!(
                                "{} {}",
                                crate::server::postgres::constants::SELECT_NAME,
                                crate::server::postgres::generate_query::GenerateQuery::generate_query(
                                    &self.payload.select
                                )
                            ));
                        }
                        query.push_str(&format!(
                            " {} {}",
                            crate::server::postgres::constants::FROM_NAME,
                            crate::repositories_types::tufa_server::routes::api::cats::CATS
                        ));
                        let additional_parameters = {
                            let mut additional_parameters = std::string::String::default();
                            let mut increment: u64 = 0;
                            #additional_parameters_id_modification_token_stream
                            #(#additional_parameters_modification_token_stream)*
                            {
                                let prefix = match additional_parameters.is_empty() {
                                    true => "",
                                    false => " ",
                                };
                                let value = &self.payload.order_by;
                                let order_stringified = match &value.order {
                                    Some(order) => order.to_string(),
                                    None => crate::server::postgres::order::Order::default().to_string(),
                                };
                                additional_parameters.push_str(&format!(
                                    "{prefix}{} {} {order_stringified}",
                                    crate::server::postgres::constants::ORDER_BY_NAME,
                                    value.column
                                ));
                            }
                            {
                                let prefix = match additional_parameters.is_empty() {
                                    true => "",
                                    false => " ",
                                };
                                let value = match crate::server::postgres::bind_query::BindQuery::try_generate_bind_increments(
                                    &self.payload.limit,
                                    &mut increment
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                                            checked_add: e.into_serialize_deserialize_version(), 
                                            code_occurence: crate::code_occurence_tufa_common!(),
                                        };
                                    },
                                };
                                additional_parameters.push_str(&format!(
                                    "{prefix}{} {value}",
                                    crate::server::postgres::constants::LIMIT_NAME,
                                ));
                            }
                            {
                                let prefix = match additional_parameters.is_empty() {
                                    true => "",
                                    false => " ",
                                };
                                let value = match crate::server::postgres::bind_query::BindQuery::try_generate_bind_increments(
                                    &self.payload.offset,
                                    &mut increment
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                                            checked_add: e.into_serialize_deserialize_version(), 
                                            code_occurence: crate::code_occurence_tufa_common!(),
                                        };
                                    },
                                };
                                additional_parameters.push_str(&format!(
                                    "{prefix}{} {value}",
                                    crate::server::postgres::constants::OFFSET_NAME,
                                ));
                            }
                            additional_parameters
                        };
                        query.push_str(&format!(" {additional_parameters}"));
                        query
                    };
                    println!("{query_string}");
                    let binded_query = {
                        let mut query = sqlx::query::<sqlx::Postgres>(&query_string);
                        #(#binded_query_modifications_token_stream)*
                        query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                            self.payload.limit,
                            query,
                        );
                        query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                            self.payload.offset,
                            query,
                        );
                        query
                    };
                    let vec_values = {
                        let mut rows = binded_query.fetch(app_info_state.get_postgres_pool());
                        let mut vec_values = Vec::new();
                        while let Some(row) = {
                            match {
                                use futures::TryStreamExt;
                                rows.try_next()
                            }
                            .await
                            {
                                Ok(option_pg_row) => option_pg_row,
                                Err(e) => {
                                    let error = #prepare_and_execute_query_error_token_stream::from(e);
                                    #error_log_call_token_stream
                                    return #prepare_and_execute_query_response_variants_token_stream::from(error);
                                }
                            }
                        } {
                            match self.payload.select.options_try_from_sqlx_row(&row) {
                                Ok(value) => {
                                    vec_values.push(value);
                                }
                                Err(e) => {
                                    let error = #prepare_and_execute_query_error_token_stream::from(e);
                                    #error_log_call_token_stream
                                    return #prepare_and_execute_query_response_variants_token_stream::from(error);
                                }
                            }
                        }
                        vec_values
                    };
                    #prepare_and_execute_query_response_variants_token_stream::Desirable(vec_values)
                }
            }
        }
    };
    let read_token_stream = {
        let read_name_camel_case_stringified = "Read";
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
        let fields_into_url_encoding_version_with_excluded_id_token_stream = fields_named.iter().map(|field| {
            let field_ident = field.ident.clone()
                .unwrap_or_else(|| {
                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                });
            quote::quote!{
                let #field_ident = self.#field_ident.map(|value| {
                    crate::common::serde_urlencoded::SerdeUrlencodedParameter::serde_urlencoded_parameter(
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
        let id_field_ident = id_field.ident.clone()
            .unwrap_or_else(|| {
                panic!("{proc_macro_name_ident_stringified} id_field.ident is None")
            });
        let read_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&read_name_camel_case_stringified);
        let prepare_and_execute_query_response_variants_token_stream = {
            let try_response_variants_path_stringified = format!("{path_to_crud}{read_name_lower_case_stringified}::{try_camel_case_stringified}{read_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_response_variants_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_response_variants_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let prepare_and_execute_query_error_token_stream = {
            let error_path_stringified = format!("{path_to_crud}{read_name_lower_case_stringified}::{try_camel_case_stringified}{read_name_camel_case_stringified}");
            error_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
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
                if let Some(value) = &self.query.#field_ident {
                    let prefix = match additional_parameters.is_empty() {
                        true => crate::server::postgres::constants::WHERE_NAME.to_string(),
                        false => format!(" {}", crate::server::postgres::constants::AND_NAME),
                    };
                    let value = match crate::server::postgres::bind_query::BindQuery::try_generate_bind_increments(
                        value,
                        &mut increment
                    ) {
                        Ok(value) => value,
                        Err(e) => {
                            return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                                checked_add: e.into_serialize_deserialize_version(), 
                                code_occurence: crate::code_occurence_tufa_common!(),
                            };
                        },
                    };
                    additional_parameters.push_str(&format!(
                        #handle_token_stream,
                        crate::server::postgres::constants::ANY_NAME,
                        crate::server::postgres::constants::ARRAY_NAME,
                    ));
                }
            }
        });
        let binded_query_modifications_token_stream = fields_named.iter().map(|field|{
            let field_ident = field.ident.clone()
                .unwrap_or_else(|| {
                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                });
            quote::quote!{
                if let Some(value) = self.query.#field_ident {
                    query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                        value, query,
                    );
                }
            }
        });
        quote::quote!{
            #[derive(Debug, serde::Deserialize)]
            pub struct #read_parameters_camel_case_token_stream {
                pub query: #read_query_camel_case_token_stream,
            }
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            pub struct #read_query_camel_case_token_stream {
                pub select: Option<#column_select_ident_token_stream>,
                pub #id_field_ident: Option<crate::server::postgres::bigserial_ids::BigserialIds>,
                #(#fields_with_excluded_id_token_stream)*
                pub order_by: Option<CatOrderByWrapper>,//todo
                pub limit: crate::server::postgres::postgres_number::PostgresNumber,
                pub offset: Option<crate::server::postgres::postgres_number::PostgresNumber>,
            }
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            struct #read_query_for_url_encoding_camel_case_token_stream {
                select: Option<std::string::String>,
                pub #id_field_ident: Option<std::string::String>,
                #(#fields_for_url_encoding_with_excluded_id_token_stream)*
                order_by: Option<std::string::String>,
                limit: std::string::String,
                offset: Option<std::string::String>,
            }
            impl #read_query_camel_case_token_stream {
                fn into_url_encoding_version(self) -> #read_query_for_url_encoding_camel_case_token_stream {
                    let select = self.select.map(|value| {
                        crate::common::serde_urlencoded::SerdeUrlencodedParameter::serde_urlencoded_parameter(
                            value,
                        )
                    });
                    #(#fields_into_url_encoding_version_with_excluded_id_token_stream)*
                    let order_by = self.order_by.map(|value| {
                        crate::common::serde_urlencoded::SerdeUrlencodedParameter::serde_urlencoded_parameter(
                            value,
                        )
                    });
                    let limit = crate::common::serde_urlencoded::SerdeUrlencodedParameter::serde_urlencoded_parameter(
                        self.limit,
                    );
                    let offset = self.offset.map(|value| {
                        crate::common::serde_urlencoded::SerdeUrlencodedParameter::serde_urlencoded_parameter(
                            value,
                        )
                    });
                    #read_query_for_url_encoding_camel_case_token_stream {
                        select,
                        #(#fields_into_url_encoding_version_constract_with_excluded_id_token_stream)*
                        order_by,
                        limit,
                        offset,
                    }
                }
            }
            impl #read_parameters_camel_case_token_stream {
                pub async fn #prepare_and_execute_query_token_stream(
                    self, //impl crate::server::routes::helpers::bind_sqlx_query::BindSqlxQuer + crate::server::postgres::generate_query::GenerateQuery
                    app_info_state: &#app_info_state_path,
                ) -> #prepare_and_execute_query_response_variants_token_stream
                {
                    let select = #column_select_ident_token_stream::from(self.query.select.clone());
                    let query_string = {
                        let additional_parameters = {
                            let mut additional_parameters = std::string::String::default();
                            let mut increment: u64 = 0;
                            #(#additional_parameters_modification_token_stream)*
                            if let Some(value) = &self.query.order_by {
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
                                    crate::server::postgres::constants::ORDER_BY_NAME,
                                    value.0.column
                                ));
                            }
                            {
                                let prefix = match additional_parameters.is_empty() {
                                    true => "",
                                    false => " ",
                                };
                                let value = match crate::server::postgres::bind_query::BindQuery::try_generate_bind_increments(
                                    &self.query.limit,
                                    &mut increment
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                                            checked_add: e.into_serialize_deserialize_version(), 
                                            code_occurence: crate::code_occurence_tufa_common!(),
                                        };
                                    },
                                };
                                additional_parameters.push_str(&format!(
                                    "{prefix}{} {value}",
                                    crate::server::postgres::constants::LIMIT_NAME,
                                ));
                            }
                            if let Some(value) = &self.query.offset {
                                let prefix = match additional_parameters.is_empty() {
                                    true => "",
                                    false => " ",
                                };
                                let value = match crate::server::postgres::bind_query::BindQuery::try_generate_bind_increments(
                                    value,
                                    &mut increment
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                                            checked_add: e.into_serialize_deserialize_version(), 
                                            code_occurence: crate::code_occurence_tufa_common!(),
                                        };
                                    },
                                };
                                additional_parameters.push_str(&format!(
                                    "{prefix}{} {value}",
                                    crate::server::postgres::constants::OFFSET_NAME,
                                ));
                            }
                            additional_parameters
                        };
                        format!(
                            "{} {} {} {} {additional_parameters}",
                            crate::server::postgres::constants::SELECT_NAME,
                            crate::server::postgres::generate_query::GenerateQuery::generate_query(&select),
                            crate::server::postgres::constants::FROM_NAME,
                            crate::repositories_types::tufa_server::routes::api::cats::CATS
                        )
                    };
                    println!("{query_string}");
                    let binded_query = {
                        let mut query = sqlx::query::<sqlx::Postgres>(&query_string);
                        #(#binded_query_modifications_token_stream)*
                        query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                            self.query.limit,
                            query,
                        );
                        if let Some(value) = self.query.offset {
                            query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                                value, query,
                            );
                        }
                        query
                    };
                    let vec_values = {
                        let mut rows = binded_query.fetch(app_info_state.get_postgres_pool());
                        let mut vec_values = Vec::new();
                        while let Some(row) = {
                            match {
                                use futures::TryStreamExt;
                                rows.try_next()
                            }
                            .await
                            {
                                Ok(option_pg_row) => option_pg_row,
                                Err(e) => {
                                    let error = #prepare_and_execute_query_error_token_stream::from(e);
                                    #error_log_call_token_stream
                                    return #prepare_and_execute_query_response_variants_token_stream::from(error);
                                }
                            }
                        } {
                            match select.options_try_from_sqlx_row(&row) {
                                Ok(value) => {
                                    vec_values.push(value);
                                }
                                Err(e) => {
                                    let error = #prepare_and_execute_query_error_token_stream::from(e);
                                    #error_log_call_token_stream
                                    return #prepare_and_execute_query_response_variants_token_stream::from(error);
                                }
                            }
                        }
                        vec_values
                    };
                    #prepare_and_execute_query_response_variants_token_stream::Desirable(vec_values)
                }
            }
        }
    };
    let update_by_id_token_stream = {//todo WHY ITS RETURN SUCCESS EVEN IF ROW DOES NOT EXISTS?
        let update_by_id_name_camel_case_stringified = "UpdateById";
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
        let id_field_ident = id_field.ident.clone()
            .unwrap_or_else(|| {
                panic!("{proc_macro_name_ident_stringified} id_field.ident is None")
            });
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
        let update_by_id_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&update_by_id_name_camel_case_stringified);
        let prepare_and_execute_query_response_variants_token_stream = {
            let try_response_variants_path_stringified = format!("{path_to_crud}{update_by_id_name_lower_case_stringified}::{try_camel_case_stringified}{update_by_id_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_response_variants_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_response_variants_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let prepare_and_execute_query_error_token_stream = {
            let error_path_stringified = format!("{path_to_crud}{update_by_id_name_lower_case_stringified}::{try_camel_case_stringified}{update_by_id_name_camel_case_stringified}");
            error_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let check_for_all_none_token_stream = {
            let (mut none_elements, mut match_elements) = fields_named.iter().filter(|field|*field != &id_field).fold(
                (
                    std::string::String::default(),
                    std::string::String::default()
                ), |mut acc, field| {
                    let field_ident = field.ident.clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                    acc.0.push_str(&format!("None{dot_space}"));
                    acc.1.push_str(&format!("&self.payload.{field_ident}{dot_space}"));
                    acc
                });
            none_elements.pop();
            none_elements.pop();
            match_elements.pop();
            match_elements.pop();
            let none_elements_token_stream = none_elements.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {none_elements} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let match_elements_token_stream = match_elements.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {match_elements} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            quote::quote!{
                if let (#none_elements_token_stream) = (#match_elements_token_stream) {
                    return #prepare_and_execute_query_response_variants_token_stream::NoPayloadFields {//tood make standart between all Crud error variants
                        no_payload_fields: std::string::String::from("no payload fields"), 
                        code_occurence: crate::code_occurence_tufa_common!()
                    };
                }
            }
        };
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
                    if let Some(value) = &self.payload.#field_ident {
                        match crate::server::postgres::bind_query::BindQuery::try_increment(value, &mut increment) {
                            Ok(_) => {
                                query.push_str(&format!(#handle_token_stream));//add dot_space for all elements except last
                            },
                            Err(e) => {
                                return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                                    checked_add: e.into_serialize_deserialize_version(), 
                                    code_occurence: crate::code_occurence_tufa_common!() 
                                };
                            },
                        }
                    }
                }
            }).collect::<Vec<proc_macro2::TokenStream>>()
        };
        let additional_parameters_id_modification_token_stream = {
            let query_part_token_stream = {
                let query_part_stringified = format!("\" {{}} {id_field_ident} = ${{increment}}\"");
                query_part_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_part_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                match crate::server::postgres::bind_query::BindQuery::try_increment(&self.path.#id_field_ident, &mut increment) {
                    Ok(_) => {
                        query.push_str(&format!(
                            #query_part_token_stream,
                            crate::server::postgres::constants::WHERE_NAME,
                        ));
                    },
                    Err(e) => {
                        return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                            checked_add: e.into_serialize_deserialize_version(), 
                            code_occurence: crate::code_occurence_tufa_common!(),
                        };
                    },
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
                Some(quote::quote!{
                    if let Some(value) = self.payload.#field_ident {
                        query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                            value,
                            query,
                        );
                    }
                })
            }
        });
        let binded_query_id_modification_token_stream = quote::quote!{
            query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                self.path.#id_field_ident,
                query,
            );
        };
        let create_or_replace_function_token_stream = {
            let create_or_replace_function_stringified = format!("r#\"create or replace function cats_update_by_id_name_color(cat_name varchar, cat_color varchar, cat_id bigint)
returns void language plpgsql
as $$
begin
    update cats set name = cat_name, color = cat_color where id = cat_id;
    if not found then raise exception 'cat id % not found', cat_id;
    end if;
end $$\"#");
                create_or_replace_function_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_or_replace_function_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            
//             quote::quote!{
// r#"create or replace function cats_update_by_id_name_color(cat_name varchar, cat_color varchar, cat_id bigint)
// returns void language plpgsql
// as $$
// begin
//     update cats set name = cat_name, color = cat_color where id = cat_id;
//     if not found then raise exception 'cat id % not found', cat_id;
//     end if;
// end $$"#
//             }
        };
        let f = quote::quote!{
            impl #update_by_id_parameters_camel_case_token_stream {
                pub async fn #prepare_and_execute_query_token_stream(
                    self,
                    app_info_state: &#app_info_state_path,
                ) -> #prepare_and_execute_query_response_variants_token_stream
                {
                    #check_for_all_none_token_stream
                    if let Err(e) = sqlx::query::<sqlx::Postgres>(
                        #create_or_replace_function_token_stream
// r#"create or replace function cats_update_by_id_name_color(cat_name varchar, cat_color varchar, cat_id bigint)
// returns void language plpgsql
// as $$
// begin
//     update cats set name = cat_name, color = cat_color where id = cat_id;
//     if not found then raise exception 'cat id % not found', cat_id;
//     end if;
// end $$"#//;
                    )
                    .execute(app_info_state.get_postgres_pool())
                    .await
                    {
                        let error = crate :: repositories_types :: tufa_server ::
                            routes :: api :: cats :: update_by_id :: TryUpdateById ::
                            from(e) ;
                        crate::common::error_logs_logic::error_log::ErrorLog::error_log(
                            &error,
                            app_info_state.as_ref(),
                        );
                        return crate :: repositories_types :: tufa_server :: routes :: api ::
                            cats :: update_by_id :: TryUpdateByIdResponseVariants ::
                            from(error);
                    }
                    let query_string = {
                        let mut query = format!(
                            "{} {} {} ",
                            crate::server::postgres::constants::UPDATE_NAME,
                            crate::repositories_types::tufa_server::routes::api::cats::CATS,
                            crate::server::postgres::constants::SET_NAME,
                        );
                        let mut increment: u64 = 0;
                        
                        #(#additional_parameters_modification_token_stream)*
                        #additional_parameters_id_modification_token_stream
                        query
                        // String::from("
                        //     do $$
                        //     begin
                        //         update cats set name = $1, color = $2 where id = $3;
                        //         if not found then raise exception 'id 1 in cats not found';
                        //         end if;
                        //     end $$;
                        // ")
                    };
                    println!("{query_string}");
                    let binded_query = {
                        let mut query = sqlx::query::<sqlx::Postgres>(&query_string);
                        #(#binded_query_modifications_token_stream)*
                        #binded_query_id_modification_token_stream
                        query
                    };
                    match binded_query
                        .execute(app_info_state.get_postgres_pool())
                        .await
                    {
                        Ok(_) => {
                            //todo - is need to return rows affected?
                            #prepare_and_execute_query_response_variants_token_stream::Desirable(())
                        }
                        Err(e) => {
                            let error = #prepare_and_execute_query_error_token_stream::from(e);
                            #error_log_call_token_stream
                            #prepare_and_execute_query_response_variants_token_stream::from(error)
                        }
                    }
                }
            }
        };
        // println!("{f}");
        quote::quote!{
            #[derive(Debug, serde::Deserialize)]
            pub struct #update_by_id_parameters_camel_case_token_stream {
                pub path: #update_by_id_path_camel_case_token_stream,
                pub payload: #update_by_id_payload_camel_case_token_stream,
            }
            #[derive(Debug, serde::Deserialize)]
            pub struct #update_by_id_path_camel_case_token_stream {
                pub #id_field_ident: crate::server::postgres::bigserial::Bigserial,//#id_field_type
            }
            #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
            pub struct #update_by_id_payload_camel_case_token_stream {
                #(#fields_with_excluded_id_token_stream),*
            }
        }
    };
    let update_token_stream = {
        let update_name_camel_case_stringified = "Update";
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
        let id_field_ident = id_field.ident.clone()
            .unwrap_or_else(|| {
                panic!("{proc_macro_name_ident_stringified} id_field.ident is None")
            });
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
        let update_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&update_name_camel_case_stringified);
        let prepare_and_execute_query_response_variants_token_stream = {
            let try_response_variants_path_stringified = format!("{path_to_crud}{update_name_lower_case_stringified}::{try_camel_case_stringified}{update_name_camel_case_stringified}{response_variants_camel_case_stringified}");
            try_response_variants_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_response_variants_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let prepare_and_execute_query_error_token_stream = {
            let error_path_stringified = format!("{path_to_crud}{update_name_lower_case_stringified}::{try_camel_case_stringified}{update_name_camel_case_stringified}");
            error_path_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {error_path_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
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
            let query_stringified = format!("\"{{}} {{}} {{}} t {{}} {declarations} {{}} (values {{values}}) as data({column_names}) where t.{id_field_ident} = data.{id_field_ident}\"");
            query_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let additional_parameters_modification_token_stream = fields_named.iter().map(|field| {
            let field_ident = field.ident.clone()
                .unwrap_or_else(|| {
                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                });
            quote::quote!{
                match crate::server::postgres::bind_query::BindQuery::try_generate_bind_increments(
                    &element.#field_ident,
                    &mut increment
                ) {
                    Ok(value) => {
                        element_value.push_str(&format!("{value}, "));
                    },
                    Err(e) => {
                        return #prepare_and_execute_query_response_variants_token_stream::BindQuery { 
                            checked_add: e.into_serialize_deserialize_version(), 
                            code_occurence: crate::code_occurence_tufa_common!(),
                        };
                    },
                };
            }
        });
        let binded_query_modifications_token_stream = fields_named.iter().map(|field|{
            let field_ident = field.ident.clone()
                .unwrap_or_else(|| {
                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                });
            quote::quote!{
                query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                    element.#field_ident,
                    query,
                ); 
            }
        });
        quote::quote!{
            #[derive(Debug, serde :: Deserialize)]
            pub struct #update_parameters_camel_case_token_stream {
                pub payload: Vec<#update_payload_element_camel_case_token_stream>,
            }
            #[derive(Debug, serde_derive :: Serialize, serde_derive :: Deserialize)]
            pub struct #update_payload_element_camel_case_token_stream {
                pub #id_field_ident: crate::server::postgres::bigserial::Bigserial,
                #(#fields_with_excluded_id_token_stream),*
            }
            impl #update_parameters_camel_case_token_stream {
                pub async fn #prepare_and_execute_query_token_stream(
                    self,
                    app_info_state: &#app_info_state_path,
                ) -> #prepare_and_execute_query_response_variants_token_stream
                {
                    let query_string = {
                        let mut values = std::string::String::default();
                        let mut increment: u64 = 0;
                        for element in &self.payload {
                            let mut element_value = std::string::String::default();
                            #(#additional_parameters_modification_token_stream)*
                            element_value.pop();//todo - remove it 
                            element_value.pop();
                            values.push_str(&format!("({element_value}), "));
                        }
                        values.pop();
                        values.pop();
                        format!(
                            #query_token_stream,
                            crate::server::postgres::constants::UPDATE_NAME,
                            crate::repositories_types::tufa_server::routes::api::cats::CATS,
                            crate::server::postgres::constants::AS_NAME,
                            crate::server::postgres::constants::SET_NAME,
                            crate::server::postgres::constants::FROM_NAME,
                        )
                    };
                    println!("{query_string}");
                    let binded_query = {
                        let mut query = sqlx::query::<sqlx::Postgres>(&query_string);
                        for element in self.payload {
                            #(#binded_query_modifications_token_stream)*
                        }
                        query
                    };
                    match binded_query
                        .execute(app_info_state.get_postgres_pool())
                        .await
                    {
                        Ok(_) => {
                            //todo - is need to return rows affected?
                            #prepare_and_execute_query_response_variants_token_stream::Desirable(())
                        }
                        Err(e) => {
                            let error = #prepare_and_execute_query_error_token_stream::from(e);
                            #error_log_call_token_stream
                            #prepare_and_execute_query_response_variants_token_stream::from(error)
                        }
                    }
                }
            }
        }
    };
    let gen = quote::quote! {
        #struct_options_token_stream
        #from_ident_for_ident_options_token_stream
        #(#structs_variants_token_stream)*
        #(#structs_variants_impl_from_token_stream)*
        #column_token_stream
        #column_select_token_stream

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

fn column_names_factorial(
    original_input: Vec<(usize, &syn::Field)>,
    input: Vec<&syn::Field>,
    output: &mut Vec<Vec<syn::Field>>,
    proc_macro_name_ident_stringified: &std::string::String
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
                                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                        })
                                        == field
                                            .ident
                                            .clone()
                                            .unwrap_or_else(|| {
                                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                            })
                                })
                                .unwrap_or_else(|| {
                                    panic!(
                                        "{proc_macro_name_ident_stringified} cannot find original input index"
                                    )
                                });
                            let (index_b, _) = original_input
                                .iter()
                                .find(|(_, field)| {
                                    b.ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                        })
                                        == field
                                            .ident
                                            .clone()
                                            .unwrap_or_else(|| {
                                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                            })
                                })
                                .unwrap_or_else(|| {
                                    panic!(
                                        "{proc_macro_name_ident_stringified} cannot find original input index"
                                    )
                                });
                            index_a.partial_cmp(index_b).unwrap_or_else(|| {
                                panic!(
                                    "{proc_macro_name_ident_stringified} index_a.partial_cmp(index_b) is None"
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
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                })
                                .to_string()
                                .partial_cmp(
                                    &b_elem
                                        .ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
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
                    panic!("{proc_macro_name_ident_stringified} index_a.partial_cmp(index_b) is None")
                })
            });
            end_out
        }
        // 1 => {
        //     let mut output_handle = vec![];
        //     original_input.iter().for_each(|(_, element)| {
        //         output_handle.push(vec![element.clone()]);
        //     });
        //     let first_element = input.get(0).unwrap_or_else(||panic!("{proc_macro_name_ident_stringified} input.get(0) is None"));
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
                    .unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} input.get(0) is None"));
                let output_len = output.len();
                output.iter_mut().fold(Vec::with_capacity(output_len * 2), |mut acc, out| {
                    if !acc.contains(out) {
                        out.sort_by(|a,b|{
                            let (index_a, _) = original_input.iter().find(|(_, field)|{a.ident
                                .clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                }) == field
                                .ident
                                .clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                })
                            }).unwrap_or_else(||panic!("{proc_macro_name_ident_stringified} cannot find original input index"));
                            let (index_b, _) = original_input.iter().find(|(_, field)|{b.ident
                                .clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                }) == field
                                .ident
                                .clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                })
                            }).unwrap_or_else(||panic!("{proc_macro_name_ident_stringified} cannot find original input index"));
                            index_a.partial_cmp(index_b).unwrap_or_else(||panic!("{proc_macro_name_ident_stringified} index_a.partial_cmp(index_b) is None"))
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
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                }) == field
                                .ident
                                .clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                })
                            }).unwrap_or_else(||panic!("{proc_macro_name_ident_stringified} cannot find original input index"));
                            let (index_b, _) = original_input.iter().find(|(_, field)|{b.ident
                                .clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                }) == field
                                .ident
                                .clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                })
                            }).unwrap_or_else(||panic!("{proc_macro_name_ident_stringified} cannot find original input index"));
                            index_a.partial_cmp(index_b).unwrap_or_else(||panic!("{proc_macro_name_ident_stringified} index_a.partial_cmp(index_b) is None"))
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
            column_names_factorial(original_input, new_input_vec, &mut output_handle, proc_macro_name_ident_stringified)
        }
    }
}

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