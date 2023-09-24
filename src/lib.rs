mod column_names_factorial;
mod check_for_all_none;
mod acquire_pool_and_connection;
mod from_log_and_return_error;

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
    let ident_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&ident.to_string());
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
    let prepare_and_execute_query_name_token_stream = quote::quote!{prepare_and_execute_query};
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
    let pg_temp_stringified = "pg_temp";
    let pg_connection_token_stream = quote::quote!{pg_connection};
    let desirable_token_stream = quote::quote!{Desirable};
    let query_string_name_token_stream = quote::quote!{query_string};
    let function_creation_query_string_name_token_stream = quote::quote!{function_creation_query_string};
    let binded_query_name_token_stream = quote::quote!{binded_query};
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
    let generate_create_or_replace_function_token_stream = |
        operation: Operation
    | -> proc_macro2::TokenStream {
        let create_or_replace_function_name_original_token_stream = {
            let create_or_replace_function_name_original_stringified = format!("\"{ident_lower_case_stringified}{operation}\"");
            create_or_replace_function_name_original_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_or_replace_function_name_original_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let create_or_replace_function_name_additions_token_stream = fields_named.iter().filter_map(|field|match field == &id_field {
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
                    if self.payload.#field_ident.is_some() {
                        value.push_str(&format!(#format_value_token_stream));
                    }
                })
            },
        });
        quote::quote!{
            let mut value = format!(#create_or_replace_function_name_original_token_stream);
            #(#create_or_replace_function_name_additions_token_stream)*
            value
        }
    };
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
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
                pub struct #create_batch_parameters_camel_case_token_stream {
                    pub payload: Vec<#create_batch_payload_element_camel_case_token_stream>,
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
                #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
                pub struct #create_batch_payload_element_camel_case_token_stream {
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let prepare_and_execute_query_token_stream = {
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
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &prepare_and_execute_query_response_variants_token_stream,
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
                quote::quote!{
                    format!(
                        #query_token_stream,
                        #crate_server_postgres_constants_insert_name_token_stream,
                        #crate_server_postgres_constants_into_name_token_stream,
                        crate::repositories_types::tufa_server::routes::api::cats::CATS,
                        #crate_server_postgres_constants_values_name_token_stream,
                        {
                            #increment_initialization_token_stream
                            let mut bind_increments = std::string::String::default();
                            for element in &self.payload {
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
                            query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(element.#field_ident, query);  
                        })
                    },
                });
                quote::quote!{
                    let mut query = sqlx::query::<sqlx::Postgres>(&#query_string_name_token_stream);
                    for element in self.payload {
                        #(#bind_value_to_query_modificate_token_stream)*
                    }
                    query
                }
            };
            quote::quote!{
                impl #create_batch_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        app_info_state: &#app_info_state_path,
                    ) -> #prepare_and_execute_query_response_variants_token_stream
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
                                #prepare_and_execute_query_response_variants_token_stream::#desirable_token_stream(())
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
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #prepare_and_execute_query_token_stream
        }
    };
    // println!("{create_batch_token_stream}");
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
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
                pub struct #create_parameters_camel_case_token_stream {
                    pub payload: #create_payload_camel_case_token_stream,
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
                #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
                pub struct #create_payload_camel_case_token_stream {
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let prepare_and_execute_query_token_stream = {
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
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &prepare_and_execute_query_response_variants_token_stream,
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
                        crate::repositories_types::tufa_server::routes::api::cats::CATS,
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
                            query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(self.payload.#field_ident, query);
                        })
                    },
                });
                quote::quote!{
                    let mut query = sqlx::query::<sqlx::Postgres>(&#query_string_name_token_stream);
                    #(#binded_query_modifications_token_stream)*
                    query
                }
            };
            quote::quote!{
                impl #create_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        app_info_state: &#app_info_state_path,
                    ) -> #prepare_and_execute_query_response_variants_token_stream
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
                                #prepare_and_execute_query_response_variants_token_stream::#desirable_token_stream(())
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
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #prepare_and_execute_query_token_stream
        }
    };
    // println!("{create_token_stream}");
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
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde::Deserialize)]
                pub struct #delete_by_id_parameters_camel_case_token_stream {
                    pub path: #delete_by_id_path_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let path_token_stream = {
            quote::quote!{
                #[derive(Debug, serde::Deserialize)]
                pub struct #delete_by_id_path_camel_case_token_stream {
                    pub #id_field_ident: crate::server::postgres::bigserial::Bigserial,//#id_field_type
                }
            }
        };
        // println!("{path_token_stream}");
        let prepare_and_execute_query_token_stream = {
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
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &prepare_and_execute_query_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
            );
            let query_string_token_stream = {
                let query_token_stream = {
                    let query_stringified = format!("\"{{}} {{}} {{}} {{}} {id_field_ident} = $1\"");
                    query_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                quote::quote!{
                    format!(
                        #query_token_stream,
                        #crate_server_postgres_constants_delete_name_token_stream,
                        #crate_server_postgres_constants_from_name_token_stream,
                        crate::repositories_types::tufa_server::routes::api::cats::CATS,
                        #crate_server_postgres_constants_where_name_token_stream
                    )
                }
            };
            let binded_query_token_stream = {
                let binded_query_modifications_token_stream = quote::quote!{
                    query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(self.path.#id_field_ident, query);
                };
                quote::quote!{
                    let mut query = sqlx::query::<sqlx::Postgres>(&#query_string_name_token_stream);
                    #binded_query_modifications_token_stream
                    query
                }
            };
            quote::quote!{
                impl #delete_by_id_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        app_info_state: &#app_info_state_path,
                    ) -> #prepare_and_execute_query_response_variants_token_stream
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
                                #prepare_and_execute_query_response_variants_token_stream::#desirable_token_stream(())
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
        quote::quote!{
            #parameters_token_stream
            #path_token_stream
            #prepare_and_execute_query_token_stream
        }
    };
    // println!("{delete_by_id_token_stream}");
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
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde::Deserialize)]
                pub struct #delete_with_body_parameters_camel_case_token_stream {
                    pub payload: #delete_with_body_payload_camel_case_token_stream,
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
                        pub #field_ident: Option<Vec<crate::server::postgres::regex_filter::RegexFilter>>
                    })
                },
            });
            quote::quote!{
                #[derive(Debug, serde::Serialize, serde::Deserialize)]
                pub struct #delete_with_body_payload_camel_case_token_stream {
                    pub #id_field_ident: Option<Vec<crate::server::postgres::bigserial::Bigserial>>,
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let prepare_and_execute_query_token_stream = {
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
            let check_for_all_none_token_stream = crate::check_for_all_none::check_for_all_none(
                &fields_named,
                &id_field,
                &proc_macro_name_ident_stringified,
                dot_space,
                &prepare_and_execute_query_response_variants_token_stream,
                crate::check_for_all_none::QueryPart::Payload
            );
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &prepare_and_execute_query_response_variants_token_stream,
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
                        if let Some(value) = &self.payload.#id_field_ident {
                            let prefix = match additional_parameters.is_empty() {
                                true => #crate_server_postgres_constants_where_name_token_stream.to_string(),
                                false => format!(" {}", #crate_server_postgres_constants_and_name_token_stream),
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
                            let handle_stringified = format!("\"{field_ident} ~ {{bind_increments_handle}} \"");
                            handle_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        Some(quote::quote!{
                            if let Some(value) = &self.payload.#field_ident {
                                let prefix = match additional_parameters.is_empty() {
                                    true => #crate_server_postgres_constants_where_name_token_stream.to_string(),
                                    false => format!(" {}", #crate_server_postgres_constants_and_name_token_stream),
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
                quote::quote!{
                    format!(
                        "{} {} {} {}",
                        #crate_server_postgres_constants_delete_name_token_stream,
                        #crate_server_postgres_constants_from_name_token_stream,
                        crate::repositories_types::tufa_server::routes::api::cats::CATS,
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
                    let mut query = sqlx::query::<sqlx::Postgres>(&#query_string_name_token_stream);
                    #(#binded_query_modifications_token_stream)*
                    query
                }
            };
            quote::quote!{
                impl #delete_with_body_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        app_info_state: &#app_info_state_path,
                    ) -> #prepare_and_execute_query_response_variants_token_stream
                    {
                        #check_for_all_none_token_stream
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
                                #prepare_and_execute_query_response_variants_token_stream::#desirable_token_stream(())
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
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #prepare_and_execute_query_token_stream
        }
    };
    // println!("{delete_with_body_token_stream}");
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
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde::Deserialize)]
                pub struct #delete_parameters_camel_case_token_stream {
                    pub query: #delete_query_camel_case_token_stream,
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
                #[derive(Debug, serde::Serialize, serde::Deserialize)]
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
            quote::quote!{
                impl #delete_query_camel_case_token_stream {
                    fn into_url_encoding_version(self) -> #delete_query_for_url_encoding_camel_case_token_stream {
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
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &prepare_and_execute_query_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
            );
            let check_for_all_none_token_stream = crate::check_for_all_none::check_for_all_none(
                &fields_named,
                &id_field,
                &proc_macro_name_ident_stringified,
                dot_space,
                &prepare_and_execute_query_response_variants_token_stream,
                crate::check_for_all_none::QueryPart::QueryParameters
            );
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
                quote::quote!{
                    format!(
                        "{} {} {} {} {}",
                        #crate_server_postgres_constants_delete_name_token_stream,
                        #crate_server_postgres_constants_from_name_token_stream,
                        crate::repositories_types::tufa_server::routes::api::cats::CATS,
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
            let binded_query_token_stream = {
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
                    let mut query = sqlx::query::<sqlx::Postgres>(&#query_string_name_token_stream);
                    #(#binded_query_modifications_token_stream)*
                    query
                }
            };
            quote::quote!{
                impl #delete_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        app_info_state: &#app_info_state_path,
                    ) -> #prepare_and_execute_query_response_variants_token_stream
                    {
                        #check_for_all_none_token_stream
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
                                #prepare_and_execute_query_response_variants_token_stream::#desirable_token_stream(())
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
        quote::quote!{
            #parameters_token_stream
            #query_token_stream
            #query_for_url_encoding_token_stream
            #into_url_encoding_version_token_stream
            #prepare_and_execute_query_token_stream 
        }
    };
    // println!("{delete_token_stream}");
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
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde::Deserialize)]
                pub struct #read_by_id_parameters_camel_case_token_stream {
                    pub path: #read_by_id_path_camel_case_token_stream,
                    pub query: #read_by_id_query_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let path_token_stream = {
            quote::quote!{
                #[derive(Debug, serde::Deserialize)]
                pub struct #read_by_id_path_camel_case_token_stream {
                    pub #id_field_ident: crate::server::postgres::bigserial::Bigserial//#id_field_type,
                }
            }
        };
        // println!("{path_token_stream}");
        let query_token_stream = {
            quote::quote!{
                #[derive(Debug, serde::Serialize, serde::Deserialize)]
                pub struct #read_by_id_query_camel_case_token_stream {
                    pub select: Option<#column_select_ident_token_stream>,
                }
            }
        };
        // println!("{query_token_stream}");
        let query_for_url_encoding_token_stream = {
            quote::quote!{
                #[derive(Debug, serde::Serialize, serde::Deserialize)]
                struct #read_by_id_query_for_url_encoding_camel_case_token_stream {
                    select: Option<std::string::String>,
                } 
            }
        };
        // println!("{query_for_url_encoding_token_stream}");
        let into_url_encoding_version_token_stream = {
            quote::quote!{
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
            }
        };
        // println!("{into_url_encoding_version_token_stream}");
        let prepare_and_execute_query_token_stream = {
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
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &prepare_and_execute_query_response_variants_token_stream,
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
                        crate::server::postgres::generate_query::GenerateQuery::generate_query(&select),
                        #crate_server_postgres_constants_from_name_token_stream,
                        crate::repositories_types::tufa_server::routes::api::cats::CATS,
                        #crate_server_postgres_constants_where_name_token_stream,
                    )
                }
            };
            let binded_query_token_stream = {
                let binded_query_modifications_token_stream = quote::quote!{
                    query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                        self.path.#id_field_ident, query,
                    );
                };
                quote::quote!{
                    let mut query = sqlx::query::<sqlx::Postgres>(&#query_string_name_token_stream);
                    #binded_query_modifications_token_stream
                    query
                }
            };
            quote::quote!{
                impl #read_by_id_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        app_info_state: &#app_info_state_path,
                    ) -> #prepare_and_execute_query_response_variants_token_stream
                    {
                        let select = self.query.select.unwrap_or_default();
                        let #query_string_name_token_stream = #query_string_token_stream;
                        // println!("{query_string}");
                        let #binded_query_name_token_stream = {
                            #binded_query_token_stream
                        };
                        #acquire_pool_and_connection_token_stream
                        match #binded_query_name_token_stream.fetch_one(#pg_connection_token_stream.as_mut()).await {
                            Ok(row) => match select.options_try_from_sqlx_row(&row) {
                                Ok(value) => #prepare_and_execute_query_response_variants_token_stream::#desirable_token_stream(value),
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
        quote::quote!{
            #parameters_token_stream
            #path_token_stream
            #query_token_stream
            #query_for_url_encoding_token_stream
            #into_url_encoding_version_token_stream
            #prepare_and_execute_query_token_stream
        }
    };
    // println!("{read_by_id_token_stream}");
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
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
                pub struct #read_with_body_parameters_camel_case_token_stream {
                    pub payload: #read_with_body_payload_camel_case_token_stream,
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
                #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
                pub struct #read_with_body_payload_camel_case_token_stream {
                    pub select: #column_select_ident_token_stream,
                    pub #id_field_ident: Option<Vec<crate::server::postgres::bigserial::Bigserial>>,
                    #(#fields_with_excluded_id_token_stream)*
                    pub order_by: crate::server::postgres::order_by::OrderBy<CatColumn>,
                    pub limit: crate::server::postgres::postgres_bigint::PostgresBigint,
                    pub offset: crate::server::postgres::postgres_bigint::PostgresBigint,
                }
            }
        };
        // println!("{payload_token_stream}");
        let prepare_and_execute_query_token_stream = {
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
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &prepare_and_execute_query_response_variants_token_stream,
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
                        if let Some(value) = &self.payload.#id_field_ident {
                            let prefix = match additional_parameters.is_empty() {
                                true => #crate_server_postgres_constants_where_name_token_stream.to_string(),
                                false => format!(" {}", #crate_server_postgres_constants_and_name_token_stream),
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
                            let handle_stringified = format!("\"{field_ident} ~ {{bind_increments_handle}} \"");
                            handle_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        Some(quote::quote!{
                            if let Some(value) = &self.payload.#field_ident {
                                let prefix = match additional_parameters.is_empty() {
                                    true => #crate_server_postgres_constants_where_name_token_stream.to_string(),
                                    false => format!(" {}", #crate_server_postgres_constants_and_name_token_stream),
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
                quote::quote!{
                    format!(
                        "{} {} {} {} {}",
                        #crate_server_postgres_constants_select_name_token_stream,
                        crate::server::postgres::generate_query::GenerateQuery::generate_query(
                            &self.payload.select
                        ),
                        #crate_server_postgres_constants_from_name_token_stream,
                        crate::repositories_types::tufa_server::routes::api::cats::CATS,
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
                                let value = &self.payload.order_by;
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
                                    #crate_server_postgres_constants_limit_name_token_stream,
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
                    let mut query = sqlx::query::<sqlx::Postgres>(&#query_string_name_token_stream);
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
                }
            };
            quote::quote!{
                impl #read_with_body_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        app_info_state: &#app_info_state_path,
                    ) -> #prepare_and_execute_query_response_variants_token_stream
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
                                    Ok(option_pg_row) => option_pg_row,
                                    Err(e) => {
                                        #from_log_and_return_error_token_stream;
                                    }
                                }
                            } {
                                match self.payload.select.options_try_from_sqlx_row(&row) {
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
                        #prepare_and_execute_query_response_variants_token_stream::#desirable_token_stream(vec_values)
                    }
                }
            }
        };
        // println!("{prepare_and_execute_query_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #prepare_and_execute_query_token_stream
        }
    };
    // println!("{read_with_body_token_stream}");
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
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde::Deserialize)]
                pub struct #read_parameters_camel_case_token_stream {
                    pub query: #read_query_camel_case_token_stream,
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
                #[derive(Debug, serde::Serialize, serde::Deserialize)]
                pub struct #read_query_camel_case_token_stream {
                    pub select: Option<#column_select_ident_token_stream>,
                    pub #id_field_ident: Option<crate::server::postgres::bigserial_ids::BigserialIds>,
                    #(#fields_with_excluded_id_token_stream)*
                    pub order_by: Option<CatOrderByWrapper>,//todo
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
                    select: Option<std::string::String>,
                    pub #id_field_ident: Option<std::string::String>,
                    #(#fields_for_url_encoding_with_excluded_id_token_stream)*
                    order_by: Option<std::string::String>,
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
            quote::quote!{
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
            }
        };
        // println!("{into_url_encoding_version_token_stream}");
        let prepare_and_execute_query_token_stream = {
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
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &prepare_and_execute_query_response_variants_token_stream,
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
                        if let Some(value) = &self.query.#field_ident {
                            let prefix = match additional_parameters.is_empty() {
                                true => #crate_server_postgres_constants_where_name_token_stream.to_string(),
                                false => format!(" {}", #crate_server_postgres_constants_and_name_token_stream),
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
                        crate::server::postgres::generate_query::GenerateQuery::generate_query(&select),
                        #crate_server_postgres_constants_from_name_token_stream,
                        crate::repositories_types::tufa_server::routes::api::cats::CATS,
                        {
                            #increment_initialization_token_stream
                            let mut additional_parameters = std::string::String::default();
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
                                    #crate_server_postgres_constants_order_by_name_token_stream,
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
                                    #crate_server_postgres_constants_limit_name_token_stream,
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
                        if let Some(value) = self.query.#field_ident {
                            query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                                value, query,
                            );
                        }
                    }
                });
                quote::quote!{
                    let mut query = sqlx::query::<sqlx::Postgres>(&#query_string_name_token_stream);
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
                }
            };
            quote::quote!{
                impl #read_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self, //impl crate::server::routes::helpers::bind_sqlx_query::BindSqlxQuer + crate::server::postgres::generate_query::GenerateQuery
                        app_info_state: &#app_info_state_path,
                    ) -> #prepare_and_execute_query_response_variants_token_stream
                    {
                        let select = #column_select_ident_token_stream::from(self.query.select.clone());
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
                                    Ok(option_pg_row) => option_pg_row,
                                    Err(e) => {
                                        #from_log_and_return_error_token_stream;
                                    }
                                }
                            } {
                                match select.options_try_from_sqlx_row(&row) {
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
                        #prepare_and_execute_query_response_variants_token_stream::#desirable_token_stream(vec_values)
                    }
                }
            }
        };
        // println!("{prepare_and_execute_query_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #query_token_stream
            #query_for_url_encoding_token_stream
            #into_url_encoding_version_token_stream
            #prepare_and_execute_query_token_stream
        }
    };
    // println!("{read_token_stream}");
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
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde::Deserialize)]
                pub struct #update_by_id_parameters_camel_case_token_stream {
                    pub path: #update_by_id_path_camel_case_token_stream,
                    pub payload: #update_by_id_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let path_token_stream = {
            quote::quote!{
                #[derive(Debug, serde::Deserialize)]
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
                #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
                pub struct #update_by_id_payload_camel_case_token_stream {
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        //
        //
        // println!("{payload_token_stream}");
        let prepare_and_execute_query_token_stream = {
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
            let check_for_all_none_token_stream = crate::check_for_all_none::check_for_all_none(
                &fields_named,
                &id_field,
                &proc_macro_name_ident_stringified,
                dot_space,
                &prepare_and_execute_query_response_variants_token_stream,
                crate::check_for_all_none::QueryPart::Payload
            );
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &prepare_and_execute_query_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
            );
            let update_by_id_stringified = "_update_by_id";
            let create_or_replace_function_token_stream = {
                let create_or_replace_function_name_token_stream = generate_create_or_replace_function_token_stream(Operation::UpdateById);
                println!("{create_or_replace_function_name_token_stream}");
                println!("-----------");
                let create_or_replace_function_parameters_token_stream = {
                    let create_or_replace_function_parameters_original_token_stream = {
                        let create_or_replace_function_parameters_stringified = format!("\"{ident_lower_case_stringified}_{id_field_ident} bigint, \"");//todo postgresql type
                        create_or_replace_function_parameters_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {create_or_replace_function_parameters_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    let create_or_replace_function_parameters_additions_token_stream = {
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
                                let handle_stringified = format!("\"{ident_lower_case_stringified}_{field_ident} varchar{possible_dot_space}\"");//todo postgresql type attribute
                                handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            quote::quote!{
                                if self.payload.#field_ident.is_some() {
                                    value.push_str(&format!(#handle_token_stream));
                                }
                            }
                        }).collect::<Vec<proc_macro2::TokenStream>>()
                    };
                    quote::quote!{
                        let mut value = std::string::String::from(#create_or_replace_function_parameters_original_token_stream);//format!("cats_id bigint, cats_name varchar, cats_color varchar");
                        #(#create_or_replace_function_parameters_additions_token_stream)*
                        value
                    }
                };
                let create_or_replace_function_additional_parameters_modification_token_stream = {
                    let fields_named_filtered = fields_named.iter().filter(|field|*field != &id_field).collect::<Vec<&syn::Field>>();
                    let fields_named_len = fields_named_filtered.len();
                    fields_named_filtered.iter().enumerate().map(|(index, field)| {
                        let field_ident = field.ident.clone()
                            .unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                        let handle_token_stream = {
                            let possible_dot_space = match (index + 1) == fields_named_len {
                                true => " ",
                                false => dot_space,
                            };

                            let handle_stringified = format!("\"{field_ident} = {ident_lower_case_stringified}_{field_ident}{possible_dot_space}\"");//todo postgresql type attribute
                            handle_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{
                            if self.payload.#field_ident.is_some() {
                                value.push_str(&format!(#handle_token_stream));
                            }
                        }
                    }).collect::<Vec<proc_macro2::TokenStream>>()
                };
                let create_or_replace_function_additional_parameters_id_modification_token_stream = {
                    let handle_token_stream = {
                        let handle_stringified = format!("\"{{}} {id_field_ident} = {ident_lower_case_stringified}_{id_field_ident}\"");//todo postgresql type
                        handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    quote::quote!{
                        value.push_str(&format!(
                            #handle_token_stream,
                            #crate_server_postgres_constants_where_name_token_stream
                        ));
                    }
                };
                quote::quote!{
                    let create_or_replace_function_name = {
                        #create_or_replace_function_name_token_stream
                    };
                    let create_or_replace_function_parameters = {
                        #create_or_replace_function_parameters_token_stream
                    };
                    let create_or_replace_function_first_line_query = {
                        let mut value = format!(
                            "{} {} {} ",
                            #crate_server_postgres_constants_update_name_token_stream,
                            crate::repositories_types::tufa_server::routes::api::cats::CATS,
                            #crate_server_postgres_constants_set_name_token_stream,
                        );
                        #(#create_or_replace_function_additional_parameters_modification_token_stream)*
                        #create_or_replace_function_additional_parameters_id_modification_token_stream
                        value
                    };
                    let create_or_replace_function_second_line_query = std::string::String::from("if not found then raise exception 'cats id % not found', cats_id");
                    let create_or_replace_function_third_line_query = std::string::String::from("end if");
                    format!("create or replace function pg_temp.{create_or_replace_function_name}({create_or_replace_function_parameters}) returns void language plpgsql as $$ begin {create_or_replace_function_first_line_query};{create_or_replace_function_second_line_query};{create_or_replace_function_third_line_query};end $$")
                }
            };
            let query_string_token_stream = {
                let function_name_token_stream = {
                    let create_or_replace_function_token_stream = generate_create_or_replace_function_token_stream(Operation::UpdateById);
                    let function_name_handle_token_stream = {
                        let handle_stringified = format!("\"{ident_lower_case_stringified}{update_by_id_stringified}{{}}\"");//{{additional_function_name}}
                        handle_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    quote::quote!{
                        let additional_function_name = {
                            #create_or_replace_function_token_stream
                        };
                        format!(
                            #function_name_handle_token_stream,
                            additional_function_name
                        )
                    }
                };
                println!("{function_name_token_stream}");
                let query_token_stream = {
                    let query_stringified = format!("\"{{}} {pg_temp_stringified}.{{function_name}}({{}})\"");
                    query_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let query_parameters_token_stream = {
                    let additional_parameters_id_modification_token_stream = {
                        let query_part_token_stream = {
                            let query_part_stringified = format!("\"{ident_lower_case_stringified}_{id_field_ident} => ${{increment}}{dot_space}\"");
                            query_part_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_part_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{
                            match crate::server::postgres::bind_query::BindQuery::try_increment(&self.path.#id_field_ident, &mut increment) {
                                Ok(_) => {
                                    query.push_str(&format!(#query_part_token_stream));
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
                                let handle_stringified = format!("\"{ident_lower_case_stringified}_{field_ident} => ${{increment}}{possible_dot_space}\"");
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
                    quote::quote!{
                        #increment_initialization_token_stream
                        let mut query = std::string::String::from("");
                        #additional_parameters_id_modification_token_stream
                        #(#additional_parameters_modification_token_stream)*
                        query
                    }
                };
                quote::quote!{
                    let function_name = {
                        #function_name_token_stream
                    };
                    format!(
                        #query_token_stream,
                        #crate_server_postgres_constants_select_name_token_stream,
                        {
                            #query_parameters_token_stream
                        }
                    )
                }
            };
            let binded_query_token_stream = {
                let binded_query_id_modification_token_stream = quote::quote!{
                    query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                        self.path.#id_field_ident,
                        query,
                    );
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
                quote::quote!{
                    let mut query = sqlx::query::<sqlx::Postgres>(&#query_string_name_token_stream);
                    #binded_query_id_modification_token_stream
                    #(#binded_query_modifications_token_stream)*
                    query
                }
            };
            quote::quote!{
                impl #update_by_id_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        app_info_state: &#app_info_state_path,
                    ) -> #prepare_and_execute_query_response_variants_token_stream
                    {
                        #check_for_all_none_token_stream
                        let #function_creation_query_string_name_token_stream = {
                            #create_or_replace_function_token_stream
                        };
                        // println!("{function_creation_query_string}");
                        let #query_string_name_token_stream = {
                            #query_string_token_stream
                        };
                        // println!("{query_string}");
                        let #binded_query_name_token_stream = {
                            #binded_query_token_stream
                        };
                        #acquire_pool_and_connection_token_stream
                        //todo - maybe add transaction here? 
                        if let Err(e) = sqlx::query::<sqlx::Postgres>(&#function_creation_query_string_name_token_stream)
                            .execute(#pg_connection_token_stream.as_mut())
                            .await {
                            #from_log_and_return_error_token_stream;
                        }
                        match #binded_query_name_token_stream
                            .execute(#pg_connection_token_stream.as_mut())
                            .await
                        {
                            Ok(_) => {
                                //todo - is need to return rows affected?
                                #prepare_and_execute_query_response_variants_token_stream::#desirable_token_stream(())
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
        quote::quote!{
            #parameters_token_stream
            #path_token_stream
            #payload_token_stream
            #prepare_and_execute_query_token_stream
        }
    };
    // println!("{update_by_id_token_stream}");
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
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug, serde :: Deserialize)]
                pub struct #update_parameters_camel_case_token_stream {
                    pub payload: Vec<#update_payload_element_camel_case_token_stream>,
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
                #[derive(Debug, serde_derive :: Serialize, serde_derive :: Deserialize)]
                pub struct #update_payload_element_camel_case_token_stream {
                    pub #id_field_ident: crate::server::postgres::bigserial::Bigserial,
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let prepare_and_execute_query_token_stream = {
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
            let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                &prepare_and_execute_query_error_token_stream,
                &error_log_call_token_stream,
                &prepare_and_execute_query_response_variants_token_stream,
            );
            let acquire_pool_and_connection_token_stream = crate::acquire_pool_and_connection::acquire_pool_and_connection(
                &from_log_and_return_error_token_stream,
                &pg_connection_token_stream
            );
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
                    let query_stringified = format!("\"{{}} {{}} {{}} t {{}} {declarations} {{}} (values {{values}}) as data({column_names}) where t.{id_field_ident} = data.{id_field_ident}\"");
                    query_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                quote::quote!{
                    #increment_initialization_token_stream
                    let mut values = std::string::String::default();
                    for element in &self.payload {
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
                        crate::repositories_types::tufa_server::routes::api::cats::CATS,
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
                        query = crate::server::postgres::bind_query::BindQuery::bind_value_to_query(
                            element.#field_ident,
                            query,
                        ); 
                    }
                });
                quote::quote!{
                    let mut query = sqlx::query::<sqlx::Postgres>(&#query_string_name_token_stream);
                    for element in self.payload {
                        #(#binded_query_modifications_token_stream)*
                    }
                    query
                }
            };
            quote::quote!{
                impl #update_parameters_camel_case_token_stream {
                    pub async fn #prepare_and_execute_query_name_token_stream(
                        self,
                        app_info_state: &#app_info_state_path,
                    ) -> #prepare_and_execute_query_response_variants_token_stream
                    {
                        let #query_string_name_token_stream = {
                            #query_string_token_stream
                        };
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
                                #prepare_and_execute_query_response_variants_token_stream::#desirable_token_stream(())
                            }
                            Err(e) => {
                                #from_log_and_return_error_token_stream;
                            }
                        }
                    }
                }
            }
        };
        // println!("{prepare_and_execute_query_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #prepare_and_execute_query_token_stream
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
    CreateBatch,
    Create,
    DeleteById,
    DeleteWithBody,
    Delete,
    ReadById,
    ReadWithBody,
    Read,
    UpdateById,
    Update
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CreateBatch => write!(f, "create_batch"),
            Self::Create => write!(f, "create"),
            Self::DeleteById => write!(f, "delete_by_id"),
            Self::DeleteWithBody => write!(f, "delete_with_body"),
            Self::Delete => write!(f, "delete"),
            Self::ReadById => write!(f, "read_by_id"),
            Self::ReadWithBody => write!(f, "read_with_body"),
            Self::Read => write!(f, "read"),
            Self::UpdateById => write!(f, "update_by_id"),
            Self::Update => write!(f, "update"),
        }
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