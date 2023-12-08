mod column_names_factorial;
mod check_for_none;
mod acquire_pool_and_connection;
mod from_log_and_return_error;
mod generate_postgres_transaction;
mod generate_postgres_execute_query;
mod type_variants_from_request_response_generator;
//slavic

// trait Something {
//     fn something();
// }

// impl Something for i8 {
//     fn something() {
//         println!("");
//     }
// }

// impl Something for sqlx::types::Uuid {
//     fn something() {}
// }

// Rust type	Postgres type(s)
// bool	BOOL
// i8	“CHAR”
// i16	SMALLINT, SMALLSERIAL, INT2
// i32	INT, SERIAL, INT4
// i64	BIGINT, BIGSERIAL, INT8
// f32	REAL, FLOAT4
// f64	DOUBLE PRECISION, FLOAT8
// &str, String	VARCHAR, CHAR(N), TEXT, NAME
// &[u8], std::vec::Vec<u8>	BYTEA
// ()	VOID
// PgInterval	INTERVAL
// PgRange<T>	INT8RANGE, INT4RANGE, TSRANGE, TSTZRANGE, DATERANGE, NUMRANGE
// PgMoney	MONEY
// PgLTree	LTREE
// PgLQuery	LQUERY




//todo fix bug with body { Desirable: null } if there is not body by idea in response
//todo generate for each create update delete body length checked and for path query headers too
//todo how to write filter logic for sqlx rust postgresql types?
//todo decide where to do error log (maybe add in some places)
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

        generate_postgresql_crud_bool,
        generate_postgresql_crud_char,
        generate_postgresql_crud_smallint,
        generate_postgresql_crud_smallserial,
        generate_postgresql_crud_int2,
        generate_postgresql_crud_int,
        generate_postgresql_crud_serial, 
        generate_postgresql_crud_int4,
        generate_postgresql_crud_bigint,
        generate_postgresql_crud_bigserial, 
        generate_postgresql_crud_int8,
        generate_postgresql_crud_real, 
        generate_postgresql_crud_float4,
        generate_postgresql_crud_double_precision,
        generate_postgresql_crud_float8,
        generate_postgresql_crud_varchar,
        generate_postgresql_crud_charn, //wtf????
        generate_postgresql_crud_text,
        generate_postgresql_crud_name,
        generate_postgresql_crud_bytea,
        generate_postgresql_crud_void,
        generate_postgresql_crud_interval,
        generate_postgresql_crud_int8range,
        generate_postgresql_crud_int4range,
        generate_postgresql_crud_tsrange,
        generate_postgresql_crud_tstzrange,
        generate_postgresql_crud_daterange,
        generate_postgresql_crud_numrange,
        generate_postgresql_crud_money,
        generate_postgresql_crud_ltree,
        generate_postgresql_crud_lquery,

        generate_postgresql_crud_numeric,

        generate_postgresql_crud_timestamptz,
        generate_postgresql_crud_timestamp,
        generate_postgresql_crud_date,
        generate_postgresql_crud_time,
        generate_postgresql_crud_timetz,
        generate_postgresql_crud_uuid,

        generate_postgresql_crud_inet,
        generate_postgresql_crud_cidr,

        generate_postgresql_crud_macaddr,

        generate_postgresql_crud_bit,
        generate_postgresql_crud_varbit,

        generate_postgresql_crud_json,
        generate_postgresql_crud_jsonb
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
    let response_variants_camel_case_stringified = "ResponseVariants";
    // let ident_response_variants_stringified = format!("{ident}{response_variants_camel_case_stringified}");
    // let ident_response_variants_token_stream = {
    //     ident_response_variants_stringified.parse::<proc_macro2::TokenStream>()
    //     .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ident_response_variants_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    // };
    let with_serialize_deserialize_camel_case_stringified = "WithSerializeDeserialize";
    let table_name_stringified = pluralizer::pluralize(&ident_lower_case_stringified, 2, false);
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
    let sqlx_types_uuid_stringified = "sqlx::types::Uuid";
    let sqlx_types_uuid_token_stream = {
        sqlx_types_uuid_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {sqlx_types_uuid_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    {
        let id_field_ident_handle = quote::quote!{#id_field_type}.to_string().replace(" ", "");
        if let false = id_field_ident_handle == sqlx_types_uuid_stringified {
            panic!("{proc_macro_name_ident_stringified} primary_key is not type {sqlx_types_uuid_stringified}");
        }
    }
    //
    let id_field_ident = id_field.ident.clone()
        .unwrap_or_else(|| {
            panic!("{proc_macro_name_ident_stringified} id_field.ident is None")
        });
    let std_string_string_token_stream = quote::quote!{std::string::String};
    let fields_named_wrappers_excluding_primary_key = fields_named.clone().into_iter().filter(|field|*field != id_field).map(|element|{
        let field_ident = element.ident
            .clone()
            .unwrap_or_else(|| {
                panic!("{proc_macro_name_ident_stringified} field.ident is None")
            });
        let attrs = &element.attrs;
        let (
            supported_attribute_type,
            supported_field_type
        ) = match attrs.iter().fold(None, |mut acc, element| {
            let generated_path = proc_macro_helpers::error_occurence::generate_path_from_segments::generate_path_from_segments(&element.path.segments);
            match {
                use std::str::FromStr;
                SupportedAttributeType::from_str(&generated_path)
            } {
                Ok(value) => match acc {
                    Some(acc_value) => panic!("{proc_macro_name_ident_stringified} supported only one attribute per field, detected both: {acc_value} and {value}"),
                    None => {
                        acc = Some(value);
                    }
                },
                Err(e) => panic!("{proc_macro_name_ident_stringified} SupportedAttributeType::from_str {generated_path} error: {e}")
            }
            acc
        }) {
            Some(supported_attribute_type) => {
                let ty = &element.ty;
                let ty_stringified = quote::quote!{#ty}.to_string().replace(" ", "");
                let supported_field_type = {
                    use std::str::FromStr;
                    SupportedFieldType::from_str(ty_stringified.as_str()).unwrap_or_else(|_| panic!(
                        "{proc_macro_name_ident_stringified} {ty_stringified} SupportedFieldType::try_from failed. supported: {:?}", 
                        SupportedFieldType::into_array().into_iter().map(|element|element.to_string()).collect::<std::vec::Vec<std::string::String>>()
                    ))
                };
                match try_match_supported_attribute_type_with_supported_field_type(&supported_attribute_type, &supported_field_type) {
                    true => (
                        supported_attribute_type,
                        supported_field_type
                    ),
                    false => panic!(
                        "{proc_macro_name_ident_stringified} supported_attribute_type {supported_attribute_type} is not matching to supported_field_type {supported_field_type}, see https://docs.rs/sqlx-postgres/0.7.2/sqlx_postgres/types/index.html", 
                    )
                }
            }
            None => panic!(
                "{proc_macro_name_ident_stringified} no field attribute found for {field_ident}, supported: {:?}", 
                SupportedAttributeType::into_array().into_iter().map(|element|element.to_string()).collect::<std::vec::Vec<std::string::String>>()
            )
        };
        FieldNamedWrapperExcludingPrimaryKey {
            field: element,
            supported_attribute_type,
            supported_field_type
        }
    }).collect::<std::vec::Vec<FieldNamedWrapperExcludingPrimaryKey>>();
    let fields_named_len = fields_named.len();
    let fields_named_wrappers_excluding_primary_key_len = fields_named_wrappers_excluding_primary_key.len();
    let id_field_ident_quotes_token_stream = {
        let id_field_ident_quotes_stringified = format!("\"{id_field_ident}\"");
        id_field_ident_quotes_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {id_field_ident_quotes_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
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
    // let derive_debug_deserialize_token_stream = quote::quote!{#[derive(Debug, serde::Deserialize)]};
    let derive_debug_serialize_deserialize_token_stream = quote::quote!{#[derive(Debug, serde::Serialize, serde::Deserialize)]};
    let try_camel_case_stringified = "Try";
    let from_camel_case_stringified = "From";
    let try_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&try_camel_case_stringified.to_string());
    let try_from_camel_case_stringified = format!("{try_camel_case_stringified}{from_camel_case_stringified}");
    let from_str_camel_case_stringified = format!("{from_camel_case_stringified}Str");
    let from_str_camel_case_token_stream = {
        from_str_camel_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {from_str_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let from_str_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&from_str_camel_case_stringified.to_string());
    let from_str_lower_case_token_stream = {
        from_str_lower_case_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {from_str_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let sqlx_row_token_stream = quote::quote!{sqlx::Row};
    let std_primitive_str_sqlx_column_index_token_stream = quote::quote!{&'a std::primitive::str: sqlx::ColumnIndex<R>,};
    let sqlx_decode_decode_database_token_stream = quote::quote!{sqlx::decode::Decode<'a, R::Database>};
    let sqlx_types_type_database_token_stream = quote::quote!{sqlx::types::Type<R::Database>};
    let primary_key_uuid_wrapper_try_from_sqlx_row_name_token_stream = quote::quote!{primary_key_uuid_wrapper_try_from_sqlx_row};
    let crate_server_postgres_uuid_wrapper_token_stream = quote::quote!{crate::server::postgres::uuid_wrapper};
    let error_named_camel_case_stringified = "ErrorNamed";
    let uuid_wrapper_try_from_possible_uuid_wrapper_camel_case_stringified = format!("UuidWrapper{try_from_camel_case_stringified}PossibleUuidWrapper");
    let uuid_wrapper_try_from_possible_uuid_wrapper_camel_case_token_stream = {
        uuid_wrapper_try_from_possible_uuid_wrapper_camel_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {uuid_wrapper_try_from_possible_uuid_wrapper_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let uuid_wrapper_try_from_possible_uuid_wrapper_lower_case_token_stream = {
        let uuid_wrapper_try_from_possible_uuid_wrapper_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(
            &uuid_wrapper_try_from_possible_uuid_wrapper_camel_case_stringified.to_string()
        );
        uuid_wrapper_try_from_possible_uuid_wrapper_lower_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {uuid_wrapper_try_from_possible_uuid_wrapper_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let uuid_wrapper_try_from_possible_uuid_wrapper_error_named_camel_case_token_stream = {
        let uuid_wrapper_try_from_possible_uuid_wrapper_error_named_camel_case_stringified = format!("{uuid_wrapper_try_from_possible_uuid_wrapper_camel_case_stringified}{error_named_camel_case_stringified}");
        uuid_wrapper_try_from_possible_uuid_wrapper_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {uuid_wrapper_try_from_possible_uuid_wrapper_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_uuid_wrapper_uuid_wrapper_try_from_possible_uuid_wrapper_error_named_token_stream = quote::quote!{#crate_server_postgres_uuid_wrapper_token_stream::#uuid_wrapper_try_from_possible_uuid_wrapper_error_named_camel_case_token_stream};
    let crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream = quote::quote!{#crate_server_postgres_uuid_wrapper_token_stream::UuidWrapper};
    let crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream = quote::quote!{#crate_server_postgres_uuid_wrapper_token_stream::PossibleUuidWrapper};
    let std_str_from_str_token_stream = quote::quote!{std::str::#from_str_camel_case_token_stream};
    let struct_options_ident_stringified = format!("{ident}Options");
    let struct_options_ident_token_stream = {
        struct_options_ident_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {struct_options_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let struct_options_token_stream = {
        let field_option_id_token_stream = quote::quote!{
            #[serde(skip_serializing_if = "Option::is_none")]
            pub #id_field_ident: std::option::Option<#crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream>
        };
        let fields_options_excluding_id_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element| {
            let field_vis = &element.field.vis;
            let field_ident = &element.field.ident;
            let field_type_path = &element.field.ty;
            quote::quote!{
                #[serde(skip_serializing_if = "Option::is_none")]
                #field_vis #field_ident: std::option::Option<#field_type_path>
            }
        });
        quote::quote! {
            #derive_debug_serialize_deserialize_token_stream
            pub struct #struct_options_ident_token_stream {
                #field_option_id_token_stream,
                #(#fields_options_excluding_id_token_stream),*
            }
        }
    };
    let from_ident_for_ident_options_token_stream = {
        let ident_option_variant_id_token_stream = quote::quote!{
            #id_field_ident: Some(value.#id_field_ident.into()),
        };
        let ident_option_variants_excluding_id_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element| {
            let field_ident = element.field.ident
                .clone()
                .unwrap_or_else(|| {
                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                });
            quote::quote!{
                #field_ident: Some(value.#field_ident.into())//todo what if type does not implement serialize deserialize
            }
        });
        quote::quote! {
            impl std::convert::From<#ident> for #struct_options_ident_token_stream {
                fn from(value: #ident) -> Self {
                    Self {                        
                        #ident_option_variant_id_token_stream
                        #(#ident_option_variants_excluding_id_token_stream),*
                    }
                }
            }
        }
    }; 
    // println!("{from_ident_for_ident_options_token_stream}");
    let column_variants = {
        let fields_named_enumerated = fields_named
            .iter()
            .enumerate()
            .map(|(index, field)| (index, field))
            .collect::<std::vec::Vec<(usize, &syn::Field)>>();
        let fields_named_clone_stringified = fields_named.iter().collect::<std::vec::Vec<&syn::Field>>();
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
            .collect::<std::vec::Vec<proc_macro2::TokenStream>>()
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
    let code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream = quote::quote!{
        #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream
    };
    let crate_code_occurence_tufa_common_macro_call_token_stream = quote::quote!{crate::code_occurence_tufa_common!()};
    let code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream = quote::quote!{
        #code_occurence_lower_case_token_stream: #crate_code_occurence_tufa_common_macro_call_token_stream
    };
    let eo_error_occurence_attribute_token_stream = proc_macro_helpers::error_occurence::named_attribute::NamedAttribute::EoErrorOccurence.to_attribute_view_token_stream();
    let eo_display_token_stream = proc_macro_helpers::error_occurence::named_attribute::NamedAttribute::EoDisplay.to_attribute_view_token_stream();
    let eo_display_with_serialize_deserialize_token_stream = proc_macro_helpers::error_occurence::named_attribute::NamedAttribute::EoDisplayWithSerializeDeserialize.to_attribute_view_token_stream();
    let eo_display_foreign_type_token_stream = proc_macro_helpers::error_occurence::named_attribute::NamedAttribute::EoDisplayForeignType.to_attribute_view_token_stream();
    let eo_vec_error_occurence_token_stream = proc_macro_helpers::error_occurence::named_attribute::NamedAttribute::EoVecErrorOccurence.to_attribute_view_token_stream();
    // let value_token_stream = quote::quote! {value};
    let impl_std_convert_try_from_ident_options_for_struct_variants_token_stream = {
        column_variants
            .iter()
            .map(|variant_columns| {
                let struct_name_stringified = {
                    let variant_columns_merged_camel_case_stringified = variant_columns.iter().fold(std::string::String::from(""), |mut acc, element| {
                        use convert_case::Casing;
                        let column_title_cased = element.ident
                            .clone()
                            .unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            })
                            .to_string().to_case(convert_case::Case::Title);
                        acc.push_str(&column_title_cased);
                        acc
                    });
                    format!("{ident}{variant_columns_merged_camel_case_stringified}")
                };
                let struct_name_token_stream = {
                    struct_name_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {struct_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let self_fields_token_stream = variant_columns.iter().map(|field|{
                    let field_ident = field.ident.clone().unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                    quote::quote! {
                        #field_ident
                    }   
                });
                let ident_try_from_ident_options_error_named_camel_case_token_stream = {
                    let ident_try_from_ident_options_error_named_camel_case_stringified = format!(
                        "{struct_name_stringified}{try_from_camel_case_stringified}{struct_options_ident_stringified}{error_named_camel_case_stringified}"
                    );
                    ident_try_from_ident_options_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ident_try_from_ident_options_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let is_none_camel_case_stringified = "IsNone";
                let postfix_is_none_lower_case_stringified = "_is_none";
                let ident_try_from_ident_options_error_named_token_stream = {
                    let uuid_wrapper_try_from_possible_uuid_wrapper_primary_key_variant_token_stream = match variant_columns.iter().find(|element| {
                        element.ident
                            .clone()
                            .unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            })
                            == id_field_ident
                    }) {
                        Some(value) => {
                            let column_variant_ident_stringified = value.ident.clone().unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                            match column_variant_ident_stringified == id_field_ident {
                                true => quote::quote!{
                                    #uuid_wrapper_try_from_possible_uuid_wrapper_camel_case_token_stream {
                                        #eo_error_occurence_attribute_token_stream
                                        #uuid_wrapper_try_from_possible_uuid_wrapper_lower_case_token_stream: #crate_server_postgres_uuid_wrapper_uuid_wrapper_try_from_possible_uuid_wrapper_error_named_token_stream,
                                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                                    },
                                },
                                false => proc_macro2::TokenStream::new()
                            }
                        },
                        None => proc_macro2::TokenStream::new()  
                    };
                    let is_none_variant_columns_token_stream = variant_columns.iter().map(|element|{
                        let field_ident = element.ident.clone().unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                        let field_ident_title_case_stringified = {
                            use convert_case::Casing;
                            field_ident.to_string().to_case(convert_case::Case::Title)
                        };
                        let field_ident_is_none_title_case_token_stream = {
                            let field_ident_is_none_title_case_stringified = format!("{field_ident_title_case_stringified}{is_none_camel_case_stringified}");
                            field_ident_is_none_title_case_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_is_none_title_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        let field_ident_is_none_lower_case_token_stream = {
                            let field_ident_is_none_lower_case_stringified = format!("{field_ident}{postfix_is_none_lower_case_stringified}");
                            field_ident_is_none_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_is_none_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote! {
                            #field_ident_is_none_title_case_token_stream {
                                #eo_display_with_serialize_deserialize_token_stream
                                #field_ident_is_none_lower_case_token_stream: #std_string_string_token_stream,
                                #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                            }
                        }   
                    });
                    quote::quote! {
                        #error_named_derive_token_stream
                        pub enum #ident_try_from_ident_options_error_named_camel_case_token_stream {
                            #uuid_wrapper_try_from_possible_uuid_wrapper_primary_key_variant_token_stream
                            #(#is_none_variant_columns_token_stream),*
                        }
                    }
                };
                let impl_std_convert_try_from_ident_options_ident_token_stream = {
                    let primary_key_field_assignment_token_stream = match variant_columns.iter().find(|element| {
                        element.ident
                            .clone()
                            .unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            })
                            == id_field_ident
                    }) {
                        Some(value) => {
                            let column_variant_ident_stringified = value.ident.clone().unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                            match column_variant_ident_stringified == id_field_ident {
                                true => quote::quote!{
                                    let #id_field_ident = match value.#id_field_ident {
                                        Some(value) => match #crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream::try_from(value) {
                                            Ok(value) => value.into_inner(),
                                            Err(e) => {
                                                return Err(Self::Error::#uuid_wrapper_try_from_possible_uuid_wrapper_camel_case_token_stream {
                                                    #uuid_wrapper_try_from_possible_uuid_wrapper_lower_case_token_stream: e,
                                                    #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                                });
                                            }
                                        },
                                        None => {
                                            return Err(Self::Error::IdIsNone {
                                                id_is_none: #std_string_string_token_stream::from("id is None"),//todo primary key field naming fix
                                                #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                            });
                                        }
                                    };
                                },
                                false => proc_macro2::TokenStream::new()
                            }
                        },
                        None => proc_macro2::TokenStream::new()  
                    };
                    let variant_columns_assignment_token_stream = variant_columns.iter().filter(|element|{
                        element.ident.clone().unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        })
                         != id_field_ident
                    }).map(|element|{
                        let field_ident = element.ident.clone().unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        });
                        let field_ident_title_case_stringified = {
                            use convert_case::Casing;
                            field_ident.to_string().to_case(convert_case::Case::Title)
                        };
                        let field_ident_is_none_title_case_token_stream = {
                            let field_ident_is_none_title_case_stringified = format!("{field_ident_title_case_stringified}{is_none_camel_case_stringified}");
                            field_ident_is_none_title_case_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_is_none_title_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        let field_ident_is_none_lower_case_token_stream = {
                            let field_ident_is_none_lower_case_stringified = format!("{field_ident}{postfix_is_none_lower_case_stringified}");
                            field_ident_is_none_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_is_none_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        let field_ident_is_none_message_lower_case_token_stream = {
                            let field_ident_is_none_lower_case_stringified = format!("\"{field_ident} is None\"");
                            field_ident_is_none_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_is_none_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote! {
                            let #field_ident = match value.#field_ident {
                                Some(value) => value,
                                None => {
                                    return Err(Self::Error::#field_ident_is_none_title_case_token_stream {
                                        #field_ident_is_none_lower_case_token_stream: #std_string_string_token_stream::from(#field_ident_is_none_message_lower_case_token_stream),
                                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                    });
                                }
                            };
                        }
                    });
                    quote::quote! {
                        impl std::convert::TryFrom<#struct_options_ident_token_stream> for #struct_name_token_stream {
                            type Error = #ident_try_from_ident_options_error_named_camel_case_token_stream;
                            fn try_from(value: #struct_options_ident_token_stream) -> Result<Self, Self::Error> {
                                #primary_key_field_assignment_token_stream
                                #(#variant_columns_assignment_token_stream)*
                                Ok(Self {
                                    #(#self_fields_token_stream),*
                                })
                            }
                        }
                    }
                };
                quote::quote! {
                    #ident_try_from_ident_options_error_named_token_stream
                    #impl_std_convert_try_from_ident_options_ident_token_stream
                }
            })
            .collect::<std::vec::Vec<proc_macro2::TokenStream>>()
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
            .collect::<std::vec::Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            #[derive(
                Debug,
                serde::Serialize,
                serde::Deserialize,
                enum_extension::EnumExtension,
                strum_macros::EnumIter,
                PartialEq,
                Eq,
                #from_str_lower_case_token_stream::#from_str_camel_case_token_stream,
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
    let column_select_camel_case_stringified = "ColumnSelect";
    let column_select_ident_token_stream = {
        let column_select_ident_stringified = format!("{ident}{column_select_camel_case_stringified}");
        column_select_ident_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {column_select_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let options_try_from_sqlx_row_name_token_stream = quote::quote!{options_try_from_sqlx_row};
    let crate_common_serde_urlencoded_serde_url_encoded_parameter_token_stream = quote::quote!{crate::common::serde_urlencoded::SerdeUrlencodedParameter};
    let ident_column_select_from_str_error_named_camel_case_token_stream = {
        let ident_column_select_from_str_error_named_camel_case_stringified = format!("{ident}{column_select_camel_case_stringified}{from_str_camel_case_stringified}{error_named_camel_case_stringified}");
        ident_column_select_from_str_error_named_camel_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ident_column_select_from_str_error_named_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_bind_query_try_generate_bind_increments_error_named_name_token_stream = {
        let crate_server_postgres_bind_query_try_generate_bind_increments_error_named_name_stringified = format!("crate::server::postgres::bind_query::TryGenerateBindIncrements{error_named_camel_case_stringified}");
        crate_server_postgres_bind_query_try_generate_bind_increments_error_named_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {crate_server_postgres_bind_query_try_generate_bind_increments_error_named_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
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
        // println!("{column_select_struct_token_stream}");
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
                    Self::#variant_ident_token_stream => #std_string_string_token_stream::from(#write_ident_token_stream)
                }
            });
            quote::quote!{
                impl crate::server::postgres::generate_query::GenerateQuery for #column_select_ident_token_stream {
                    fn generate_query(&self) -> #std_string_string_token_stream {
                        match self {
                            #(#generate_query_variants_token_stream),*
                        }
                    }
                }
            }
        };
        // println!("{generate_query_token_stream}");
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
        // println!("{impl_default_token_stream}");
        let from_option_self_token_stream = {
            quote::quote! {
                impl std::convert::From<std::option::Option<Self>> for #column_select_ident_token_stream {
                    fn from(option_value: std::option::Option<Self>) -> Self {
                        match option_value {
                            Some(value) => value,
                            None => Self::default(),
                        }
                    }
                }
            }
        };
        // println!("{from_option_self_token_stream}");
        let ident_column_select_from_str_error_named_token_stream = {
            quote::quote! {
                #error_named_derive_token_stream
                pub enum #ident_column_select_from_str_error_named_camel_case_token_stream {
                    NotCorrect {
                        #eo_display_with_serialize_deserialize_token_stream
                        not_correct_value: #std_string_string_token_stream,
                        #eo_display_with_serialize_deserialize_token_stream
                        supported_values: #std_string_string_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{ident_column_select_from_str_error_named_token_stream}");
        let impl_std_str_from_str_for_ident_column_select_token_stream = {
            let match_acceptable_variants_token_stream = column_variants.iter().map(|column_variant|{
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
                let write_ident_token_stream = {
                    let write_ident_stringified = format!("\"{variant_ident_stringified_handle}\"");
                    write_ident_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {write_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let variant_ident_token_stream = {
                    variant_ident_stringified_handle.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {variant_ident_stringified_handle} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                quote::quote! {
                    #write_ident_token_stream => Ok(Self::#variant_ident_token_stream)
                }
            });
            let supported_values_handle_token_stream = {
                let mut column_variants_stringified = column_variants.iter().fold(std::string::String::default(), |mut acc, column_variant| {
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
                    acc.push_str(&format!("\\\"{variant_ident_stringified_handle}\\\","));
                    acc
                });
                column_variants_stringified.pop();
                let supported_values_handle_stringified = format!("\"{column_variants_stringified}\"");
                supported_values_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {supported_values_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote! {
                impl #std_str_from_str_token_stream for #column_select_ident_token_stream {
                    type Err = #ident_column_select_from_str_error_named_camel_case_token_stream;
                    fn from_str(value: &str) -> Result<Self, Self::Err> {
                        match value {
                            #(#match_acceptable_variants_token_stream),*,
                            _ => Err(Self::Err::NotCorrect {
                                not_correct_value: #std_string_string_token_stream::from(value),
                                supported_values: #std_string_string_token_stream::from(#supported_values_handle_token_stream),
                                #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                            }),
                        }
                    }
                }
            }
        };
        // println!("{impl_std_str_from_str_for_ident_column_select_token_stream}");
        let serde_urlencoded_parameter_token_stream = {
            quote::quote! {
                impl #crate_common_serde_urlencoded_serde_url_encoded_parameter_token_stream for #column_select_ident_token_stream {
                    fn serde_urlencoded_parameter(self) -> #std_string_string_token_stream {
                        self.to_string()
                    }
                }
            }
        };
        // println!("{serde_urlencoded_parameter_token_stream}");
        let options_try_from_sqlx_row_token_stream = {
            let declaration_primary_key_token_stream = {
                quote::quote! {
                    let mut #id_field_ident: std::option::Option<#crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream> = None;
                }
            };
            let declaration_excluding_primary_key_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident
                    .clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                let field_type = &element.field.ty;
                quote::quote! {
                    let mut #field_ident: std::option::Option<#field_type> = None;
                }
            });
            let assignment_token_stream = column_variants.iter().map(|column_variant|{
                let write_ident_primary_key_token_stream = {
                    let field_ident_string_quotes_token_stream = {
                        let field_ident_string_quotes = format!("\"{id_field_ident}\"");
                        field_ident_string_quotes.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_string_quotes} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    };
                    quote::quote!{
                        let primary_key_try_get_result: Result<std::option::Option<#sqlx_types_uuid_token_stream>, sqlx::Error> = row.try_get(#field_ident_string_quotes_token_stream);
                        #id_field_ident = match primary_key_try_get_result {
                            Ok(option_primary_key) => option_primary_key.map(|value| #crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream::from(value)),
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
                std::option::Option<#sqlx_types_uuid_token_stream>: #sqlx_decode_decode_database_token_stream,
                std::option::Option<#sqlx_types_uuid_token_stream>: #sqlx_types_type_database_token_stream,
            };
            let sqlx_decode_decode_and_sqlx_types_type_with_excluded_id_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element| {
                let field_type = &element.field.ty;
                quote::quote!{
                    std::option::Option<#field_type>: #sqlx_decode_decode_database_token_stream,
                    std::option::Option<#field_type>: #sqlx_types_type_database_token_stream,
                }
            });
            quote::quote! {
                impl #column_select_ident_token_stream {
                    fn #options_try_from_sqlx_row_name_token_stream<'a, R: #sqlx_row_token_stream>(
                        &self,
                        row: &'a R,
                    ) -> sqlx::Result<#struct_options_ident_token_stream>
                    where
                        #std_primitive_str_sqlx_column_index_token_stream
                        #sqlx_decode_decode_and_sqlx_types_type_primary_key_token_stream
                        #(#sqlx_decode_decode_and_sqlx_types_type_with_excluded_id_token_stream)*
                    {
                        #declaration_primary_key_token_stream
                        #(#declaration_excluding_primary_key_token_stream)*
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
            #ident_column_select_from_str_error_named_token_stream
            #impl_std_str_from_str_for_ident_column_select_token_stream
            #serde_urlencoded_parameter_token_stream
            #options_try_from_sqlx_row_token_stream
        }
    };
    // println!("{column_select_token_stream}");
    let crate_server_postgres_regex_filter_regex_filter_token_stream = quote::quote!{crate::server::postgres::regex_filter::RegexFilter};
    let crate_server_postgres_postgres_bigint_postgres_bigint_token_stream = quote::quote!{crate::server::postgres::postgres_bigint::PostgresBigint};
    let primary_key_uuid_wrapper_try_from_sqlx_row_token_stream = {
        let primary_key_str_token_stream = {
            let primary_key_str_stringified = format!("\"{id_field_ident}\"");
            primary_key_str_stringified.parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {primary_key_str_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let row_name_token_stream = quote::quote!{row};
        let primary_key_name_token_stream = quote::quote!{primary_key};
        quote::quote! {
            fn #primary_key_uuid_wrapper_try_from_sqlx_row_name_token_stream<'a, R: #sqlx_row_token_stream>(#row_name_token_stream: &'a R) -> sqlx::Result<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>
            where
                #std_primitive_str_sqlx_column_index_token_stream
                #sqlx_types_uuid_token_stream: #sqlx_decode_decode_database_token_stream,
                #sqlx_types_uuid_token_stream: #sqlx_types_type_database_token_stream,
            {
                let #primary_key_name_token_stream: #sqlx_types_uuid_token_stream = #row_name_token_stream.try_get(#primary_key_str_token_stream)?;
                Ok(#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream::from(#primary_key_name_token_stream))
            }
        }
    };
    // println!("{primary_key_uuid_wrapper_try_from_sqlx_row_token_stream}");
    let order_camel_case_stringified = "Order";
    // let order_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&order_camel_case_stringified);
    let order_by_camel_case_stringified = format!("{order_camel_case_stringified}By");
    let order_by_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&order_by_camel_case_stringified);
    let order_by_camel_case_token_stream = {
        order_by_camel_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {order_by_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let order_by_lower_case_token_stream = {
        order_by_lower_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {order_by_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let crate_server_postgres_order_by_order_by_token_stream = quote::quote!{crate::server::postgres::#order_by_lower_case_token_stream::#order_by_camel_case_token_stream};
    let crate_server_postgres_order_order_token_stream = quote::quote!{crate::server::postgres::order::Order};
    let ident_order_by_wrapper_stringified = format!("{ident}{order_by_camel_case_stringified}Wrapper");
    let ident_order_by_wrapper_name_token_stream = {
        ident_order_by_wrapper_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ident_order_by_wrapper_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let ident_order_by_wrapper_from_str_error_named_name_token_stream = {
        let ident_order_by_wrapper_from_str_error_named_name_stringified = format!("{ident_order_by_wrapper_stringified}{from_str_camel_case_stringified}{error_named_camel_case_stringified}");
        ident_order_by_wrapper_from_str_error_named_name_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ident_order_by_wrapper_from_str_error_named_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
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
            ) -> Result<#crate_server_postgres_order_by_order_by_token_stream<#column_ident_token_stream>, D::Error>
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
                                            use #std_str_from_str_token_stream;
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
                                        use #std_str_from_str_token_stream;
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
                                            use #std_str_from_str_token_stream;
                                            #crate_server_postgres_order_order_token_stream::from_str(possible_order)
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
                                        use #std_str_from_str_token_stream;
                                        #crate_server_postgres_order_order_token_stream::from_str(possible_order)
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
                Ok(#crate_server_postgres_order_by_order_by_token_stream { column, order })
            }
        }
    };
    let order_by_wrapper_token_stream = {
        let struct_token_stream = {
            let deserialize_with_name_quotes_token_stream = {
                let deserialize_with_name_quotes_stringified = format!("\"deserialize_{ident_lower_case_stringified}_order_by\"");
                deserialize_with_name_quotes_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {deserialize_with_name_quotes_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #ident_order_by_wrapper_name_token_stream(
                    #[serde(deserialize_with = #deserialize_with_name_quotes_token_stream)]
                    pub #crate_server_postgres_order_by_order_by_token_stream<#column_ident_token_stream>,
                );
            }
        };
        let impl_crate_common_serde_urlencoded_serde_urlencoded_parameter_for_ident_order_by_wrapper_token_stream = {
            quote::quote!{
                impl #crate_common_serde_urlencoded_serde_url_encoded_parameter_token_stream for #ident_order_by_wrapper_name_token_stream {
                    fn serde_urlencoded_parameter(self) -> #std_string_string_token_stream {
                        let column = &self.0.column;
                        let order = self.0.order.unwrap_or_default();
                        format!("column={column},order={order}")
                    }
                }
            }
        };
        let ident_order_by_wrapper_from_str_error_named_token_stream = {
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #ident_order_by_wrapper_from_str_error_named_name_token_stream {
                    ColumnFromStr {
                        #eo_display_with_serialize_deserialize_token_stream
                        column_from_str: #std_string_string_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                    ColumnNoOffsetValue {
                        #eo_display_with_serialize_deserialize_token_stream
                        column_no_offset_value: #std_string_string_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                    ColumnOffsetSliceGet {
                        #eo_display_with_serialize_deserialize_token_stream
                        column_offset_slice_get: #std_string_string_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                    ColumnStringDeserializedGet {
                        #eo_display_with_serialize_deserialize_token_stream
                        column_string_deserialized_get: #std_string_string_token_stream,
                        code_occurence: #crate_common_code_occurence_code_occurence_token_stream,
                    },   
                    ColumnIndexCheckedAdd {
                        #eo_display_with_serialize_deserialize_token_stream
                        column_index_checked_add: #std_string_string_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                    ColumnStringDeserializedFind {
                        #eo_display_with_serialize_deserialize_token_stream
                        column_string_deserialized_find: #std_string_string_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },    
                    //todo make client explicitly write order and column
                    OrderFromStr {
                        #eo_display_with_serialize_deserialize_token_stream
                        order_from_str: #std_string_string_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                    OrderOffsetSliceGetNone {
                        #eo_display_with_serialize_deserialize_token_stream
                        order_offset_slice_get_none: #std_string_string_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                    OrderStringDeserializedGetNone {
                        #eo_display_with_serialize_deserialize_token_stream
                        order_string_deserialized_get_none: #std_string_string_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                    OrderIndexCheckedAdd {
                        #eo_display_with_serialize_deserialize_token_stream
                        order_index_checked_add: #std_string_string_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        let impl_std_str_from_str_for_ident_order_by_wrapper_token_stream = {
            let default_message_handle_token_stream = {
                let default_message_handle_stringified = format!("\"Invalid {ident}OrderBy:\"");
                default_message_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {default_message_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                impl #std_str_from_str_token_stream for #ident_order_by_wrapper_name_token_stream {
                    type Err = #ident_order_by_wrapper_from_str_error_named_name_token_stream;
                    fn from_str(value: &str) -> Result<Self, Self::Err> {
                        let string_deserialized = value.to_string();
                        let split_inner_url_parameters_symbol = ',';
                        let default_message = format!(#default_message_handle_token_stream);
                        let column_equal_str = "column=";
                        let order_equal_str = "order=";
                        let column = match string_deserialized.find(column_equal_str) {
                            Some(index) => match index.checked_add(column_equal_str.len()) {
                                Some(offset) => match string_deserialized.get(offset..) {
                                    Some(offset_slice) => match offset_slice.find(split_inner_url_parameters_symbol) {
                                        Some(offset_slice_next_comma_index) => {
                                            match offset_slice.get(0..offset_slice_next_comma_index) {
                                                Some(possible_column) => match #column_ident_token_stream::from_str(possible_column) {
                                                    Ok(column) => column,
                                                    Err(e) => {
                                                        return Err(Self::Err::ColumnFromStr {
                                                            column_from_str: e,
                                                            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                                        });
                                                    }
                                                },
                                                None => {
                                                    return Err(Self::Err::ColumnNoOffsetValue {
                                                        column_no_offset_value: #std_string_string_token_stream::from("no offset value"),
                                                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                                    });
                                                }
                                            }
                                        }
                                        None => match offset_slice.get(0..) {
                                            Some(possible_column) => match #column_ident_token_stream::from_str(possible_column) {
                                                Ok(column) => column,
                                                Err(e) => {
                                                    return Err(Self::Err::ColumnFromStr {
                                                        column_from_str: e,
                                                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                                    });
                                                }
                                            },
                                            None => {
                                                return Err(Self::Err::ColumnOffsetSliceGet {
                                                    column_offset_slice_get: #std_string_string_token_stream::from("offset_slice_get"),
                                                    #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                                });
                                            }
                                        },
                                    },
                                    None => {
                                        return Err(Self::Err::ColumnStringDeserializedGet {
                                            column_string_deserialized_get: #std_string_string_token_stream::from("string_deserialized_get"),
                                            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                        });
                                    }
                                },
                                None => {
                                    return Err(Self::Err::ColumnIndexCheckedAdd {
                                        column_index_checked_add: #std_string_string_token_stream::from("index_checked_add"),
                                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                    });
                                }
                            },
                            None => {
                                return Err(Self::Err::ColumnStringDeserializedFind {
                                    column_string_deserialized_find: #std_string_string_token_stream::from("string_deserialized_find"),
                                    #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                });
                            }
                        };
                        let order = match string_deserialized.find(order_equal_str) {
                            Some(index) => match index.checked_add(order_equal_str.len()) {
                                Some(offset) => match string_deserialized.get(offset..) {
                                    Some(offset_slice) => match offset_slice.find(split_inner_url_parameters_symbol) {
                                        Some(offset_slice_next_comma_index) => {
                                            match offset_slice.get(0..offset_slice_next_comma_index) {
                                                Some(possible_order) => match #crate_server_postgres_order_order_token_stream::from_str(possible_order) {
                                                    Ok(order) => Some(order),
                                                    Err(e) => {
                                                        return Err(Self::Err::OrderFromStr {
                                                            order_from_str: e,
                                                            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                                        });
                                                    }
                                                },
                                                None => {
                                                    return Err(Self::Err::OrderOffsetSliceGetNone {
                                                        order_offset_slice_get_none: #std_string_string_token_stream::from("order_offset_slice_get_none"),
                                                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                                    });
                                                }
                                            }
                                        }
                                        None => match offset_slice.get(0..) {
                                            Some(possible_order) => match #crate_server_postgres_order_order_token_stream::from_str(possible_order) {
                                                Ok(order) => Some(order),
                                                Err(e) => {
                                                    return Err(Self::Err::OrderFromStr {
                                                        order_from_str: e,
                                                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream
                                                    });
                                                }
                                            },
                                            None => {
                                                return Err(Self::Err::OrderOffsetSliceGetNone {
                                                    order_offset_slice_get_none: #std_string_string_token_stream::from("order_offset_slice_get_none"),
                                                    #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                                });
                                            }
                                        },
                                    },
                                    None => {
                                        return Err(Self::Err::OrderStringDeserializedGetNone {
                                            order_string_deserialized_get_none: #std_string_string_token_stream::from("string_deserialized_get_none"),
                                            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                        });
                                    }
                                },
                                None => {
                                    return Err(Self::Err::OrderIndexCheckedAdd {
                                        order_index_checked_add: #std_string_string_token_stream::from("order_index_checked_add"),
                                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                    });
                                }
                            },
                            None => None,
                        };
                        Ok(Self(#crate_server_postgres_order_by_order_by_token_stream { column, order }))
                    }
                }
            }
        };
        quote::quote!{
            #struct_token_stream
            #impl_crate_common_serde_urlencoded_serde_urlencoded_parameter_for_ident_order_by_wrapper_token_stream
            #ident_order_by_wrapper_from_str_error_named_token_stream
            #impl_std_str_from_str_for_ident_order_by_wrapper_token_stream
        }
    };
    // println!("{order_by_wrapper_token_stream}");
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
    let payload_element_camel_case_stringified = format!("{payload_camel_case_stringified}Element");
    let payload_element_with_serialize_deserialize_camel_case_stringified = format!("{payload_element_camel_case_stringified}{with_serialize_deserialize_camel_case_stringified}");
    let tvfrr_extraction_logic_lower_case_stringified = "tvfrr_extraction_logic";
    let request_error_camel_case_stringified = "RequestError";
    let returning_stringified = "returning";
    let returning_id_stringified = format!(" {returning_stringified} {id_field_ident}");
    let batch_stringified = "batch";
    let serde_urlencoded_to_string_token_stream = quote::quote!{serde_urlencoded::to_string};
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
    let created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_camel_case_stringified = "CreatedButCannotConvertUuidWrapperFromPossibleUuidWrapper";
    let created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_server_camel_case_stringified = format!("{created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_camel_case_stringified}InServer");
    let created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_server_camel_case_token_stream = {
        created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_server_camel_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_server_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_camel_case_stringified = format!("{created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_camel_case_stringified}InClient");//todo reuse it
    let created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_camel_case_token_stream = {
        created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_camel_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_lower_case_token_stream = {
        let created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_camel_case_stringified.to_string());
        created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_lower_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_error_unnamed_camel_case_token_stream = {
        let created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_lower_case_stringified = format!("{created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_camel_case_stringified}ErrorUnnamed");
        created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_lower_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
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
    let request_error_variant_initialization_token_stream = quote::quote!{
        #request_error_camel_case_token_stream {
            #request_error_lower_case_token_stream: e,
            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
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
            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let http_request_error_named_serde_json_to_string_variant_token_stream = quote::quote!{
        #serde_json_to_string_camel_case_token_stream {
            #eo_display_token_stream
            #serde_json_to_string_lower_case_token_stream: serde_json::Error,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let std_string_string_syn_punctuated_punctuated = {
        let mut handle = syn::punctuated::Punctuated::<syn::PathSegment, syn::token::Colon2>::new();
        handle.push_value(
            syn::PathSegment {
                ident: proc_macro2::Ident::new("std", proc_macro2::Span::call_site()),
                arguments: syn::PathArguments::None,
            }
        );
        handle.push_punct(syn::token::Colon2{
            spans: [proc_macro2::Span::call_site(),proc_macro2::Span::call_site()],
        });
        handle.push_value(
            syn::PathSegment {
                ident: proc_macro2::Ident::new("string", proc_macro2::Span::call_site()),
                arguments: syn::PathArguments::None,
            }
        );
        handle.push_punct(syn::token::Colon2{
            spans: [proc_macro2::Span::call_site(),proc_macro2::Span::call_site()],
        });
        handle.push_value(
            syn::PathSegment {
                ident: proc_macro2::Ident::new("String", proc_macro2::Span::call_site()),
                arguments: syn::PathArguments::None,
            }
        );
        handle
    };
    let code_occurence_field = syn::Field {
        attrs: vec![],
        vis: syn::Visibility::Inherited,
        ident: Some(
            syn::Ident::new("code_occurence", proc_macro2::Span::call_site())
        ),
        colon_token: Some(
            syn::token::Colon {
                spans: [proc_macro2::Span::call_site()],
            },
        ),
        ty: syn::Type::Path(
            syn::TypePath {
                qself: None,
                path: syn::Path {
                    leading_colon: None,
                    segments: {
                        let mut handle = syn::punctuated::Punctuated::<syn::PathSegment, syn::token::Colon2>::new();
                        handle.push_value(
                            syn::PathSegment {
                                ident: proc_macro2::Ident::new("crate", proc_macro2::Span::call_site()),
                                arguments: syn::PathArguments::None,
                            }
                        );
                        handle.push_punct(syn::token::Colon2{
                            spans: [proc_macro2::Span::call_site(),proc_macro2::Span::call_site()],
                        });
                        handle.push_value(
                            syn::PathSegment {
                                ident: proc_macro2::Ident::new("common", proc_macro2::Span::call_site()),
                                arguments: syn::PathArguments::None,
                            }
                        );
                        handle.push_punct(syn::token::Colon2{
                            spans: [proc_macro2::Span::call_site(),proc_macro2::Span::call_site()],
                        });
                        handle.push_value(
                            syn::PathSegment {
                                ident: proc_macro2::Ident::new("code_occurence", proc_macro2::Span::call_site()),
                                arguments: syn::PathArguments::None,
                            }
                        );
                        handle.push_punct(syn::token::Colon2{
                            spans: [proc_macro2::Span::call_site(),proc_macro2::Span::call_site()],
                        });
                        handle.push_value(
                            syn::PathSegment {
                                ident: proc_macro2::Ident::new("CodeOccurence", proc_macro2::Span::call_site()),
                                arguments: syn::PathArguments::None,
                            }
                        );
                        handle
                    }
                },
            },
        ),
    };
    let error_occurence_variant_field = crate::type_variants_from_request_response_generator::ErrorVariantField {
        field_name: quote::quote!{#code_occurence_lower_case_token_stream},
        error_occurence_attribute: quote::quote!{},
        field_type_original: quote::quote!{#crate_common_code_occurence_code_occurence_token_stream},
        field_type_with_serialize_deserialize: quote::quote!{#crate_common_code_occurence_code_occurence_token_stream},
    };
    let bind_query_syn_variant = syn::Variant {
        attrs: vec![
            syn::Attribute {
                pound_token: syn::token::Pound {
                    spans: [proc_macro2::Span::call_site()],
                },
                style: syn::AttrStyle::Outer,
                bracket_token: syn::token::Bracket {
                    span: proc_macro2::Span::call_site(),
                },
                path: syn::Path {
                    leading_colon: None,
                    segments: {
                        let mut handle = syn::punctuated::Punctuated::new();
                        handle.push(syn::PathSegment {
                            ident: proc_macro2::Ident::new("tvfrr_500_internal_server_error", proc_macro2::Span::call_site()),
                            arguments: syn::PathArguments::None,
                        });
                       handle
                    },
                },
                tokens: proc_macro2::TokenStream::new(),
            },
        ],
        ident: syn::Ident::new("BindQuery", proc_macro2::Span::call_site()),
        fields: syn::Fields::Named(
            syn::FieldsNamed {
                brace_token: syn::token::Brace {
                    span: proc_macro2::Span::call_site(),
                },
                named: {
                    let mut handle = syn::punctuated::Punctuated::new();
                    handle.push_value(
                        syn::Field {
                            attrs: vec![
                                syn::Attribute {
                                    pound_token: syn::token::Pound {
                                        spans: [proc_macro2::Span::call_site()],
                                    },
                                    style: syn::AttrStyle::Outer,
                                    bracket_token: syn::token::Bracket {
                                        span: proc_macro2::Span::call_site(),
                                    },
                                    path: syn::Path {
                                        leading_colon: None,
                                        segments: {
                                            let mut handle = syn::punctuated::Punctuated::new();
                                            handle.push(
                                                syn::PathSegment {
                                                    ident: proc_macro2::Ident::new("eo_error_occurence", proc_macro2::Span::call_site()),
                                                    arguments: syn::PathArguments::None,
                                                }
                                            );
                                            handle
                                        },
                                    },
                                    tokens: proc_macro2::TokenStream::new(),
                                },
                            ],
                            vis: syn::Visibility::Inherited,
                            ident: Some(
                                syn::Ident::new("checked_add", proc_macro2::Span::call_site())
                            ),
                            colon_token: Some(
                                syn::token::Colon {
                                    spans: [proc_macro2::Span::call_site()],
                                },
                            ),
                            ty: syn::Type::Path(
                                syn::TypePath {
                                    qself: None,
                                    path: syn::Path {
                                        leading_colon: None,
                                        segments: {
                                            let mut handle = syn::punctuated::Punctuated::<syn::PathSegment, syn::token::Colon2>::new();
                                            handle.push_value(
                                                syn::PathSegment {
                                                    ident: proc_macro2::Ident::new("crate", proc_macro2::Span::call_site()),
                                                    arguments: syn::PathArguments::None,
                                                }
                                            );
                                            handle.push_punct(syn::token::Colon2{
                                                spans: [proc_macro2::Span::call_site(),proc_macro2::Span::call_site()],
                                            });
                                            handle.push_value(
                                                syn::PathSegment {
                                                    ident: proc_macro2::Ident::new("server", proc_macro2::Span::call_site()),
                                                    arguments: syn::PathArguments::None,
                                                }
                                            );
                                            handle.push_punct(syn::token::Colon2{
                                                spans: [proc_macro2::Span::call_site(),proc_macro2::Span::call_site()],
                                            });
                                            handle.push_value(
                                                syn::PathSegment {
                                                    ident: proc_macro2::Ident::new("postgres", proc_macro2::Span::call_site()),
                                                    arguments: syn::PathArguments::None,
                                                }
                                            );
                                            handle.push_punct(syn::token::Colon2{
                                                spans: [proc_macro2::Span::call_site(),proc_macro2::Span::call_site()],
                                            });
                                            handle.push_value(
                                                syn::PathSegment {
                                                    ident: proc_macro2::Ident::new("bind_query", proc_macro2::Span::call_site()),
                                                    arguments: syn::PathArguments::None,
                                                }
                                            );
                                            handle.push_punct(syn::token::Colon2{
                                                spans: [proc_macro2::Span::call_site(),proc_macro2::Span::call_site()],
                                            });
                                            handle.push_value(
                                                syn::PathSegment {
                                                    ident: proc_macro2::Ident::new("TryGenerateBindIncrementsErrorNamed", proc_macro2::Span::call_site()),
                                                    arguments: syn::PathArguments::None,
                                                }
                                            );
                                            handle
                                        }
                                    },
                                },
                            ),
                        }
                    );
                    handle.push_punct(
                        syn::token::Comma {
                            spans: [proc_macro2::Span::call_site()],
                        }
                    );
                    handle.push_value(code_occurence_field.clone());
                    handle
                },
            },
        ),
        discriminant: None,
    };
    let bind_query_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{BindQuery},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{checked_add},
                    error_occurence_attribute: quote::quote!{#eo_error_occurence_attribute_token_stream},
                    field_type_original: quote::quote!{crate::server::postgres::bind_query::TryGenerateBindIncrementsErrorNamed},
                    field_type_with_serialize_deserialize: quote::quote!{crate::server::postgres::bind_query::TryGenerateBindIncrementsErrorNamedWithSerializeDeserialize},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let bind_query_variant_initialization_token_stream = quote::quote!{
        BindQuery { 
            checked_add: e.into_serialize_deserialize_version(), 
            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream
        }
    };
    let bind_query_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_500_internal_server_error]
        BindQuery {
            #eo_error_occurence_attribute_token_stream
            checked_add: crate::server::postgres::bind_query::TryGenerateBindIncrementsErrorNamed,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let checked_add_syn_variant = construct_syn_variant(
        proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
        "CheckedAdd",
        &code_occurence_field,
        vec![(
            proc_macro_helpers::error_occurence::named_attribute::NamedAttribute::EoDisplayWithSerializeDeserialize, 
            "checked_add", 
            std_string_string_syn_punctuated_punctuated.clone()
        )]
    );

    //
 
    //
    let checked_add_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{CheckedAdd},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{checked_add},
                    error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                    field_type_original: quote::quote!{#std_string_string_token_stream},
                    field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let checked_add_variant_initialization_token_stream = quote::quote!{
        CheckedAdd { //todo remove it? refactor it?
            checked_add: #std_string_string_token_stream::from("checked_add is None"), 
            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream, 
        }
    };
    let checked_add_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_500_internal_server_error]
        CheckedAdd {
            #eo_display_with_serialize_deserialize_token_stream
            checked_add: #std_string_string_token_stream,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let query_and_rollback_failed_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{QueryAndRollbackFailed},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{query_error},
                    error_occurence_attribute: quote::quote!{#eo_display_token_stream},
                    field_type_original: quote::quote!{sqlx::Error},
                    field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                },
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{rollback_error},
                    error_occurence_attribute: quote::quote!{#eo_display_token_stream},
                    field_type_original: quote::quote!{sqlx::Error},
                    field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let query_and_rollback_failed_variant_initialization_token_stream = quote::quote!{
        QueryAndRollbackFailed {
            query_error: e,
            #rollback_error_name_token_stream,
            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let query_and_rollback_failed_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_500_internal_server_error]
        QueryAndRollbackFailed {
            #eo_display_token_stream
            query_error: sqlx::Error,
            #eo_display_token_stream
            rollback_error: sqlx::Error,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let primary_key_from_row_and_failed_rollback_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{PrimaryKeyFromRowAndFailedRollback},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{primary_key_from_row},
                    error_occurence_attribute: quote::quote!{#eo_display_token_stream},
                    field_type_original: quote::quote!{sqlx::Error},
                    field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                },
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{rollback_error},
                    error_occurence_attribute: quote::quote!{#eo_display_token_stream},
                    field_type_original: quote::quote!{sqlx::Error},
                    field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let primary_key_from_row_and_failed_rollback_variant_initialization_token_stream = quote::quote!{
        PrimaryKeyFromRowAndFailedRollback {
            primary_key_from_row: e,
            #rollback_error_name_token_stream,
            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let primary_key_from_row_and_failed_rollback_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_500_internal_server_error]
        PrimaryKeyFromRowAndFailedRollback {
            #eo_display_token_stream
            primary_key_from_row: sqlx::Error,
            #eo_display_token_stream
            rollback_error: sqlx::Error,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let non_existing_primary_keys_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{NonExistingPrimaryKeys},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{non_existing_primary_keys},
                    error_occurence_attribute: quote::quote!{#[eo_vec_display]},
                    field_type_original: quote::quote!{std::vec::Vec<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>},
                    field_type_with_serialize_deserialize: quote::quote!{std::vec::Vec<#std_string_string_token_stream>},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let non_existing_primary_keys_variant_initialization_token_stream = quote::quote!{
        NonExistingPrimaryKeys {
            #non_existing_primary_keys_name_token_stream,
            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let non_existing_primary_keys_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request]
        NonExistingPrimaryKeys {
            #[eo_vec_display]
            non_existing_primary_keys: std::vec::Vec<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let non_existing_primary_keys_and_failed_rollback_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{NonExistingPrimaryKeysAndFailedRollback},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{non_existing_primary_keys},
                    error_occurence_attribute: quote::quote!{#[eo_vec_display]},
                    field_type_original: quote::quote!{std::vec::Vec<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>},
                    field_type_with_serialize_deserialize: quote::quote!{std::vec::Vec<#std_string_string_token_stream>},
                },
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{rollback_error},
                    error_occurence_attribute: quote::quote!{#eo_display_token_stream},
                    field_type_original: quote::quote!{sqlx::Error},
                    field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let non_existing_primary_keys_and_failed_rollback_variant_initialization_token_stream = quote::quote!{
        NonExistingPrimaryKeysAndFailedRollback {
            #non_existing_primary_keys_name_token_stream,
            #rollback_error_name_token_stream: e,
            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let non_existing_primary_keys_and_failed_rollback_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request]
        //todo what status code should return if non_existing_primary_keys = 400, but transaction rollback failed = 500
        NonExistingPrimaryKeysAndFailedRollback {
            #[eo_vec_display]
            non_existing_primary_keys: std::vec::Vec<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>,
            #eo_display_token_stream
            rollback_error: sqlx::Error,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    //
    let commit_failed_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{CommitFailed},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{commit_error},
                    error_occurence_attribute: quote::quote!{#eo_display_token_stream},
                    field_type_original: quote::quote!{sqlx::Error},
                    field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    //
    let commit_failed_variant_initialization_token_stream = quote::quote!{
        CommitFailed {
            commit_error: e,
            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let commit_failed_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_500_internal_server_error]
        CommitFailed {
            #eo_display_token_stream
            commit_error: sqlx::Error,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let not_unique_primary_keys_name_token_stream = quote::quote!{not_unique_primary_keys};
    let not_unique_primary_key_variant_initialization_token_stream = quote::quote!{
        NotUniquePrimaryKey {
            not_unique_primary_keys,
            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
        }
    };
    let not_unique_primary_key_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request]
        NotUniquePrimaryKey {
            #[eo_vec_display]
            not_unique_primary_keys: std::vec::Vec<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let not_unique_primary_key_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{NotUniquePrimaryKey},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{not_unique_primary_keys},
                    error_occurence_attribute: quote::quote!{#[eo_vec_display]},
                    field_type_original: quote::quote!{std::vec::Vec<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>},
                    field_type_with_serialize_deserialize: quote::quote!{std::vec::Vec<#std_string_string_token_stream>},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_server_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{CreatedButCannotConvertUuidWrapperFromPossibleUuidWrapperInServer},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{uuid_wrapper_try_from_possible_uuid_wrapper_in_server},
                    error_occurence_attribute: quote::quote!{#eo_display_token_stream},
                    field_type_original: quote::quote!{sqlx::Error},
                    field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_server_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_500_internal_server_error] //todo what status should be there?
        CreatedButCannotConvertUuidWrapperFromPossibleUuidWrapperInServer {
            #eo_display_token_stream
            uuid_wrapper_try_from_possible_uuid_wrapper_in_server: sqlx::Error,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let failed_to_deserialize_query_string_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{FailedToDeserializeQueryString},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{failed_to_deserialize_query_string},
                    error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                    field_type_original: quote::quote!{#std_string_string_token_stream},
                    field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let failed_to_deserialize_query_string_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request]
        FailedToDeserializeQueryString {
            #eo_display_with_serialize_deserialize_token_stream
            failed_to_deserialize_query_string: #std_string_string_token_stream,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    //todo
    let read_one_path_try_from_read_one_path_with_serialize_deserialize_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{ReadOnePathTryFromReadOnePathWithSerializeDeserialize},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{read_one_path_try_from_read_one_path_with_serialize_deserialize},
                    error_occurence_attribute: quote::quote!{#[eo_error_occurence]},
                    field_type_original: quote::quote!{ReadOnePathTryFromReadOnePathWithSerializeDeserializeErrorNamed},
                    field_type_with_serialize_deserialize: quote::quote!{ReadOnePathTryFromReadOnePathWithSerializeDeserializeErrorNamedWithSerializeDeserialize},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let read_many_with_body_payload_try_from_read_many_with_body_payload_with_serialize_deserialize_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{ReadManyWithBodyPayloadTryFromReadManyWithBodyPayloadWithSerializeDeserialize},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{read_many_with_body_payload_try_from_read_many_with_body_payload_with_serialize_deserialize},
                    error_occurence_attribute: quote::quote!{#[eo_error_occurence]},
                    field_type_original: quote::quote!{ReadManyWithBodyPayloadTryFromReadManyWithBodyPayloadWithSerializeDeserializeErrorNamed},
                    field_type_with_serialize_deserialize: quote::quote!{ReadManyWithBodyPayloadTryFromReadManyWithBodyPayloadWithSerializeDeserializeErrorNamedWithSerializeDeserialize},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let read_many_with_body_payload_try_from_read_many_with_body_payload_with_serialize_deserialize_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request]
        ReadManyWithBodyPayloadTryFromReadManyWithBodyPayloadWithSerializeDeserialize {
            #[eo_error_occurence]
            read_many_with_body_payload_try_from_read_many_with_body_payload_with_serialize_deserialize: ReadManyWithBodyPayloadTryFromReadManyWithBodyPayloadWithSerializeDeserializeErrorNamed,
            code_occurence: crate::common::code_occurence::CodeOccurence,
        }
    };
    let update_one_path_try_from_update_one_path_with_serialize_deserialize_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{UpdateOnePathTryFromUpdateOnePathWithSerializeDeserialize},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{update_one_path_try_from_update_one_path_with_serialize_deserialize},
                    error_occurence_attribute: quote::quote!{#[eo_error_occurence]},
                    field_type_original: quote::quote!{UpdateOnePathTryFromUpdateOnePathWithSerializeDeserializeErrorNamed},
                    field_type_with_serialize_deserialize: quote::quote!{UpdateOnePathTryFromUpdateOnePathWithSerializeDeserializeErrorNamedWithSerializeDeserialize},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let update_one_path_try_from_update_one_path_with_serialize_deserialize_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request]
        UpdateOnePathTryFromUpdateOnePathWithSerializeDeserialize {
            #[eo_error_occurence]
            update_one_path_try_from_update_one_path_with_serialize_deserialize: UpdateOnePathTryFromUpdateOnePathWithSerializeDeserializeErrorNamed,
            code_occurence: crate::common::code_occurence::CodeOccurence,
        }
    };
    let update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{UpdateManyPayloadElementTryFromUpdateManyPayloadElementWithSerializeDeserialize},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize},
                    error_occurence_attribute: quote::quote!{#[eo_error_occurence]},
                    field_type_original: quote::quote!{UpdateManyPayloadElementTryFromUpdateManyPayloadElementWithSerializeDeserializeErrorNamed},
                    field_type_with_serialize_deserialize: quote::quote!{UpdateManyPayloadElementTryFromUpdateManyPayloadElementWithSerializeDeserializeErrorNamedWithSerializeDeserialize},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request]
        UpdateManyPayloadElementTryFromUpdateManyPayloadElementWithSerializeDeserialize {
            #[eo_error_occurence]
            update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize: UpdateManyPayloadElementTryFromUpdateManyPayloadElementWithSerializeDeserializeErrorNamed,
            code_occurence: crate::common::code_occurence::CodeOccurence,
        } 
    };
    let delete_one_path_try_from_delete_one_path_with_serialize_deserialize_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{DeleteOnePathTryFromDeleteOnePathWithSerializeDeserialize},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{delete_one_path_try_from_delete_one_path_with_serialize_deserialize},
                    error_occurence_attribute: quote::quote!{#[eo_error_occurence]},
                    field_type_original: quote::quote!{DeleteOnePathTryFromDeleteOnePathWithSerializeDeserializeErrorNamed},
                    field_type_with_serialize_deserialize: quote::quote!{DeleteOnePathTryFromDeleteOnePathWithSerializeDeserializeErrorNamedWithSerializeDeserialize},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let delete_one_path_try_from_delete_one_path_with_serialize_deserialize_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request]
        DeleteOnePathTryFromDeleteOnePathWithSerializeDeserialize {
            #[eo_error_occurence]
            delete_one_path_try_from_delete_one_path_with_serialize_deserialize: DeleteOnePathTryFromDeleteOnePathWithSerializeDeserializeErrorNamed,
            code_occurence: crate::common::code_occurence::CodeOccurence,
        }
    };
    let delete_many_with_body_payload_try_from_delete_many_with_body_payload_with_serialize_deserialize_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{DeleteManyWithBodyPayloadTryFromDeleteManyWithBodyPayloadWithSerializeDeserialize},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{delete_many_with_body_payload_try_from_delete_many_with_body_payload_with_serialize_deserialize},
                    error_occurence_attribute: quote::quote!{#[eo_error_occurence]},
                    field_type_original: quote::quote!{DeleteManyWithBodyPayloadTryFromDeleteManyWithBodyPayloadWithSerializeDeserializeErrorNamed},
                    field_type_with_serialize_deserialize: quote::quote!{DeleteManyWithBodyPayloadTryFromDeleteManyWithBodyPayloadWithSerializeDeserializeErrorNamedWithSerializeDeserialize},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let delete_many_with_body_payload_try_from_delete_many_with_body_payload_with_serialize_deserialize_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request] 
        DeleteManyWithBodyPayloadTryFromDeleteManyWithBodyPayloadWithSerializeDeserialize {
            #[eo_error_occurence]
            delete_many_with_body_payload_try_from_delete_many_with_body_payload_with_serialize_deserialize: DeleteManyWithBodyPayloadTryFromDeleteManyWithBodyPayloadWithSerializeDeserializeErrorNamed,
            code_occurence: crate::common::code_occurence::CodeOccurence,
        }
    };
    //
    //todo generate it. now its const
    let not_unique_name_vec_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{NotUniqueNameVec},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{not_unique_name_vec},
                    error_occurence_attribute: quote::quote!{#[eo_vec_display_with_serialize_deserialize]},
                    field_type_original: quote::quote!{std::vec::Vec<#crate_server_postgres_regex_filter_regex_filter_token_stream>},
                    field_type_with_serialize_deserialize: quote::quote!{std::vec::Vec<#crate_server_postgres_regex_filter_regex_filter_token_stream>},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let not_unique_name_vec_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request]
        NotUniqueNameVec {
            #[eo_vec_display_with_serialize_deserialize]
            not_unique_name_vec: std::vec::Vec<#crate_server_postgres_regex_filter_regex_filter_token_stream>,//todo there is a std::string::String instead of crate_server_postgres_regex_filter_regex_filter_token_stream cases - refactor that
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let not_unique_color_vec_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{NotUniqueColorVec},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{not_unique_color_vec},
                    error_occurence_attribute: quote::quote!{#[eo_vec_display_with_serialize_deserialize]},
                    field_type_original: quote::quote!{std::vec::Vec<#crate_server_postgres_regex_filter_regex_filter_token_stream>},
                    field_type_with_serialize_deserialize: quote::quote!{std::vec::Vec<#crate_server_postgres_regex_filter_regex_filter_token_stream>},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    //todo use not_unique_name_vec_need_to_refactor_variant_declaration_token_stream instead of not_unique_name_vec_variant_declaration_token_stream
    let not_unique_color_vec_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request]
        NotUniqueColorVec {
            #[eo_vec_display_with_serialize_deserialize]
            not_unique_color_vec: std::vec::Vec<#crate_server_postgres_regex_filter_regex_filter_token_stream>,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    //
    let not_uuid_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{NotUuid},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{not_uuid},
                    error_occurence_attribute: quote::quote!{#[eo_display]},
                    field_type_original: quote::quote!{sqlx::types::uuid::Error},
                    field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let not_uuid_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request]
        NotUuid {
            #[eo_display]
            not_uuid: sqlx::types::uuid::Error,
            code_occurence: crate::common::code_occurence::CodeOccurence,
        }
    };
    //
    let no_payload_fields_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{NoPayloadFields},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{no_payload_fields},
                    error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                    field_type_original: quote::quote!{#std_string_string_token_stream},
                    field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let no_payload_fields_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request]
        NoPayloadFields {
            #eo_display_with_serialize_deserialize_token_stream
            no_payload_fields: #std_string_string_token_stream,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let no_payload_parameters_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
        error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
        error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
            error_variant_ident: quote::quote!{NoPayloadParameters},
            error_variant_fields: vec![
                crate::type_variants_from_request_response_generator::ErrorVariantField {
                    field_name: quote::quote!{no_payload_parameters},
                    error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                    field_type_original: quote::quote!{#std_string_string_token_stream},
                    field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                },
                error_occurence_variant_field.clone(),
            ],
        },
    };
    let no_payload_parameters_variant_declaration_token_stream = quote::quote!{
        #[tvfrr_400_bad_request]
        NoPayloadParameters {
            #eo_display_with_serialize_deserialize_token_stream
            no_payload_parameters: #std_string_string_token_stream,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let query_encode_variant_token_stream = quote::quote!{
        #query_encode_token_stream {
            #eo_display_token_stream
            #url_encoding_token_stream: #serde_urlencoded_ser_error_token_stream,
            #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
        }
    };
    let query_encode_variant_initialization_token_stream = quote::quote!{
        #query_encode_token_stream {
            #url_encoding_token_stream: e,
            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
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
    let not_uuid_camel_case_stringified = "NotUuid";
    let not_uuid_token_camel_case_stream = {
        not_uuid_camel_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {not_uuid_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let not_uuid_token_lower_case_stream = {
        let not_uuid_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&not_uuid_camel_case_stringified.to_string());
        not_uuid_lower_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {not_uuid_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let one_camel_case_stringified = "One";
    let with_body_camel_case_stringified = "WithBody";
    let many_camel_case_stringified = "Many";
    let create_camel_case_stringified = "Create";
    let read_camel_case_stringified = "Read";
    let update_camel_case_stringified = "Update";
    let delete_camel_case_stringified = "Delete";
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
    // let any_name_stringified = "any";
    // let array_name_stringified = "array";
    let select_name_stringified = "select";
    let order_by_name_stringified = "order by";
    let limit_name_stringified = "limit";
    let offset_name_stringified = "offset";
    let in_name_stringified = "in";
    let unnest_name_stringified = "unnest";//
    let common_error_variant_attribute_vec_owned = {
        let mut common_error_variants_vec = std::vec::Vec::new();
        let common_middlewares_error_variants_vec_handle_owned = {
            let project_commit_extractor_not_equal_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{ProjectCommitExtractorNotEqual},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{project_commit_not_equal},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{project_commit_to_use},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let project_commit_extractor_to_str_conversion_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{ProjectCommitExtractorToStrConversion},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{project_commit_to_str_conversion},
                            error_occurence_attribute: quote::quote!{#[eo_display]},
                            field_type_original: quote::quote!{http::header::ToStrError},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let no_project_commit_extractor_header_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{NoProjectCommitExtractorHeader},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{no_project_commit_header},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            vec![
                project_commit_extractor_not_equal_variant_attribute,
                project_commit_extractor_to_str_conversion_variant_attribute,
                no_project_commit_extractor_header_variant_attribute
            ]
        };
        for element in common_middlewares_error_variants_vec_handle_owned {
            common_error_variants_vec.push(element);
        }
        let postgres_error_variants_vec_handle_owned = {
            //todo move it into custom macro attribute
            let configuration_error_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{Configuration},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{configuration_box_dyn_error},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let database_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{Database},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{box_dyn_database_error},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let io_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{Io},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{io_error},
                            error_occurence_attribute: quote::quote!{#[eo_display]},
                            field_type_original: quote::quote!{std::io::Error},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let tls_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{Tls},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{box_dyn_error},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let protocol_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{Protocol},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{protocol},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let row_not_found_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr404NotFound,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{RowNotFound},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{row_not_found},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let type_not_found_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{TypeNotFound},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{type_not_found},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let column_index_out_of_bounds_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{ColumnIndexOutOfBounds},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{column_index_out_of_bounds},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{usize},
                            field_type_with_serialize_deserialize: quote::quote!{usize},
                        },
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{len},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{usize},
                            field_type_with_serialize_deserialize: quote::quote!{usize},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let column_not_found_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{ColumnNotFound},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{column_not_found},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let column_decode_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{ColumnDecode},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{column_decode_index},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{source_handle},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let decode_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{Decode},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{decode_box_dyn_error},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let pool_timed_out_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr408RequestTimeout,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{PoolTimedOut},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{pool_timed_out},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let pool_closed_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{PoolClosed},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{pool_closed},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let worker_crashed_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{WorkerCrashed},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{worker_crashed},
                            error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                            field_type_original: quote::quote!{#std_string_string_token_stream},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            let migrate_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
                error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
                error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                    error_variant_ident: quote::quote!{Migrate},
                    error_variant_fields: vec![
                        crate::type_variants_from_request_response_generator::ErrorVariantField {
                            field_name: quote::quote!{migrate},
                            error_occurence_attribute: quote::quote!{#[eo_display]},
                            field_type_original: quote::quote!{sqlx::migrate::MigrateError},
                            field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                        },
                        error_occurence_variant_field.clone(),
                    ],
                },
            };
            vec![
                configuration_error_variant_attribute,
                database_variant_attribute,
                io_variant_attribute,
                tls_variant_attribute,
                protocol_variant_attribute,
                row_not_found_variant_attribute,
                type_not_found_variant_attribute,
                column_index_out_of_bounds_variant_attribute,
                column_not_found_variant_attribute,
                column_decode_variant_attribute,
                decode_variant_attribute,
                pool_timed_out_variant_attribute,
                pool_closed_variant_attribute,
                worker_crashed_variant_attribute,
                migrate_variant_attribute
            ]
        };
        for element in postgres_error_variants_vec_handle_owned {
            common_error_variants_vec.push(element);
        }
        let unexpected_case_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
            error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
            error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                error_variant_ident: quote::quote!{UnexpectedCase},
                error_variant_fields: vec![
                    crate::type_variants_from_request_response_generator::ErrorVariantField {
                        field_name: quote::quote!{unexpected_case},
                        error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                        field_type_original: quote::quote!{#std_string_string_token_stream},
                        field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                    },
                    error_occurence_variant_field.clone(),
                ],
            },
        };
        common_error_variants_vec.push(unexpected_case_variant_attribute);
        common_error_variants_vec
    };
    let common_error_variant_attribute_vec = common_error_variant_attribute_vec_owned.iter().map(|element|element).collect::<std::vec::Vec<&crate::type_variants_from_request_response_generator::ErrorVariantAttribute>>();
    let path_logic_error_variants_vec_handle_owned = {
        let failed_to_deserialize_path_params_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
            error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
            error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                error_variant_ident: quote::quote!{FailedToDeserializePathParams},
                error_variant_fields: vec![
                    crate::type_variants_from_request_response_generator::ErrorVariantField {
                        field_name: quote::quote!{failed_to_deserialize_path_params},
                        error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                        field_type_original: quote::quote!{#std_string_string_token_stream},
                        field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                    },
                    error_occurence_variant_field.clone(),
                ],
            },
        };
        let missing_path_params_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
            error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
            error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                error_variant_ident: quote::quote!{MissingPathParams},
                error_variant_fields: vec![
                    crate::type_variants_from_request_response_generator::ErrorVariantField {
                        field_name: quote::quote!{missing_path_params},
                        error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                        field_type_original: quote::quote!{#std_string_string_token_stream},
                        field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                    },
                    error_occurence_variant_field.clone(),
                ],
            },
        };
        vec![
            failed_to_deserialize_path_params_variant_attribute,
            missing_path_params_variant_attribute
        ]
    };
    let path_logic_error_variants_vec_handle = path_logic_error_variants_vec_handle_owned.iter().map(|element|element).collect();
    let json_body_logic_error_variants_vec_handle_owned = {
        let json_data_error_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
            error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
            error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                error_variant_ident: quote::quote!{JsonDataError},
                error_variant_fields: vec![
                    crate::type_variants_from_request_response_generator::ErrorVariantField {
                        field_name: quote::quote!{json_data_error},
                        error_occurence_attribute: quote::quote!{#[eo_display]},
                        field_type_original: quote::quote!{axum::extract::rejection::JsonDataError},
                        field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                    },
                    error_occurence_variant_field.clone(),
                ],
            },
        };
        let json_syntax_error_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
            error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
            error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                error_variant_ident: quote::quote!{JsonSyntaxError},
                error_variant_fields: vec![
                    crate::type_variants_from_request_response_generator::ErrorVariantField {
                        field_name: quote::quote!{json_syntax_error},
                        error_occurence_attribute: quote::quote!{#[eo_display]},
                        field_type_original: quote::quote!{axum::extract::rejection::JsonSyntaxError},
                        field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                    },
                    error_occurence_variant_field.clone(),
                ],
            },
        };
        let missing_json_content_type_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
            error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr400BadRequest,
            error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                error_variant_ident: quote::quote!{MissingJsonContentType},
                error_variant_fields: vec![
                    crate::type_variants_from_request_response_generator::ErrorVariantField {
                        field_name: quote::quote!{json_syntax_error},
                        error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                        field_type_original: quote::quote!{#std_string_string_token_stream},
                        field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                    },
                    error_occurence_variant_field.clone(),
                ],
            },
        };
        let bytes_rejection_variant_attribute = crate::type_variants_from_request_response_generator::ErrorVariantAttribute {
            error_variant_attribute: proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError,
            error_variant: crate::type_variants_from_request_response_generator::ErrorVariant {
                error_variant_ident: quote::quote!{BytesRejection},
                error_variant_fields: vec![
                    crate::type_variants_from_request_response_generator::ErrorVariantField {
                        field_name: quote::quote!{bytes_rejection},
                        error_occurence_attribute: quote::quote!{#eo_display_with_serialize_deserialize_token_stream},
                        field_type_original: quote::quote!{#std_string_string_token_stream},
                        field_type_with_serialize_deserialize: quote::quote!{#std_string_string_token_stream},
                    },
                    error_occurence_variant_field.clone(),
                ],
            },
        };
        vec![
            json_data_error_variant_attribute,
            json_syntax_error_variant_attribute,
            missing_json_content_type_variant_attribute,
            bytes_rejection_variant_attribute
        ]
    };
    let json_body_logic_error_variants_vec_handle = json_body_logic_error_variants_vec_handle_owned.iter().map(|element|element).collect();
    let create_many_token_stream = {
        let operation_name_camel_case_stringified = format!("{create_camel_case_stringified}{many_camel_case_stringified}");
        let operation_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&operation_name_camel_case_stringified.to_string());
        let operation_parameters_camel_case_token_stream = generate_operation_parameters_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            parameters_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_element_camel_case_token_stream = generate_operation_payload_element_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            &payload_element_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_camel_case_token_stream = quote::quote!{std::vec::Vec<#operation_payload_element_camel_case_token_stream>};
        let try_operation_error_named_camel_case_token_stream = generate_try_operation_error_named_camel_case_token_stream(
            try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_response_variants_token_stream = generate_try_operation_response_variants_token_stream(
            try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            response_variants_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_camel_case_stringified = generate_try_operation_camel_case_stringified(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
        );
        let try_operation_camel_case_token_stream = generate_try_operation_camel_case_token_stream(
            &try_operation_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #operation_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #operation_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let payload_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element| {
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    }); 
                let field_type = &element.field.ty;
                quote::quote!{
                    pub #field_ident: #field_type
                }
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #operation_payload_element_camel_case_token_stream {
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let try_operation_error_named_token_stream = {
            let try_operation_request_error_camel_case_token_stream = generate_try_operation_request_error_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_operation_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_operation_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                        #created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_camel_case_token_stream {
                            #eo_vec_error_occurence_token_stream
                            #created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_lower_case_token_stream: std::vec::Vec<#created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_error_unnamed_camel_case_token_stream>,
                            #code_occurence_lower_case_token_stream: #crate_common_code_occurence_code_occurence_token_stream,
                        },
                }
            }
        };
        // println!("{try_operation_error_named_token_stream}");
        let created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_error_unnamed_token_stream = {
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_error_unnamed_camel_case_token_stream {
                    #created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_camel_case_token_stream(
                        #crate_server_postgres_uuid_wrapper_uuid_wrapper_try_from_possible_uuid_wrapper_error_named_token_stream
                    ),
                }
            }
        };
        // println!("{created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_error_unnamed_token_stream}");
        let try_operation_error_with_middleware_error_variants_token_stream = {
            let desirable_attribute = proc_macro_helpers::attribute::Attribute::Tvfrr201Created;
            let try_operation_response_variants_camel_case_stringified = generate_try_operation_response_variants_camel_case_stringified(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified
            );
            let try_operation_response_variants_camel_case_token_stream = generate_try_operation_response_variants_camel_case_token_stream(
                &try_operation_response_variants_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_camel_case_token_stream = generate_try_operation_with_serialize_deserialize_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_response_variants_desirable_attribute_token_stream = generate_try_operation_response_variants_desirable_attribute_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified,
                &desirable_attribute,
                &proc_macro_name_ident_stringified
            );
            let try_operation_request_error_token_stream = generate_try_operation_request_error_token_stream(
                &try_operation_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_token_stream = generate_try_operation_with_serialize_deserialize_token_stream(
                &try_operation_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let common_error_variant_attribute_vec_handle = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &common_error_variant_attribute_vec
            );
            let json_body_logic_error_variants_vec_token_stream = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &json_body_logic_error_variants_vec_handle
            );
            let type_variants_from_request_response_vec = {
                let mut type_variants_from_request_response = std::vec::Vec::new();//todo calculate capacity
                for element in common_error_variant_attribute_vec_handle {
                    type_variants_from_request_response.push(element);
                }
                for element in json_body_logic_error_variants_vec_token_stream  {
                    type_variants_from_request_response.push(element);
                }
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &bind_query_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_server_variant_attribute
                ));
                type_variants_from_request_response
            };
            crate::type_variants_from_request_response_generator::type_variants_from_request_response_generator(
                desirable_attribute,
                &desirable_token_stream,
                &quote::quote!{std::vec::Vec::<#crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream>},
                &try_operation_camel_case_token_stream,
                &try_operation_response_variants_camel_case_stringified,
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_response_variants_desirable_attribute_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream,
                &try_operation_request_error_token_stream,
                &try_operation_with_serialize_deserialize_token_stream,
                &operation_name_lower_case_stringified,
                &code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                &code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                &error_named_derive_token_stream,
                &eo_display_token_stream,
                &eo_display_foreign_type_token_stream,
                &eo_display_with_serialize_deserialize_token_stream,
                &derive_debug_serialize_deserialize_token_stream,
                type_variants_from_request_response_vec,
                true,
                &proc_macro_name_ident_stringified,
            )
        };
        // println!("{try_operation_error_with_middleware_error_variants_token_stream}");
        let http_request_token_stream = {
            let try_operation_lower_case_token_stream = {
                let try_operation_lower_case_stringified = format!("{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
                try_operation_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let url_handle_token_stream = {
                let url_handle_stringified = format!("\"{{}}/{table_name_stringified}/{batch_stringified}\"");//todo reuse
                url_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {url_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_operation_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #operation_parameters_camel_case_token_stream,
                ) -> Result<std::vec::Vec<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>, #try_operation_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_operation_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
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
                        Ok(value) => {
                            let mut vec_values = std::vec::Vec::with_capacity(value.len());
                            let mut vec_errors = std::vec::Vec::with_capacity(value.len());
                            for element in value {
                                match #crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream::try_from(element) {
                                    Ok(value) => {
                                        vec_values.push(value);
                                    }
                                    Err(e) => {
                                        vec_errors.push(
                                            #created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_error_unnamed_camel_case_token_stream::#created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_camel_case_token_stream(e)
                                        );
                                    }
                                }
                            }
                            if let false = vec_errors.is_empty() {
                                return Err(#try_operation_error_named_camel_case_token_stream::#created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_camel_case_token_stream {
                                    #created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_lower_case_token_stream: vec_errors,
                                    #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream
                                });
                            }
                            Ok(vec_values)
                        },
                        Err(e) => Err(#try_operation_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let operation_lower_case_token_stream = operation_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {operation_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let try_operation_token_stream = {
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &try_operation_camel_case_token_stream,
                    &error_log_call_token_stream,
                    &try_operation_response_variants_token_stream,
                );
                let query_string_token_stream = {
                    let column_names = {
                        let fields_named_filtered = fields_named_wrappers_excluding_primary_key.iter()
                        .map(|element|&element.field)
                        .collect::<std::vec::Vec<&syn::Field>>();
                        fields_named_filtered.iter().enumerate().fold(std::string::String::default(), |mut acc, (index, field)| {
                            let field_ident = field.ident.clone().unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                            let incremented_index = index.checked_add(1).unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} {index} {}", proc_macro_helpers::global_variables::hardcode::CHECKED_ADD_NONE_OVERFLOW_MESSAGE));
                            match incremented_index == fields_named_wrappers_excluding_primary_key_len {
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
                        let mut column_increments = fields_named_wrappers_excluding_primary_key.iter()
                        .map(|element|&element.field)
                        .collect::<std::vec::Vec<&syn::Field>>()
                        .iter()
                        .enumerate().fold(std::string::String::default(), |mut acc, (index, _)| {
                            acc.push_str(&format!("${}, ", index.checked_add(1).unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} {index} {}", proc_macro_helpers::global_variables::hardcode::CHECKED_ADD_NONE_OVERFLOW_MESSAGE))));
                            acc
                        });
                        column_increments.pop();
                        column_increments.pop();
                        column_increments
                    };
                    let query_stringified = format!(
                        "\"{insert_name_stringified} {into_name_stringified} {table_name_stringified} ({column_names}) {select_name_stringified} {column_names} {from_name_stringified} {unnest_name_stringified}({column_increments}) {as_name_stringified} a({column_names}){returning_id_stringified}\""
                    );
                    query_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                // println!("{query_string_token_stream}");
                let binded_query_token_stream = {
                    let column_vecs_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element| {
                        let field_ident_underscore_vec_stringified = {
                            let field_ident = element.field.ident.clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                });
                            format!("{field_ident}{underscore_vec_name_stringified}")
                        };
                        field_ident_underscore_vec_stringified.parse::<proc_macro2::TokenStream>()
                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_underscore_vec_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                    });
                    let handle_fields_named_wrappers_excluding_primary_key = fields_named_wrappers_excluding_primary_key.iter()
                    .map(|element|&element.field)
                    .collect::<std::vec::Vec<&syn::Field>>();
                    let column_vecs_with_capacity_token_stream = handle_fields_named_wrappers_excluding_primary_key.iter().map(|_|quote::quote!{std::vec::Vec::with_capacity(#current_vec_len_name_token_stream)});
                    let columns_acc_push_elements_token_stream = handle_fields_named_wrappers_excluding_primary_key.iter()
                    .enumerate().map(|(index, field)|{
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
                    let column_query_bind_vecs_token_stream = handle_fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                        let field_ident_underscore_vec_token_stream = {
                            let field_ident_underscore_vec_stringified = {
                                let field_ident = element.ident.clone()
                                    .unwrap_or_else(|| {
                                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                    });
                                format!("{field_ident}{underscore_vec_name_stringified}")
                            };
                            field_ident_underscore_vec_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_underscore_vec_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{#query_name_token_stream = #query_name_token_stream.bind(#field_ident_underscore_vec_token_stream);}
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
                // crate::generate_postgres_execute_query::generate_postgres_execute_query(
                //     &query_string_name_token_stream,
                //     &query_string_token_stream,
                //     &binded_query_name_token_stream,
                //     &binded_query_token_stream,
                //     &acquire_pool_and_connection_token_stream,
                //     &pg_connection_token_stream,
                //     &try_operation_response_variants_token_stream,
                //     &desirable_token_stream,
                //     &from_log_and_return_error_token_stream,
                // )
                quote::quote! {
                    let #query_string_name_token_stream = {
                        #query_string_token_stream
                    };
                    println!("{}", #query_string_name_token_stream);
                    let #binded_query_name_token_stream = {
                        #binded_query_token_stream
                    };
                    #acquire_pool_and_connection_token_stream
                    let mut rows = #binded_query_name_token_stream.fetch(#pg_connection_token_stream.as_mut());
                    let mut vec_values = std::vec::Vec::new();
                    while let Some(row) = {
                        match {
                            use futures::TryStreamExt;
                            rows.try_next()
                        }
                        .await
                        {
                            Ok(value) => value,
                            Err(e) => {
                                let error = #try_operation_camel_case_token_stream::from(e);
                                #error_log_call_token_stream
                                return #try_operation_response_variants_token_stream::from(error);
                            }
                        }
                    } {
                        match {
                            use #sqlx_row_token_stream;
                            row.try_get::<#sqlx_types_uuid_token_stream, &str>(#id_field_ident_quotes_token_stream)
                        } {
                            Ok(value) => {
                                vec_values.push(
                                    #crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream::from(value),
                                );
                            }
                            Err(e) => {
                                let error = #try_operation_camel_case_token_stream::#created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_server_camel_case_token_stream {
                                    uuid_wrapper_try_from_possible_uuid_wrapper_in_server: e,
                                    #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                };
                                #error_log_call_token_stream
                                return #try_operation_response_variants_token_stream::from(error);
                            }
                        }
                    }
                    #try_operation_response_variants_token_stream::#desirable_token_stream(vec_values)
                }
            };
            // println!("{try_operation_token_stream}");
            quote::quote!{
                pub async fn #operation_lower_case_token_stream(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#operation_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #operation_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #operation_payload_camel_case_token_stream,
                            #try_operation_response_variants_token_stream,
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
                        #try_operation_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #try_operation_error_named_token_stream
            #created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_error_unnamed_token_stream
            #try_operation_error_with_middleware_error_variants_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // proc_macro_helpers::write_token_stream_into_file::write_token_stream_into_file(
    //     &proc_macro_name,
    //     &create_many_token_stream,
    //     &proc_macro_name_ident_stringified
    // );
    let create_one_token_stream = {
        let operation_name_camel_case_stringified = format!("{create_camel_case_stringified}{one_camel_case_stringified}");
        let operation_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&operation_name_camel_case_stringified.to_string());
        let operation_parameters_camel_case_token_stream = generate_operation_parameters_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            parameters_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_camel_case_token_stream = generate_operation_payload_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            payload_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_error_named_camel_case_token_stream = generate_try_operation_error_named_camel_case_token_stream(
            try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_response_variants_token_stream = generate_try_operation_response_variants_token_stream(
            try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            response_variants_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_camel_case_stringified = generate_try_operation_camel_case_stringified(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
        );
        let try_operation_camel_case_token_stream = generate_try_operation_camel_case_token_stream(
            &try_operation_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #operation_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #operation_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let payload_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element| {
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                let field_type = &element.field.ty;
                quote::quote!{
                    pub #field_ident: #field_type
                }
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #operation_payload_camel_case_token_stream {
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let try_operation_error_named_token_stream = {
            let try_operation_request_error_camel_case_token_stream = generate_try_operation_request_error_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_operation_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_operation_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                    #created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        uuid_wrapper_try_from_possible_uuid_wrapper_in_client: #crate_server_postgres_uuid_wrapper_uuid_wrapper_try_from_possible_uuid_wrapper_error_named_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{try_operation_error_named_token_stream}");
        let try_operation_error_with_middleware_error_variants_token_stream = {
            let desirable_attribute = proc_macro_helpers::attribute::Attribute::Tvfrr201Created;
            let try_operation_response_variants_camel_case_stringified = generate_try_operation_response_variants_camel_case_stringified(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified
            );
            let try_operation_response_variants_camel_case_token_stream = generate_try_operation_response_variants_camel_case_token_stream(
                &try_operation_response_variants_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_camel_case_token_stream = generate_try_operation_with_serialize_deserialize_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_response_variants_desirable_attribute_token_stream = generate_try_operation_response_variants_desirable_attribute_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified,
                &desirable_attribute,
                &proc_macro_name_ident_stringified
            );
            let try_operation_request_error_token_stream = generate_try_operation_request_error_token_stream(
                &try_operation_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_token_stream = generate_try_operation_with_serialize_deserialize_token_stream(
                &try_operation_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let common_error_variant_attribute_vec_handle = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &common_error_variant_attribute_vec
            );
            let json_body_logic_error_variants_vec_token_stream = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &json_body_logic_error_variants_vec_handle
            );
            let type_variants_from_request_response_vec = {
                let mut type_variants_from_request_response = std::vec::Vec::new();
                for element in common_error_variant_attribute_vec_handle {
                    type_variants_from_request_response.push(element);
                }
                for element in json_body_logic_error_variants_vec_token_stream  {
                    type_variants_from_request_response.push(element);
                }
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_server_variant_attribute
                ));
                type_variants_from_request_response
            };
            crate::type_variants_from_request_response_generator::type_variants_from_request_response_generator(
                desirable_attribute,
                &desirable_token_stream,
                &crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream,
                &try_operation_camel_case_token_stream,
                &try_operation_response_variants_camel_case_stringified,
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_response_variants_desirable_attribute_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream,
                &try_operation_request_error_token_stream,
                &try_operation_with_serialize_deserialize_token_stream,
                &operation_name_lower_case_stringified,
                &code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                &code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                &error_named_derive_token_stream,
                &eo_display_token_stream,
                &eo_display_foreign_type_token_stream,
                &eo_display_with_serialize_deserialize_token_stream,
                &derive_debug_serialize_deserialize_token_stream,
                type_variants_from_request_response_vec,
                true,
                &proc_macro_name_ident_stringified,
            )
        };
        // println!("{try_operation_error_with_middleware_error_variants_token_stream}");
        let http_request_token_stream = {
            let try_operation_lower_case_token_stream = {
                let try_operation_lower_case_stringified = format!("{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
                try_operation_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
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
                pub async fn #try_operation_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #operation_parameters_camel_case_token_stream,
                ) -> Result<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream, #try_operation_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_operation_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
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
                        Ok(value) => match #crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream::try_from(value) {
                            Ok(value) => Ok(value),
                            Err(e) => Err(#try_operation_error_named_camel_case_token_stream::#created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_client_camel_case_token_stream {
                                uuid_wrapper_try_from_possible_uuid_wrapper_in_client: e,
                                #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                            })
                        },
                        Err(e) => Err(#try_operation_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let operation_lower_case_token_stream = operation_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {operation_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let try_operation_token_stream = {
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &try_operation_camel_case_token_stream,
                    &error_log_call_token_stream,
                    &try_operation_response_variants_token_stream,
                );
                let query_string_token_stream = {
                    let (
                        column_names,
                        column_increments
                    ) = {
                        let fields_named_filtered = fields_named_wrappers_excluding_primary_key.iter().map(|element|&element.field).collect::<std::vec::Vec<&syn::Field>>();
                        fields_named_filtered.iter().enumerate().fold((
                            std::string::String::default(),
                            std::string::String::default()
                        ), |mut acc, (index, field)| {
                            let field_ident = field.ident.clone().unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                            let incremented_index = index.checked_add(1).unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} {index} {}", proc_macro_helpers::global_variables::hardcode::CHECKED_ADD_NONE_OVERFLOW_MESSAGE));
                            match incremented_index == fields_named_wrappers_excluding_primary_key_len {
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
                    let query_stringified = format!("\"{insert_name_stringified} {into_name_stringified} {table_name_stringified}({column_names}) {values_name_stringified} ({column_increments}) {returning_id_stringified}\"");
                    query_stringified.parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {query_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                // println!("{query_string_token_stream}");
                let binded_query_token_stream = {
                    let binded_query_modifications_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                        let field_ident = element.field.ident.clone()
                            .unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                        quote::quote!{
                            query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(#parameters_lower_case_token_stream.#payload_lower_case_token_stream.#field_ident, query);
                        }
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
                // crate::generate_postgres_execute_query::generate_postgres_execute_query(
                //     &query_string_name_token_stream,
                //     &query_string_token_stream,
                //     &binded_query_name_token_stream,
                //     &binded_query_token_stream,
                //     &acquire_pool_and_connection_token_stream,
                //     &pg_connection_token_stream,
                //     &try_operation_response_variants_token_stream,
                //     &desirable_token_stream,
                //     &from_log_and_return_error_token_stream,
                // )
                quote::quote! {
                    let #query_string_name_token_stream = {
                        #query_string_token_stream
                    };
                    println!("{}", #query_string_name_token_stream);
                    let #binded_query_name_token_stream = {
                        #binded_query_token_stream
                    };
                    #acquire_pool_and_connection_token_stream
                    match #binded_query_name_token_stream.fetch_one(#pg_connection_token_stream.as_mut()).await {
                        Ok(value) => match {
                            use #sqlx_row_token_stream;
                            value.try_get::<#sqlx_types_uuid_token_stream, &str>(#id_field_ident_quotes_token_stream)
                        } {
                            Ok(value) => #try_operation_response_variants_token_stream::#desirable_token_stream(#crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream::from(value)),
                            Err(e) => {
                                let error = #try_operation_camel_case_token_stream::#created_but_cannot_convert_uuid_wrapper_from_possible_uuid_wrapper_in_server_camel_case_token_stream {
                                    uuid_wrapper_try_from_possible_uuid_wrapper_in_server: e,
                                    #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                };
                                #error_log_call_token_stream
                                return #try_operation_response_variants_token_stream::from(error);
                            }
                        },
                        Err(e) => {
                            #from_log_and_return_error_token_stream
                        }
                    }
                }
            };
            // println!("{try_operation_token_stream}");
            quote::quote!{
                pub async fn #operation_lower_case_token_stream(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#operation_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #operation_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #operation_payload_camel_case_token_stream,
                            #try_operation_response_variants_token_stream,
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
                        #try_operation_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #try_operation_error_named_token_stream
            #try_operation_error_with_middleware_error_variants_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // proc_macro_helpers::write_token_stream_into_file::write_token_stream_into_file(
    //     &proc_macro_name,
    //     &create_one_token_stream,
    //     &proc_macro_name_ident_stringified
    // );
    let read_one_token_stream = {
        let operation_name_camel_case_stringified = format!("{read_camel_case_stringified}{one_camel_case_stringified}");
        let operation_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&operation_name_camel_case_stringified.to_string());
        let operation_parameters_camel_case_token_stream = generate_operation_parameters_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            parameters_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_camel_case_token_stream = generate_path_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            path_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_with_serialize_deserialize_camel_case_token_stream = generate_path_with_serialize_deserialize_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            path_camel_case_stringified,
            with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_stringified = generate_path_try_from_path_with_serialize_deserialize_stringified(
            &operation_name_camel_case_stringified,
            path_camel_case_stringified,
            &try_from_camel_case_stringified,
            with_serialize_deserialize_camel_case_stringified,
        );
        let operation_path_try_from_operation_path_with_serialize_deserialize_error_named_camel_case_token_stream = generate_path_try_from_path_with_serialize_deserialize_error_named_token_stream(
            &operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_stringified,
            error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_token_stream = generate_path_try_from_path_with_serialize_deserialize_camel_case_token_stream(
            &operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_try_from_operation_path_with_serialize_deserialize_lower_case_token_stream = generate_path_try_from_path_with_serialize_deserialize_lower_case_token_stream(
            &operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_query_camel_case_token_stream = generate_query_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            query_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_query_with_serialize_deserialize_camel_case_token_stream = generate_query_with_serialize_deserialize_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            query_camel_case_stringified,
            with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_error_named_camel_case_token_stream = generate_try_operation_error_named_camel_case_token_stream(
            try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_response_variants_token_stream = generate_try_operation_response_variants_token_stream(
            try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            response_variants_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_camel_case_stringified = generate_try_operation_camel_case_stringified(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
        );
        let try_operation_camel_case_token_stream = generate_try_operation_camel_case_token_stream(
            &try_operation_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #operation_parameters_camel_case_token_stream {
                    pub #path_lower_case_token_stream: #operation_path_camel_case_token_stream,
                    pub #query_lower_case_token_stream: #operation_query_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let path_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #operation_path_camel_case_token_stream {
                    pub #id_field_ident: #crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream,
                }
            }
        };
        // println!("{path_token_stream}");
        let path_with_serialize_deserialize_token_stream = {
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #operation_path_with_serialize_deserialize_camel_case_token_stream {
                    #id_field_ident: #crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream,
                }
            }
        };
        // println!("{path_with_serialize_deserialize_token_stream}");
        let operation_path_try_from_operation_path_with_serialize_deserialize_error_named_token_stream = {
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #operation_path_try_from_operation_path_with_serialize_deserialize_error_named_camel_case_token_stream {
                    #not_uuid_token_camel_case_stream {
                        #eo_error_occurence_attribute_token_stream
                        #not_uuid_token_lower_case_stream: #crate_server_postgres_uuid_wrapper_uuid_wrapper_try_from_possible_uuid_wrapper_error_named_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                } 
            }
        };
        // println!("{operation_path_try_from_operation_path_with_serialize_deserialize_error_named_token_stream}");
        let impl_std_convert_try_from_operation_path_with_serialize_deserialize_for_operation_path_token_stream = {
            quote::quote!{
                impl std::convert::TryFrom<#operation_path_with_serialize_deserialize_camel_case_token_stream> for #operation_path_camel_case_token_stream {
                    type Error = #operation_path_try_from_operation_path_with_serialize_deserialize_error_named_camel_case_token_stream;
                    fn try_from(value: #operation_path_with_serialize_deserialize_camel_case_token_stream) -> Result<Self, Self::Error> {
                        let #id_field_ident = match #crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream::try_from(value.#id_field_ident) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(Self::Error::#not_uuid_token_camel_case_stream {
                                    #not_uuid_token_lower_case_stream: e,
                                    #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
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
        // println!("{impl_std_convert_try_from_operation_path_with_serialize_deserialize_for_operation_path_token_stream}");
        let query_token_stream = {
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #operation_query_camel_case_token_stream {
                    pub #select_token_stream: std::option::Option<#column_select_ident_token_stream>,
                }
            }
        };
        // println!("{query_token_stream}");
        let query_with_serialize_deserialize_token_stream = {
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                struct #operation_query_with_serialize_deserialize_camel_case_token_stream {
                    #select_token_stream: std::option::Option<#std_string_string_token_stream>,
                } 
            }
        };
        // println!("{query_with_serialize_deserialize_token_stream}");
        let into_url_encoding_version_token_stream = {
            quote::quote!{
                impl #operation_query_camel_case_token_stream {
                    fn #into_url_encoding_version_name_token_stream(self) -> #operation_query_with_serialize_deserialize_camel_case_token_stream {
                        let #select_token_stream = self.#select_token_stream.map(|value| {
                            #crate_common_serde_urlencoded_serde_urlencoded_parameter_serde_urlencoded_parameter_token_stream(
                                value,
                            )
                        });
                        #operation_query_with_serialize_deserialize_camel_case_token_stream {
                            #select_token_stream
                        }
                    }
                }
            }
        };
        // println!("{into_url_encoding_version_token_stream}");
        let try_operation_error_named_token_stream = {
            let try_operation_request_error_camel_case_token_stream = generate_try_operation_request_error_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_operation_error_named_camel_case_token_stream {
                    #query_encode_variant_token_stream,
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_operation_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{try_operation_error_named_token_stream}");
        let try_operation_error_with_middleware_error_variants_token_stream = {
            let desirable_attribute = proc_macro_helpers::attribute::Attribute::Tvfrr200Ok;
            let try_operation_response_variants_camel_case_stringified = generate_try_operation_response_variants_camel_case_stringified(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified
            );
            let try_operation_response_variants_camel_case_token_stream = generate_try_operation_response_variants_camel_case_token_stream(
                &try_operation_response_variants_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_camel_case_token_stream = generate_try_operation_with_serialize_deserialize_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_response_variants_desirable_attribute_token_stream = generate_try_operation_response_variants_desirable_attribute_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified,
                &desirable_attribute,
                &proc_macro_name_ident_stringified
            );
            let try_operation_request_error_token_stream = generate_try_operation_request_error_token_stream(
                &try_operation_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_token_stream = generate_try_operation_with_serialize_deserialize_token_stream(
                &try_operation_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let common_error_variant_attribute_vec_handle = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &common_error_variant_attribute_vec
            );
            let path_logic_error_variants_vec_token_stream = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &path_logic_error_variants_vec_handle
            );
            let type_variants_from_request_response_vec = {
                let mut type_variants_from_request_response = std::vec::Vec::new();//todo calculate capacity
                for element in common_error_variant_attribute_vec_handle {
                    type_variants_from_request_response.push(element);
                }
                for element in path_logic_error_variants_vec_token_stream  {
                    type_variants_from_request_response.push(element);
                }
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &failed_to_deserialize_query_string_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &read_one_path_try_from_read_one_path_with_serialize_deserialize_variant_attribute
                ));
                type_variants_from_request_response
            };
            crate::type_variants_from_request_response_generator::type_variants_from_request_response_generator(
                desirable_attribute,
                &desirable_token_stream,
                &quote::quote!{#struct_options_ident_token_stream},
                &try_operation_camel_case_token_stream,
                &try_operation_response_variants_camel_case_stringified,
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_response_variants_desirable_attribute_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream,
                &try_operation_request_error_token_stream,
                &try_operation_with_serialize_deserialize_token_stream,
                &operation_name_lower_case_stringified,
                &code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                &code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                &error_named_derive_token_stream,
                &eo_display_token_stream,
                &eo_display_foreign_type_token_stream,
                &eo_display_with_serialize_deserialize_token_stream,
                &derive_debug_serialize_deserialize_token_stream,
                type_variants_from_request_response_vec,
                true,
                &proc_macro_name_ident_stringified,
            )
        };
        // println!("{try_operation_error_with_middleware_error_variants_token_stream}");
        let http_request_token_stream = {
            let try_operation_lower_case_token_stream = {
                let try_operation_lower_case_stringified = format!("{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
                try_operation_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
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
                pub async fn #try_operation_lower_case_token_stream(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #operation_parameters_camel_case_token_stream,
                ) -> Result<
                    #struct_options_ident_token_stream,
                    #try_operation_error_named_camel_case_token_stream,
                > {
                    let encoded_query = match #serde_urlencoded_to_string_token_stream(#parameters_lower_case_token_stream.#query_lower_case_token_stream.#into_url_encoding_version_name_token_stream()) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_operation_error_named_camel_case_token_stream::#query_encode_variant_initialization_token_stream);
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
                        Err(e) => Err(#try_operation_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let operation_lower_case_token_stream = operation_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {operation_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let try_operation_token_stream = {
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &try_operation_camel_case_token_stream,
                    &error_log_call_token_stream,
                    &try_operation_response_variants_token_stream,
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
                            Ok(value) => #try_operation_response_variants_token_stream::#desirable_token_stream(value),
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
            // println!("{try_operation_token_stream}");
            quote::quote!{
                pub async fn #operation_lower_case_token_stream(
                    #path_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_path_token_stream<#operation_path_with_serialize_deserialize_camel_case_token_stream>,
                        #axum_extract_rejection_path_rejection_token_stream,
                    >,
                    #query_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_query_token_stream<#operation_query_camel_case_token_stream>,
                        #axum_extract_rejection_query_rejection_token_stream,
                    >,
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #operation_parameters_camel_case_token_stream {
                        #path_lower_case_token_stream: match #crate_server_routes_helpers_path_extractor_error_path_value_result_extractor_token_stream::<
                            #operation_path_with_serialize_deserialize_camel_case_token_stream,
                            #try_operation_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#path_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => match #operation_path_camel_case_token_stream::try_from(value) {
                                Ok(value) => value,
                                Err(e) => {
                                    let error = #try_operation_camel_case_token_stream::#operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_token_stream {
                                        #operation_path_try_from_operation_path_with_serialize_deserialize_lower_case_token_stream: e,
                                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                    };
                                    #error_log_call_token_stream
                                    return #try_operation_response_variants_token_stream::from(error);
                                }
                            },
                            Err(err) => {
                                return err;
                            }
                        },
                        #query_lower_case_token_stream: match #crate_server_routes_helpers_query_extractor_error_query_value_result_extractor_token_stream::<
                            #operation_query_camel_case_token_stream,
                            #try_operation_response_variants_token_stream,
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
                        #try_operation_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #path_token_stream
            #path_with_serialize_deserialize_token_stream
            #operation_path_try_from_operation_path_with_serialize_deserialize_error_named_token_stream
            #impl_std_convert_try_from_operation_path_with_serialize_deserialize_for_operation_path_token_stream
            #query_token_stream
            #query_with_serialize_deserialize_token_stream
            #into_url_encoding_version_token_stream
            #try_operation_error_named_token_stream
            #try_operation_error_with_middleware_error_variants_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // proc_macro_helpers::write_token_stream_into_file::write_token_stream_into_file(
    //     &proc_macro_name,
    //     &read_one_token_stream,
    //     &proc_macro_name_ident_stringified
    // );
    let read_many_with_body_token_stream = {
        let operation_name_camel_case_stringified = format!("{read_camel_case_stringified}{many_camel_case_stringified}{with_body_camel_case_stringified}");
        let operation_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&operation_name_camel_case_stringified.to_string());
        let operation_parameters_camel_case_token_stream = generate_operation_parameters_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            parameters_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_camel_case_token_stream = generate_operation_payload_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            payload_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_with_serialize_deserialize_camel_case_token_stream = generate_payload_with_serialize_deserialize_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            payload_camel_case_stringified,
            with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_try_from_operation_payload_with_serialize_deserialize_camel_case_stringified = generate_payload_try_from_payload_with_serialize_deserialize_stringified(
            &operation_name_camel_case_stringified,
            payload_camel_case_stringified,
            &try_from_camel_case_stringified,
            with_serialize_deserialize_camel_case_stringified,
        );
        let operation_payload_try_from_operation_payload_with_serialize_deserialize_error_named_camel_case_token_stream = generate_payload_try_from_payload_with_serialize_deserialize_error_named_camel_case_token_stream(
            &operation_payload_try_from_operation_payload_with_serialize_deserialize_camel_case_stringified,
            error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_try_from_operation_payload_with_serialize_deserialize_camel_case_token_stream = generate_payload_try_from_payload_with_serialize_deserialize_camel_case_token_stream(
            &operation_payload_try_from_operation_payload_with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_try_from_operation_payload_with_serialize_deserialize_lower_case_token_stream = generate_payload_try_from_payload_with_serialize_deserialize_lower_case_token_stream(
            &operation_payload_try_from_operation_payload_with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_error_named_camel_case_token_stream = generate_try_operation_error_named_camel_case_token_stream(
            try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_response_variants_token_stream = generate_try_operation_response_variants_token_stream(
            try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            response_variants_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_camel_case_stringified = generate_try_operation_camel_case_stringified(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
        );
        let try_operation_camel_case_token_stream = generate_try_operation_camel_case_token_stream(
            &try_operation_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let parameters_token_stream = {
            quote::quote!{
                #[derive(Debug)]
                pub struct #operation_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #operation_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let payload_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    pub #field_ident: std::option::Option<std::vec::Vec<#crate_server_postgres_regex_filter_regex_filter_token_stream>>,
                }
            });
            quote::quote!{
                #derive_debug_token_stream
                pub struct #operation_payload_camel_case_token_stream {
                    pub #select_token_stream: #column_select_ident_token_stream,
                    pub #id_field_ident: std::option::Option<std::vec::Vec<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>>,
                    #(#fields_with_excluded_id_token_stream)*
                    pub #order_by_token_stream: #crate_server_postgres_order_by_order_by_token_stream<#column_ident_token_stream>,
                    pub limit: #crate_server_postgres_postgres_bigint_postgres_bigint_token_stream,
                    pub offset: #crate_server_postgres_postgres_bigint_postgres_bigint_token_stream,
                }
            }
        };
        // println!("{payload_token_stream}");
        let payload_with_serialize_deserialize_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    pub #field_ident: std::option::Option<std::vec::Vec<#crate_server_postgres_regex_filter_regex_filter_token_stream>>,
                }
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #operation_payload_with_serialize_deserialize_camel_case_token_stream {
                    pub select: #column_select_ident_token_stream,
                    pub #id_field_ident: std::option::Option<std::vec::Vec<#std_string_string_token_stream>>,
                    #(#fields_with_excluded_id_token_stream)*
                    pub order_by: #crate_server_postgres_order_by_order_by_token_stream<#column_ident_token_stream>,
                    pub limit: #crate_server_postgres_postgres_bigint_postgres_bigint_token_stream,
                    pub offset: #crate_server_postgres_postgres_bigint_postgres_bigint_token_stream,
                }    
            }
        };
        // println!("{payload_with_serialize_deserialize_token_stream}");
        let operation_payload_try_from_operation_payload_with_serialize_deserialize_error_named_token_stream = {
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #operation_payload_try_from_operation_payload_with_serialize_deserialize_error_named_camel_case_token_stream {
                    #not_uuid_token_camel_case_stream {
                        #eo_error_occurence_attribute_token_stream
                        #not_uuid_token_lower_case_stream: #crate_server_postgres_uuid_wrapper_uuid_wrapper_try_from_possible_uuid_wrapper_error_named_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{operation_payload_try_from_operation_payload_with_serialize_deserialize_error_named_token_stream}");
        let impl_std_convert_try_from_operation_payload_with_serialize_deserialize_for_operation_payload_token_stream = {
            let primary_key_field_assignment_token_stream = {
                quote::quote!{
                    let #id_field_ident = match value.#id_field_ident {
                        Some(value) => match value.into_iter()
                            .map(|element|#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream::try_from(#crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream::from(element)))
                            .collect::<Result<
                                std::vec::Vec<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>,
                                #crate_server_postgres_uuid_wrapper_uuid_wrapper_try_from_possible_uuid_wrapper_error_named_token_stream
                            >>() 
                        {
                            Ok(value) => Some(value),
                            Err(e) => {
                                return Err(Self::Error::#not_uuid_token_camel_case_stream {
                                    #not_uuid_token_lower_case_stream: e,
                                    #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                });
                            }
                        },
                        None => None
                    };
                }
            };
            let fields_assignment_excluding_primary_key_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    let #field_ident = value.#field_ident;
                }
            });
            quote::quote!{
                impl std::convert::TryFrom<#operation_payload_with_serialize_deserialize_camel_case_token_stream> for #operation_payload_camel_case_token_stream {
                    type Error = #operation_payload_try_from_operation_payload_with_serialize_deserialize_error_named_camel_case_token_stream;
                    fn try_from(value: #operation_payload_with_serialize_deserialize_camel_case_token_stream) -> Result<Self, Self::Error> {
                        let select = value.select;
                        #primary_key_field_assignment_token_stream
                        #(#fields_assignment_excluding_primary_key_token_stream)*
                        let order_by = value.order_by;
                        let limit = value.limit;
                        let offset = value.offset;
                        Ok(Self {
                            select,
                            id,
                            name,
                            color,
                            order_by,
                            limit,
                            offset,
                        })
                    }
                }
            }
        };
        // println!("{impl_std_convert_try_from_operation_payload_with_serialize_deserialize_for_operation_payload_token_stream}");
        let impl_std_convert_from_operation_payload_for_operation_payload_with_serialize_deserialize_token_stream = {
            let primary_key_field_assignment_token_stream = {
                quote::quote!{
                    let #id_field_ident = match value.#id_field_ident {
                        Some(value) => Some(value.into_iter().map(|element|element.to_string()).collect::<std::vec::Vec<#std_string_string_token_stream>>()),
                        None => None
                    };
                }
            };
            let fields_assignment_excluding_primary_key_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    let #field_ident = value.#field_ident;
                }
            });
            quote::quote!{
                impl std::convert::From<#operation_payload_camel_case_token_stream> for #operation_payload_with_serialize_deserialize_camel_case_token_stream {
                    fn from(value: #operation_payload_camel_case_token_stream) -> Self {
                        let select = value.select;
                        #primary_key_field_assignment_token_stream
                        #(#fields_assignment_excluding_primary_key_token_stream)*
                        let order_by = value.order_by;
                        let limit = value.limit;
                        let offset = value.offset;
                        Self{
                            select,
                            id,
                            name,
                            color,
                            order_by,
                            limit,
                            offset,
                        }
                    }
                }
            }
        };
        // println!("{impl_std_convert_from_operation_payload_for_operation_payload_with_serialize_deserialize_token_stream}");
        let try_operation_error_named_token_stream = {
            let try_operation_request_error_camel_case_token_stream = generate_try_operation_request_error_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_operation_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_operation_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_operation_error_named_token_stream}");
        let try_operation_error_with_middleware_error_variants_token_stream = {
            let desirable_attribute = proc_macro_helpers::attribute::Attribute::Tvfrr200Ok;
            let try_operation_response_variants_camel_case_stringified = generate_try_operation_response_variants_camel_case_stringified(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified
            );
            let try_operation_response_variants_camel_case_token_stream = generate_try_operation_response_variants_camel_case_token_stream(
                &try_operation_response_variants_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_camel_case_token_stream = generate_try_operation_with_serialize_deserialize_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_response_variants_desirable_attribute_token_stream = generate_try_operation_response_variants_desirable_attribute_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified,
                &desirable_attribute,
                &proc_macro_name_ident_stringified
            );
            let try_operation_request_error_token_stream = generate_try_operation_request_error_token_stream(
                &try_operation_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_token_stream = generate_try_operation_with_serialize_deserialize_token_stream(
                &try_operation_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let common_error_variant_attribute_vec_handle = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &common_error_variant_attribute_vec
            );
            let json_body_logic_error_variants_vec_token_stream = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &json_body_logic_error_variants_vec_handle
            );
            let type_variants_from_request_response_vec = {
                let mut type_variants_from_request_response = std::vec::Vec::new();//todo calculate capacity
                for element in common_error_variant_attribute_vec_handle {
                    type_variants_from_request_response.push(element);
                }
                for element in json_body_logic_error_variants_vec_token_stream  {
                    type_variants_from_request_response.push(element);
                }
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &not_unique_name_vec_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &not_unique_color_vec_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &not_unique_primary_key_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &bind_query_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &not_uuid_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &read_many_with_body_payload_try_from_read_many_with_body_payload_with_serialize_deserialize_variant_attribute
                ));
                type_variants_from_request_response
            };
            crate::type_variants_from_request_response_generator::type_variants_from_request_response_generator(
                desirable_attribute,
                &desirable_token_stream,
                &quote::quote!{std::vec::Vec::<#struct_options_ident_token_stream>},
                &try_operation_camel_case_token_stream,
                &try_operation_response_variants_camel_case_stringified,
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_response_variants_desirable_attribute_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream,
                &try_operation_request_error_token_stream,
                &try_operation_with_serialize_deserialize_token_stream,
                &operation_name_lower_case_stringified,
                &code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                &code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                &error_named_derive_token_stream,
                &eo_display_token_stream,
                &eo_display_foreign_type_token_stream,
                &eo_display_with_serialize_deserialize_token_stream,
                &derive_debug_serialize_deserialize_token_stream,
                type_variants_from_request_response_vec,
                true,
                &proc_macro_name_ident_stringified,
            )
        };
        // println!("{try_operation_error_with_middleware_error_variants_token_stream}");
        let http_request_token_stream = {
            let try_operation_lower_case_token_stream = {
                let try_operation_lower_case_stringified = format!("{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
                try_operation_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
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
                pub async fn #try_operation_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #operation_parameters_camel_case_token_stream,
                ) -> Result<
                    std::vec::Vec<#struct_options_ident_token_stream>,
                    #try_operation_error_named_camel_case_token_stream,
                > {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(
                        &#operation_payload_with_serialize_deserialize_camel_case_token_stream::from(#parameters_lower_case_token_stream.#payload_lower_case_token_stream)
                    ) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_operation_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
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
                        Err(e) => Err(#try_operation_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let operation_lower_case_token_stream = operation_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {operation_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let try_operation_token_stream = {
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &try_operation_camel_case_token_stream,
                    &error_log_call_token_stream,
                    &try_operation_response_variants_token_stream,
                );
                let filter_unique_parameters_token_stream = {
                    let filter_unique_parameters_primary_key_token_stream = quote::quote!{
                        if let Some(#id_field_ident) = &#parameters_lower_case_token_stream.#payload_lower_case_token_stream.#id_field_ident {
                            let #not_unique_primary_keys_name_token_stream = {
                                let mut vec = std::vec::Vec::with_capacity(#id_field_ident.len());
                                let mut #not_unique_primary_keys_name_token_stream = std::vec::Vec::with_capacity(#id_field_ident.len());
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
                                let error = #try_operation_camel_case_token_stream::#not_unique_primary_key_variant_initialization_token_stream;
                                #error_log_call_token_stream
                                return #try_operation_response_variants_token_stream::from(error);
                            }
                        }
                    };
                    let filter_unique_parameters_other_columns_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element| {
                        let field_ident = element.field.ident.clone()
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
                        quote::quote!{
                            let #field_handle_token_stream = match #parameters_lower_case_token_stream.#payload_lower_case_token_stream.#field_ident {
                                Some(value) => {
                                    let is_unique = {
                                        let mut vec = std::vec::Vec::with_capacity(value.len());
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
                                                let mut vec = std::vec::Vec::with_capacity(value.len());
                                                let mut #not_unique_field_vec_lower_case_token_stream = std::vec::Vec::with_capacity(value.len());
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
                                            let error = #try_operation_camel_case_token_stream::#not_unique_field_vec_vec_pascal_token_stream {
                                                #not_unique_field_vec_lower_case_token_stream,
                                                #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                            };
                                            #error_log_call_token_stream
                                            return #try_operation_response_variants_token_stream::from(error);
                                        }
                                    }
                                }
                                None => None,
                            };
                        }
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
                                        //todo - think what to do with #crate_server_postgres_bind_query_try_generate_bind_increments_error_named_name_token_stream and how handle it 
                                        let e = #crate_server_postgres_bind_query_try_generate_bind_increments_error_named_name_token_stream::#checked_add_variant_initialization_token_stream;
                                        return #try_operation_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
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
                    let additional_parameters_modification_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                        let field_ident = element.field.ident.clone()
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
                        quote::quote!{
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
                                                return #try_operation_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
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
                        }
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
                                let mut additional_parameters = #std_string_string_token_stream::default();
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
                                        None => #crate_server_postgres_order_order_token_stream::default().to_string(),
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
                                            return #try_operation_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
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
                                            return #try_operation_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
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
                            query = query.bind(value.into_iter().map(|element|element.into_inner().clone()).collect::<std::vec::Vec<#sqlx_types_uuid_token_stream>>());
                        }
                    };
                    let binded_query_modifications_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                        let field_ident = element.field.ident.clone()
                            .unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                        let field_handle_token_stream = {
                            let field_handle_stringified = format!("{field_ident}_handle");
                            field_handle_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{
                            if let Some(values) = #field_handle_token_stream {
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
                        let mut vec_values = std::vec::Vec::new();
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
                    #try_operation_response_variants_token_stream::#desirable_token_stream(vec_values)
                }
            };
            // println!("{try_operation_token_stream}");
            quote::quote!{
                pub async fn #operation_lower_case_token_stream(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#operation_payload_with_serialize_deserialize_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #operation_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #operation_payload_with_serialize_deserialize_camel_case_token_stream,
                            #try_operation_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => match #operation_payload_camel_case_token_stream::try_from(value) {
                                Ok(value) => value,
                                Err(e) => {
                                    let error = #try_operation_camel_case_token_stream::#operation_payload_try_from_operation_payload_with_serialize_deserialize_camel_case_token_stream {
                                        #operation_payload_try_from_operation_payload_with_serialize_deserialize_lower_case_token_stream: e,
                                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                    };
                                    #error_log_call_token_stream
                                    return #try_operation_response_variants_token_stream::from(error);
                                }
                            },
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #try_operation_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #payload_with_serialize_deserialize_token_stream
            #operation_payload_try_from_operation_payload_with_serialize_deserialize_error_named_token_stream
            #impl_std_convert_try_from_operation_payload_with_serialize_deserialize_for_operation_payload_token_stream
            #impl_std_convert_from_operation_payload_for_operation_payload_with_serialize_deserialize_token_stream
            #try_operation_error_named_token_stream
            #try_operation_error_with_middleware_error_variants_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // proc_macro_helpers::write_token_stream_into_file::write_token_stream_into_file(
    //     &proc_macro_name,
    //     &read_many_with_body_token_stream,
    //     &proc_macro_name_ident_stringified
    // );
    //todo WHY ITS RETURN SUCCESS EVEN IF ROW DOES NOT EXISTS?
    let update_one_token_stream = {
        let operation_name_camel_case_stringified = format!("{update_camel_case_stringified}{one_camel_case_stringified}");
        let operation_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&operation_name_camel_case_stringified.to_string());
        let operation_parameters_camel_case_token_stream = generate_operation_parameters_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            parameters_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_camel_case_token_stream = generate_path_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            path_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_with_serialize_deserialize_camel_case_token_stream = generate_path_with_serialize_deserialize_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            path_camel_case_stringified,
            with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_stringified = generate_path_try_from_path_with_serialize_deserialize_stringified(
            &operation_name_camel_case_stringified,
            path_camel_case_stringified,
            &try_from_camel_case_stringified,
            with_serialize_deserialize_camel_case_stringified,
        );
        let operation_path_try_from_operation_path_with_serialize_deserialize_error_named_camel_case_token_stream = generate_path_try_from_path_with_serialize_deserialize_error_named_token_stream(
            &operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_stringified,
            error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_token_stream = generate_path_try_from_path_with_serialize_deserialize_camel_case_token_stream(
            &operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_try_from_operation_path_with_serialize_deserialize_lower_case_token_stream = generate_path_try_from_path_with_serialize_deserialize_lower_case_token_stream(
            &operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_camel_case_token_stream = generate_operation_payload_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            payload_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_error_named_camel_case_token_stream = generate_try_operation_error_named_camel_case_token_stream(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            &error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_response_variants_token_stream = generate_try_operation_response_variants_token_stream(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            &response_variants_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_camel_case_stringified = generate_try_operation_camel_case_stringified(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
        );
        let try_operation_camel_case_token_stream = generate_try_operation_camel_case_token_stream(
            &try_operation_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #operation_parameters_camel_case_token_stream {
                    pub #path_lower_case_token_stream: #operation_path_camel_case_token_stream,
                    pub #payload_lower_case_token_stream: #operation_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let path_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #operation_path_camel_case_token_stream {
                    pub #id_field_ident: #crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream,
                }
            }
        };
        // println!("{path_token_stream}");
        let path_with_serialize_deserialize_token_stream = {
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #operation_path_with_serialize_deserialize_camel_case_token_stream {
                    #id_field_ident: #crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream,
                }
            }
        };
        // println!("{path_with_serialize_deserialize_token_stream}");
        let operation_path_try_from_operation_path_with_serialize_deserialize_error_named_token_stream = {
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #operation_path_try_from_operation_path_with_serialize_deserialize_error_named_camel_case_token_stream {
                    #not_uuid_token_camel_case_stream {
                        #eo_error_occurence_attribute_token_stream
                        #not_uuid_token_lower_case_stream: #crate_server_postgres_uuid_wrapper_uuid_wrapper_try_from_possible_uuid_wrapper_error_named_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                } 
            }
        };
        // println!("{operation_path_try_from_operation_path_with_serialize_deserialize_error_named_token_stream}");
        let impl_std_convert_try_from_operation_path_with_serialize_deserialize_for_operation_path_token_stream = {
            quote::quote!{
                impl std::convert::TryFrom<#operation_path_with_serialize_deserialize_camel_case_token_stream> for #operation_path_camel_case_token_stream {
                    type Error = #operation_path_try_from_operation_path_with_serialize_deserialize_error_named_camel_case_token_stream;
                    fn try_from(value: #operation_path_with_serialize_deserialize_camel_case_token_stream) -> Result<Self, Self::Error> {
                        let #id_field_ident = match #crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream::try_from(value.#id_field_ident) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(Self::Error::#not_uuid_token_camel_case_stream {
                                    #not_uuid_token_lower_case_stream: e,
                                    #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
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
        // println!("{impl_std_convert_try_from_operation_path_with_serialize_deserialize_for_operation_path_token_stream}");
        let payload_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                let field_type = &element.field.ty;
                quote::quote!{
                    pub #field_ident: std::option::Option<#field_type>
                }
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #operation_payload_camel_case_token_stream {
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let try_operation_error_named_token_stream = {
            let try_operation_request_error_camel_case_token_stream = generate_try_operation_request_error_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_operation_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_operation_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_operation_error_named_token_stream}");
        let try_operation_error_with_middleware_error_variants_token_stream = {
            let desirable_attribute = proc_macro_helpers::attribute::Attribute::Tvfrr200Ok;
            let try_operation_response_variants_camel_case_stringified = generate_try_operation_response_variants_camel_case_stringified(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified
            );
            let try_operation_response_variants_camel_case_token_stream = generate_try_operation_response_variants_camel_case_token_stream(
                &try_operation_response_variants_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_camel_case_token_stream = generate_try_operation_with_serialize_deserialize_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_response_variants_desirable_attribute_token_stream = generate_try_operation_response_variants_desirable_attribute_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified,
                &desirable_attribute,
                &proc_macro_name_ident_stringified
            );
            let try_operation_request_error_token_stream = generate_try_operation_request_error_token_stream(
                &try_operation_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_token_stream = generate_try_operation_with_serialize_deserialize_token_stream(
                &try_operation_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let common_error_variant_attribute_vec_handle = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &common_error_variant_attribute_vec
            );
            let path_logic_error_variants_vec_token_stream = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &path_logic_error_variants_vec_handle
            );
            let json_body_logic_error_variants_vec_token_stream = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &json_body_logic_error_variants_vec_handle
            );
            let type_variants_from_request_response_vec = {
                let mut type_variants_from_request_response = std::vec::Vec::new();//todo calculate capacity
                for element in common_error_variant_attribute_vec_handle {
                    type_variants_from_request_response.push(element);
                }
                for element in path_logic_error_variants_vec_token_stream  {
                    type_variants_from_request_response.push(element);
                }
                for element in json_body_logic_error_variants_vec_token_stream  {
                    type_variants_from_request_response.push(element);
                }
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &bind_query_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &no_payload_fields_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &update_one_path_try_from_update_one_path_with_serialize_deserialize_variant_attribute
                ));
                type_variants_from_request_response
            };
            crate::type_variants_from_request_response_generator::type_variants_from_request_response_generator(
                desirable_attribute,
                &desirable_token_stream,
                &quote::quote!{()},
                &try_operation_camel_case_token_stream,
                &try_operation_response_variants_camel_case_stringified,
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_response_variants_desirable_attribute_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream,
                &try_operation_request_error_token_stream,
                &try_operation_with_serialize_deserialize_token_stream,
                &operation_name_lower_case_stringified,
                &code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                &code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                &error_named_derive_token_stream,
                &eo_display_token_stream,
                &eo_display_foreign_type_token_stream,
                &eo_display_with_serialize_deserialize_token_stream,
                &derive_debug_serialize_deserialize_token_stream,
                type_variants_from_request_response_vec,
                false,
                &proc_macro_name_ident_stringified,
            )
        };
        // println!("{try_operation_error_with_middleware_error_variants_token_stream}");
        let http_request_token_stream = {
            let try_operation_lower_case_token_stream = {
                let try_operation_lower_case_stringified = format!("{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
                try_operation_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
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
                pub async fn #try_operation_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #operation_parameters_camel_case_token_stream,
                ) -> Result<(), #try_operation_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&#parameters_lower_case_token_stream.#payload_lower_case_token_stream) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_operation_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
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
                        Err(e) => Err(#try_operation_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let operation_lower_case_token_stream = operation_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {operation_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let try_operation_token_stream = {
                let check_for_none_token_stream_excluding_primary_key = crate::check_for_none::check_for_none(
                    &fields_named,
                    &id_field,
                    &proc_macro_name_ident_stringified,
                    dot_space,
                    &try_operation_response_variants_token_stream,
                    true
                );
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &try_operation_camel_case_token_stream,
                    &error_log_call_token_stream,
                    &try_operation_response_variants_token_stream,
                );
                let query_string_token_stream = {
                    let additional_parameters_modification_token_stream = {
                        let fields_named_filtered = fields_named_wrappers_excluding_primary_key.iter().map(|element|&element.field).collect::<std::vec::Vec<&syn::Field>>();
                        fields_named_filtered.iter().enumerate().map(|(index, field)| {
                            let field_ident = field.ident.clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                });
                            let handle_token_stream = {
                                let possible_dot_space = match (
                                    index.checked_add(1).unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} {index} {}", proc_macro_helpers::global_variables::hardcode::CHECKED_ADD_NONE_OVERFLOW_MESSAGE))
                                ) == fields_named_wrappers_excluding_primary_key_len {
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
                                            return #try_operation_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                        },
                                    }
                                }
                            }
                        }).collect::<std::vec::Vec<proc_macro2::TokenStream>>()
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
                                    return #try_operation_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
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
                        let mut query = #std_string_string_token_stream::from(#handle_token_stream);
                        #(#additional_parameters_modification_token_stream)*
                        #additional_parameters_id_modification_token_stream
                        query.push_str(&format!(#returning_id_quotes_token_stream));
                        query
                    }
                };
                // println!("{query_string_token_stream}");
                let binded_query_token_stream = {
                    let binded_query_modifications_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                        let field_ident = element.field.ident.clone()
                            .unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                        quote::quote!{
                            if let Some(value) = #parameters_lower_case_token_stream.#payload_lower_case_token_stream.#field_ident {
                                query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(
                                    value,
                                    query,
                                );
                            }
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
                        Ok(_) => #try_operation_response_variants_token_stream::#desirable_token_stream(()),//todo () type token_stream
                        Err(e) => {
                            #from_log_and_return_error_token_stream;
                        }
                    }
                }
            };
            // println!("{try_operation_token_stream}");
            quote::quote!{
                pub async fn #operation_lower_case_token_stream<'a>(
                    #path_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_path_token_stream<#operation_path_with_serialize_deserialize_camel_case_token_stream>,
                        #axum_extract_rejection_path_rejection_token_stream,
                    >,
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#operation_payload_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #operation_parameters_camel_case_token_stream {
                        #path_lower_case_token_stream: match #crate_server_routes_helpers_path_extractor_error_path_value_result_extractor_token_stream::<
                            #operation_path_with_serialize_deserialize_camel_case_token_stream,
                            #try_operation_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#path_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => match #operation_path_camel_case_token_stream::try_from(value) {
                                Ok(value) => value,
                                Err(e) => {
                                    let error = #try_operation_camel_case_token_stream::#operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_token_stream {
                                        #operation_path_try_from_operation_path_with_serialize_deserialize_lower_case_token_stream: e,
                                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                    };
                                    #error_log_call_token_stream
                                    return #try_operation_response_variants_token_stream::from(error);
                                }
                            },
                            Err(err) => {
                                return err;
                            }
                        },
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #operation_payload_camel_case_token_stream,
                            #try_operation_response_variants_token_stream,
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
                        #try_operation_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #path_token_stream
            #path_with_serialize_deserialize_token_stream
            #operation_path_try_from_operation_path_with_serialize_deserialize_error_named_token_stream
            #impl_std_convert_try_from_operation_path_with_serialize_deserialize_for_operation_path_token_stream
            #payload_token_stream
            #try_operation_error_named_token_stream
            #try_operation_error_with_middleware_error_variants_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // proc_macro_helpers::write_token_stream_into_file::write_token_stream_into_file(
    //     &proc_macro_name,
    //     &update_one_token_stream,
    //     &proc_macro_name_ident_stringified
    // );
    let update_many_token_stream = {
        let operation_name_camel_case_stringified = format!("{update_camel_case_stringified}{many_camel_case_stringified}");
        let operation_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&operation_name_camel_case_stringified.to_string());
        let operation_parameters_camel_case_token_stream = generate_operation_parameters_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            parameters_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_element_camel_case_token_stream = generate_operation_payload_element_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            &payload_element_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_camel_case_token_stream = quote::quote!{std::vec::Vec<#operation_payload_element_camel_case_token_stream>};
        let operation_payload_element_with_serialize_deserialize_camel_case_token_stream = generate_payload_element_with_serialize_deserialize_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            &payload_element_with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_with_serialize_deserialize_camel_case_token_stream = quote::quote!{std::vec::Vec<#operation_payload_element_with_serialize_deserialize_camel_case_token_stream>};
        let operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_camel_case_stringified = generate_payload_element_try_from_payload_element_with_serialize_deserialize_camel_case_stringified(
            &operation_name_camel_case_stringified,
            &payload_element_camel_case_stringified,
            &try_from_camel_case_stringified,
            &payload_element_with_serialize_deserialize_camel_case_stringified
        );
        let operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_camel_case_token_stream = generate_payload_element_try_from_payload_element_with_serialize_deserialize_camel_case_token_stream(
            &operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_lower_case_token_stream = generate_payload_element_try_from_payload_element_with_serialize_deserialize_lower_case_token_stream(
            &operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_error_named_camel_case_token_stream = generate_payload_element_try_from_payload_element_with_serialize_deserialize_error_named_camel_case_token_stream(
            &operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_camel_case_stringified,
            error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_error_named_camel_case_token_stream = generate_try_operation_error_named_camel_case_token_stream(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            &error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_response_variants_token_stream = generate_try_operation_response_variants_token_stream(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            &response_variants_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_camel_case_stringified = generate_try_operation_camel_case_stringified(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
        );
        let try_operation_camel_case_token_stream = generate_try_operation_camel_case_token_stream(
            &try_operation_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #operation_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #operation_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let payload_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                let field_type = &element.field.ty;
                //todo make sure name and color both are not None(make it option<value>, not just a value)
                quote::quote!{
                    pub #field_ident: #field_type
                }
            });
            quote::quote!{
                #derive_debug_token_stream
                pub struct #operation_payload_element_camel_case_token_stream {
                    pub #id_field_ident: #crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream,
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let payload_with_serialize_deserialize_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                let field_type = &element.field.ty;
                quote::quote!{
                    pub #field_ident: #field_type
                }
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #operation_payload_element_with_serialize_deserialize_camel_case_token_stream {
                    pub #id_field_ident: #crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream,
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_with_serialize_deserialize_token_stream}");
        let impl_std_convert_from_operation_payload_elemen_for_operation_payload_element_with_serialize_deserialize_token_stream = {
            let fields_assignments_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    let #field_ident = value.#field_ident;
                }
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
                impl std::convert::From<#operation_payload_element_camel_case_token_stream> for #operation_payload_element_with_serialize_deserialize_camel_case_token_stream {
                    fn from(value: #operation_payload_element_camel_case_token_stream) -> Self {
                        let #id_field_ident = #crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream::from(value.#id_field_ident);
                        #(#fields_assignments_token_stream)*
                        Self {
                            #(#self_init_fields_token_stream),*
                        }
                    }
                }
            }
        };
        // println!("{impl_std_convert_from_operation_payload_elemen_for_operation_payload_element_with_serialize_deserialize_token_stream}");
        let operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_error_named_token_stream = {
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_error_named_camel_case_token_stream {
                    #not_uuid_token_camel_case_stream {
                        #eo_display_token_stream
                        #not_uuid_token_lower_case_stream: sqlx::types::uuid::Error,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_error_named_token_stream}");
        let impl_std_convert_try_from_operation_payload_element_with_serialize_deserialize_for_operation_payload_element_token_stream = {
            let fields_assignments_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    let #field_ident = value.#field_ident;
                }
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
                impl std::convert::TryFrom<#operation_payload_element_with_serialize_deserialize_camel_case_token_stream> for #operation_payload_element_camel_case_token_stream {
                    type Error = #operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_error_named_camel_case_token_stream;
                    fn try_from(value: #operation_payload_element_with_serialize_deserialize_camel_case_token_stream) -> Result<Self, Self::Error> {
                        let #id_field_ident = match #sqlx_types_uuid_token_stream::parse_str(value.#id_field_ident.to_inner()) {
                            Ok(value) => #crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream::from(value),
                            Err(e) => {
                                return Err(Self::Error::#not_uuid_token_camel_case_stream {
                                    #not_uuid_token_lower_case_stream: e,
                                    #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
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
        // println!("{impl_std_convert_try_from_operation_payload_element_with_serialize_deserialize_for_operation_payload_element_token_stream}");
        let try_operation_error_named_token_stream = {
            let try_operation_request_error_camel_case_token_stream = generate_try_operation_request_error_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_operation_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_operation_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_operation_error_named_token_stream}");
        let try_operation_error_with_middleware_error_variants_token_stream = {
            let desirable_attribute = proc_macro_helpers::attribute::Attribute::Tvfrr200Ok;
            let try_operation_response_variants_camel_case_stringified = generate_try_operation_response_variants_camel_case_stringified(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified
            );
            let try_operation_response_variants_camel_case_token_stream = generate_try_operation_response_variants_camel_case_token_stream(
                &try_operation_response_variants_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_camel_case_token_stream = generate_try_operation_with_serialize_deserialize_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_response_variants_desirable_attribute_token_stream = generate_try_operation_response_variants_desirable_attribute_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified,
                &desirable_attribute,
                &proc_macro_name_ident_stringified
            );
            let try_operation_request_error_token_stream = generate_try_operation_request_error_token_stream(
                &try_operation_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_token_stream = generate_try_operation_with_serialize_deserialize_token_stream(
                &try_operation_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let common_error_variant_attribute_vec_handle = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &common_error_variant_attribute_vec
            );
            let json_body_logic_error_variants_vec_token_stream = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &json_body_logic_error_variants_vec_handle
            );
            let type_variants_from_request_response_vec = {
                let mut type_variants_from_request_response = std::vec::Vec::new();//todo calculate capacity
                for element in common_error_variant_attribute_vec_handle {
                    type_variants_from_request_response.push(element);
                }
                for element in json_body_logic_error_variants_vec_token_stream  {
                    type_variants_from_request_response.push(element);
                }
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &not_unique_primary_key_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &bind_query_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &checked_add_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &no_payload_fields_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &commit_failed_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &non_existing_primary_keys_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &primary_key_from_row_and_failed_rollback_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &non_existing_primary_keys_and_failed_rollback_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &query_and_rollback_failed_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &update_many_payload_element_try_from_update_many_payload_element_with_serialize_deserialize_variant_attribute
                ));
                type_variants_from_request_response
            };
            crate::type_variants_from_request_response_generator::type_variants_from_request_response_generator(
                desirable_attribute,
                &desirable_token_stream,
                &quote::quote!{()},
                &try_operation_camel_case_token_stream,
                &try_operation_response_variants_camel_case_stringified,
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_response_variants_desirable_attribute_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream,
                &try_operation_request_error_token_stream,
                &try_operation_with_serialize_deserialize_token_stream,
                &operation_name_lower_case_stringified,
                &code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                &code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                &error_named_derive_token_stream,
                &eo_display_token_stream,
                &eo_display_foreign_type_token_stream,
                &eo_display_with_serialize_deserialize_token_stream,
                &derive_debug_serialize_deserialize_token_stream,
                type_variants_from_request_response_vec,
                false,
                &proc_macro_name_ident_stringified,
            )
        };
        // println!("{try_operation_error_with_middleware_error_variants_token_stream}");
        let http_request_token_stream = {
            let try_operation_lower_case_token_stream = {
                let try_operation_lower_case_stringified = format!("{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
                try_operation_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
                tvfrr_extraction_logic_stringified
                .parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let url_handle_token_stream = {
                let url_handle_stringified = format!("\"{{}}/{table_name_stringified}/{batch_stringified}\"");//todo where
                url_handle_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {url_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            quote::quote!{
                pub async fn #try_operation_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #operation_parameters_camel_case_token_stream,
                ) -> Result<(), #try_operation_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(&
                        #parameters_lower_case_token_stream.#payload_lower_case_token_stream
                        .into_iter()
                        .map(
                            |element|
                            #operation_payload_element_with_serialize_deserialize_camel_case_token_stream::from(element)
                        )
                        .collect::<std::vec::Vec<#operation_payload_element_with_serialize_deserialize_camel_case_token_stream>>()
                    ) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_operation_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
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
                        Err(e) => Err(#try_operation_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let operation_lower_case_token_stream = operation_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {operation_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let try_operation_token_stream = {
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &try_operation_camel_case_token_stream,
                    &error_log_call_token_stream,
                    &try_operation_response_variants_token_stream,
                );
                let expected_updated_primary_keys_token_stream = quote::quote!{
                    #parameters_lower_case_token_stream
                    .#payload_lower_case_token_stream
                    .iter()
                    .map(|element| element.#id_field_ident.clone()) //todo - maybe its not a good idea to remove .clone here coz in macro dont know what type
                    .collect::<std::vec::Vec<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>>()
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
                        let fields_named_filtered = fields_named_wrappers_excluding_primary_key.iter().map(|element|&element.field).collect::<std::vec::Vec<&syn::Field>>();
                        fields_named_filtered.iter().enumerate().fold(std::string::String::default(), |mut acc, (index, field)| {
                            let field_ident = field.ident.clone().unwrap_or_else(|| {
                                panic!("{proc_macro_name_ident_stringified} field.ident is None")
                            });
                            let possible_dot_space = match (
                                index.checked_add(1).unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} {index} {}", proc_macro_helpers::global_variables::hardcode::CHECKED_ADD_NONE_OVERFLOW_MESSAGE))
                            ) == fields_named_wrappers_excluding_primary_key_len {
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
                    let column_vecs_with_capacity_token_stream = fields_named.iter().map(|_|quote::quote!{std::vec::Vec::with_capacity(#current_vec_len_name_token_stream)});
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
                                .collect::<std::vec::Vec<#sqlx_types_uuid_token_stream>>()
                            );
                        }
                    };
                    let column_query_bind_vecs_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                        let field_ident_underscore_vec_token_stream = {
                            let field_ident_underscore_vec_stringified = {
                                let field_ident = element.field.ident.clone()
                                    .unwrap_or_else(|| {
                                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                    });
                                format!("{field_ident}{underscore_vec_name_stringified}")
                            };
                            field_ident_underscore_vec_stringified.parse::<proc_macro2::TokenStream>()
                            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_ident_underscore_vec_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                        };
                        quote::quote!{#query_name_token_stream = #query_name_token_stream.bind(#field_ident_underscore_vec_token_stream);}
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
                    &query_and_rollback_failed_variant_initialization_token_stream,
                    &primary_key_uuid_wrapper_try_from_sqlx_row_name_token_stream,
                    &from_log_and_return_error_token_stream,
                    &rollback_error_name_token_stream,
                    &primary_key_from_row_and_failed_rollback_variant_initialization_token_stream,
                    &non_existing_primary_keys_name_token_stream,
                    &expected_updated_primary_keys_name_token_stream,
                    &primary_key_vec_name_token_stream,
                    &rollback_token_stream,
                    &non_existing_primary_keys_variant_initialization_token_stream,
                    &non_existing_primary_keys_and_failed_rollback_variant_initialization_token_stream,
                    &postgres_transaction_token_stream,
                    &commit_token_stream,
                    &try_operation_response_variants_token_stream,
                    &desirable_token_stream,
                    &try_operation_camel_case_token_stream,
                    &commit_failed_variant_initialization_token_stream,
                    &error_log_call_token_stream,
                );
                quote::quote!{
                    {
                        let #not_unique_primary_keys_name_token_stream = {
                            let mut vec = std::vec::Vec::with_capacity(#parameters_lower_case_token_stream.#payload_lower_case_token_stream.len());
                            let mut #not_unique_primary_keys_name_token_stream = std::vec::Vec::with_capacity(#parameters_lower_case_token_stream.#payload_lower_case_token_stream.len());
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
                            let error = #try_operation_camel_case_token_stream::#not_unique_primary_key_variant_initialization_token_stream;
                            #error_log_call_token_stream
                            return #try_operation_response_variants_token_stream::from(error);
                        }
                    }
                    #generate_postgres_transaction_token_stream
                }  
            };
            // println!("{try_operation_token_stream}");
            quote::quote!{
                pub async fn #operation_lower_case_token_stream<'a>(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#operation_payload_with_serialize_deserialize_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #operation_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #operation_payload_with_serialize_deserialize_camel_case_token_stream,
                            #try_operation_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => match value.into_iter()
                                .map(|element|#operation_payload_element_camel_case_token_stream::try_from(element))
                                .collect::<Result<
                                    #operation_payload_camel_case_token_stream, 
                                    #operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_error_named_camel_case_token_stream
                                >>() 
                                {
                                    Ok(value) => value,
                                    Err(e) => {
                                        let error = #try_operation_camel_case_token_stream::#operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_camel_case_token_stream {
                                            #operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_lower_case_token_stream: e,
                                            #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                        };
                                        #error_log_call_token_stream
                                        return #try_operation_response_variants_token_stream::from(error);
                                    }
                                },
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #try_operation_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #payload_with_serialize_deserialize_token_stream
            #impl_std_convert_from_operation_payload_elemen_for_operation_payload_element_with_serialize_deserialize_token_stream
            #operation_payload_element_try_from_operation_payload_element_with_serialize_deserialize_error_named_token_stream
            #impl_std_convert_try_from_operation_payload_element_with_serialize_deserialize_for_operation_payload_element_token_stream
            #try_operation_error_named_token_stream
            #try_operation_error_with_middleware_error_variants_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // proc_macro_helpers::write_token_stream_into_file::write_token_stream_into_file(
    //     &proc_macro_name,
    //     &update_many_token_stream,
    //     &proc_macro_name_ident_stringified
    // );
    let delete_one_token_stream = {
        let operation_name_camel_case_stringified = format!("{delete_camel_case_stringified}{one_camel_case_stringified}");
        let operation_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&operation_name_camel_case_stringified.to_string());
        let operation_parameters_camel_case_token_stream = generate_operation_parameters_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            parameters_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_camel_case_token_stream = generate_path_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            path_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_with_serialize_deserialize_camel_case_token_stream = generate_path_with_serialize_deserialize_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            path_camel_case_stringified,
            with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_stringified = generate_path_try_from_path_with_serialize_deserialize_stringified(
            &operation_name_camel_case_stringified,
            path_camel_case_stringified,
            &try_from_camel_case_stringified,
            with_serialize_deserialize_camel_case_stringified,
        );
        let operation_path_try_from_operation_path_with_serialize_deserialize_error_named_name_token_stream = generate_path_try_from_path_with_serialize_deserialize_error_named_token_stream(
            &operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_stringified,
            error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_token_stream = generate_path_try_from_path_with_serialize_deserialize_camel_case_token_stream(
            &operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_path_try_from_operation_path_with_serialize_deserialize_lower_case_token_stream = generate_path_try_from_path_with_serialize_deserialize_lower_case_token_stream(
            &operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_error_named_camel_case_token_stream = generate_try_operation_error_named_camel_case_token_stream(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            &error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_response_variants_token_stream = generate_try_operation_response_variants_token_stream(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            &response_variants_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_camel_case_stringified = generate_try_operation_camel_case_stringified(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
        );
        let try_operation_camel_case_token_stream = generate_try_operation_camel_case_token_stream(
            &try_operation_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #operation_parameters_camel_case_token_stream {
                    pub #path_lower_case_token_stream: #operation_path_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let path_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #operation_path_camel_case_token_stream {
                    pub #id_field_ident: #crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream,
                }
            }
        };
        // println!("{path_token_stream}");
        let path_with_serialize_deserialize_token_stream = {
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #operation_path_with_serialize_deserialize_camel_case_token_stream {
                    pub #id_field_ident: #crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream,
                }
            }
        };
        // println!("{path_with_serialize_deserialize_token_stream}");
        let operation_path_try_from_operation_path_with_serialize_deserialize_error_named_token_stream = {
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #operation_path_try_from_operation_path_with_serialize_deserialize_error_named_name_token_stream {
                    #not_uuid_token_camel_case_stream {
                        #eo_error_occurence_attribute_token_stream
                        #not_uuid_token_lower_case_stream: #crate_server_postgres_uuid_wrapper_uuid_wrapper_try_from_possible_uuid_wrapper_error_named_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{operation_path_try_from_operation_path_with_serialize_deserialize_error_named_token_stream}");
        let impl_std_convert_try_from_operation_path_with_serialize_deserialize_for_operation_path_token_stream = {
            quote::quote!{
                impl std::convert::TryFrom<#operation_path_with_serialize_deserialize_camel_case_token_stream> for #operation_path_camel_case_token_stream {
                    type Error = #operation_path_try_from_operation_path_with_serialize_deserialize_error_named_name_token_stream;
                    fn try_from(value: #operation_path_with_serialize_deserialize_camel_case_token_stream) -> Result<Self, Self::Error> {
                        match #crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream::try_from(value.#id_field_ident) {
                            Ok(value) => Ok(Self { #id_field_ident: value }),
                            Err(e) => Err(Self::Error::#not_uuid_token_camel_case_stream {
                                #not_uuid_token_lower_case_stream: e,
                                #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                            }),
                        }
                    }
                }
            }
        };
        // println!("{impl_std_convert_try_from_operation_path_with_serialize_deserialize_for_operation_path_token_stream}");
        let try_operation_error_named_token_stream = {
            let try_operation_request_error_camel_case_token_stream = generate_try_operation_request_error_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_operation_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_operation_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{try_operation_error_named_token_stream}");
        let try_operation_error_with_middleware_error_variants_token_stream = {
            let desirable_attribute = proc_macro_helpers::attribute::Attribute::Tvfrr200Ok;
            let try_operation_response_variants_camel_case_stringified = generate_try_operation_response_variants_camel_case_stringified(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified
            );
            let try_operation_response_variants_camel_case_token_stream = generate_try_operation_response_variants_camel_case_token_stream(
                &try_operation_response_variants_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_camel_case_token_stream = generate_try_operation_with_serialize_deserialize_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_response_variants_desirable_attribute_token_stream = generate_try_operation_response_variants_desirable_attribute_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified,
                &desirable_attribute,
                &proc_macro_name_ident_stringified
            );
            let try_operation_request_error_token_stream = generate_try_operation_request_error_token_stream(
                &try_operation_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_token_stream = generate_try_operation_with_serialize_deserialize_token_stream(
                &try_operation_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let common_error_variant_attribute_vec_handle = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &common_error_variant_attribute_vec
            );
            let path_logic_error_variants_vec_token_stream = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &path_logic_error_variants_vec_handle
            );
            let type_variants_from_request_response_vec = {
                let mut type_variants_from_request_response = std::vec::Vec::new();//todo calculate capacity
                for element in common_error_variant_attribute_vec_handle {
                    type_variants_from_request_response.push(element);
                }
                for element in path_logic_error_variants_vec_token_stream  {
                    type_variants_from_request_response.push(element);
                }
                //todo why no bind query error here?
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &delete_one_path_try_from_delete_one_path_with_serialize_deserialize_variant_attribute
                ));
                type_variants_from_request_response
            };
            crate::type_variants_from_request_response_generator::type_variants_from_request_response_generator(
                desirable_attribute,
                &desirable_token_stream,
                &quote::quote!{()},
                &try_operation_camel_case_token_stream,
                &try_operation_response_variants_camel_case_stringified,
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_response_variants_desirable_attribute_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream,
                &try_operation_request_error_token_stream,
                &try_operation_with_serialize_deserialize_token_stream,
                &operation_name_lower_case_stringified,
                &code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                &code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                &error_named_derive_token_stream,
                &eo_display_token_stream,
                &eo_display_foreign_type_token_stream,
                &eo_display_with_serialize_deserialize_token_stream,
                &derive_debug_serialize_deserialize_token_stream,
                type_variants_from_request_response_vec,
                false,
                &proc_macro_name_ident_stringified,
            )
        };
        // println!("{try_operation_error_with_middleware_error_variants_token_stream}");
        let http_request_token_stream = {
            let try_operation_lower_case_token_stream = {
                let try_operation_lower_case_stringified = format!("{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
                try_operation_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
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
                pub async fn #try_operation_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #operation_parameters_camel_case_token_stream,
                ) -> Result<(), #try_operation_error_named_camel_case_token_stream> {
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
                        Err(e) => Err(#try_operation_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let operation_lower_case_token_stream = operation_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {operation_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let try_operation_token_stream = {
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &try_operation_camel_case_token_stream,
                    &error_log_call_token_stream,
                    &try_operation_response_variants_token_stream,
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
                        Ok(row) => #try_operation_response_variants_token_stream::#desirable_token_stream(()),//todo - () as variable token stream
                        Err(e) => {
                            #from_log_and_return_error_token_stream;
                        }
                    }
                }
            };
            // println!("{try_operation_token_stream}");
            quote::quote!{
                #[utoipa::path(
                    delete,
                    path = "/api/cats",//todo
                    responses(
                        (status = 200, description = "delete by primary key", body = [#try_operation_camel_case_token_stream])
                    )
                )]
                pub async fn #operation_lower_case_token_stream<'a>(
                    #path_extraction_result_lower_case_token_stream: Result<
                        #axum_extract_path_token_stream<#operation_path_with_serialize_deserialize_camel_case_token_stream>,
                        #axum_extract_rejection_path_rejection_token_stream,
                    >,
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #operation_parameters_camel_case_token_stream {
                        #path_lower_case_token_stream: match #crate_server_routes_helpers_path_extractor_error_path_value_result_extractor_token_stream::<
                            #operation_path_with_serialize_deserialize_camel_case_token_stream,
                            #try_operation_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#path_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => match #operation_path_camel_case_token_stream::try_from(value) {
                                Ok(value) => value,
                                Err(e) => {
                                    let error = #try_operation_camel_case_token_stream::#operation_path_try_from_operation_path_with_serialize_deserialize_camel_case_token_stream {
                                        #operation_path_try_from_operation_path_with_serialize_deserialize_lower_case_token_stream: e,
                                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                    };
                                    #error_log_call_token_stream
                                    return #try_operation_response_variants_token_stream::from(error);
                                }
                            },
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #try_operation_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #path_token_stream
            #path_with_serialize_deserialize_token_stream
            #operation_path_try_from_operation_path_with_serialize_deserialize_error_named_token_stream
            #impl_std_convert_try_from_operation_path_with_serialize_deserialize_for_operation_path_token_stream
            #try_operation_error_named_token_stream
            #try_operation_error_with_middleware_error_variants_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // proc_macro_helpers::write_token_stream_into_file::write_token_stream_into_file(
    //     &proc_macro_name,
    //     &delete_one_token_stream,
    //     &proc_macro_name_ident_stringified
    // );
    let delete_many_with_body_token_stream = {
        let operation_name_camel_case_stringified = format!("{delete_camel_case_stringified}{many_camel_case_stringified}{with_body_camel_case_stringified}");
        let operation_name_lower_case_stringified = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&operation_name_camel_case_stringified.to_string());
        let operation_parameters_camel_case_token_stream = generate_operation_parameters_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            parameters_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_camel_case_token_stream = generate_operation_payload_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            payload_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_with_serialize_deserialize_camel_case_token_stream = generate_payload_with_serialize_deserialize_camel_case_token_stream(
            &operation_name_camel_case_stringified,
            payload_camel_case_stringified,
            with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_try_from_operation_payload_with_serialize_deserialize_camel_case_stringified = generate_payload_try_from_payload_with_serialize_deserialize_stringified(
            &operation_name_camel_case_stringified,
            payload_camel_case_stringified,
            &try_from_camel_case_stringified,
            with_serialize_deserialize_camel_case_stringified,
        );
        let operation_payload_try_from_operation_payload_with_serialize_deserialize_error_named_camel_case_token_stream = generate_payload_try_from_payload_with_serialize_deserialize_error_named_camel_case_token_stream(
            &operation_payload_try_from_operation_payload_with_serialize_deserialize_camel_case_stringified,
            error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_try_from_operation_payload_with_serialize_deserialize_camel_case_token_stream = generate_payload_try_from_payload_with_serialize_deserialize_camel_case_token_stream(
            &operation_payload_try_from_operation_payload_with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let operation_payload_try_from_operation_payload_with_serialize_deserialize_lower_case_token_stream = generate_payload_try_from_payload_with_serialize_deserialize_lower_case_token_stream(
            &operation_payload_try_from_operation_payload_with_serialize_deserialize_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_error_named_camel_case_token_stream = generate_try_operation_error_named_camel_case_token_stream(
            try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            error_named_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_response_variants_token_stream = generate_try_operation_response_variants_token_stream(
            try_camel_case_stringified,
            &operation_name_camel_case_stringified,
            response_variants_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let try_operation_camel_case_stringified = generate_try_operation_camel_case_stringified(
            &try_camel_case_stringified,
            &operation_name_camel_case_stringified,
        );
        let try_operation_camel_case_token_stream = generate_try_operation_camel_case_token_stream(
            &try_operation_camel_case_stringified,
            &proc_macro_name_ident_stringified
        );
        let parameters_token_stream = {
            quote::quote!{
                #derive_debug_token_stream
                pub struct #operation_parameters_camel_case_token_stream {
                    pub #payload_lower_case_token_stream: #operation_payload_camel_case_token_stream,
                }
            }
        };
        // println!("{parameters_token_stream}");
        let payload_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    pub #field_ident: std::option::Option<std::vec::Vec<#crate_server_postgres_regex_filter_regex_filter_token_stream>>//todo
                }
            });
            quote::quote!{
                #derive_debug_token_stream
                pub struct #operation_payload_camel_case_token_stream {
                    pub #id_field_ident: std::option::Option<std::vec::Vec<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>>,
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{payload_token_stream}");
        let operation_payload_with_serialize_deserialize_token_stream = {
            let fields_with_excluded_id_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    pub #field_ident: std::option::Option<std::vec::Vec<#crate_server_postgres_regex_filter_regex_filter_token_stream>>
                }
            });
            quote::quote!{
                #derive_debug_serialize_deserialize_token_stream
                pub struct #operation_payload_with_serialize_deserialize_camel_case_token_stream {
                    pub #id_field_ident: std::option::Option<std::vec::Vec<#crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream>>,
                    #(#fields_with_excluded_id_token_stream),*
                }
            }
        };
        // println!("{operation_payload_with_serialize_deserialize_token_stream}");
        let operation_payload_try_from_operation_payload_with_serialize_deserialize_error_named_token_stream = {
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #operation_payload_try_from_operation_payload_with_serialize_deserialize_error_named_camel_case_token_stream {
                    #not_uuid_token_camel_case_stream {
                        #eo_error_occurence_attribute_token_stream
                        #not_uuid_token_lower_case_stream: #crate_server_postgres_uuid_wrapper_uuid_wrapper_try_from_possible_uuid_wrapper_error_named_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                }
            }
        };
        // println!("{operation_payload_try_from_operation_payload_with_serialize_deserialize_error_named_token_stream}");
        let impl_std_convert_try_from_operation_payload_with_serialize_deserialize_for_operation_payload_token_stream = {
            let fields_assignments_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    let #field_ident = value.#field_ident;
                }
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
                impl std::convert::TryFrom<#operation_payload_with_serialize_deserialize_camel_case_token_stream> for #operation_payload_camel_case_token_stream {
                    type Error = #operation_payload_try_from_operation_payload_with_serialize_deserialize_error_named_camel_case_token_stream;
                    fn try_from(value: #operation_payload_with_serialize_deserialize_camel_case_token_stream) -> Result<Self, Self::Error> {
                        let #id_field_ident = match value.#id_field_ident {
                            Some(value) => match value.into_iter().map(|element|#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream::try_from(element)).collect::<Result<
                                std::vec::Vec<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>,                                    
                                #crate_server_postgres_uuid_wrapper_uuid_wrapper_try_from_possible_uuid_wrapper_error_named_token_stream
                            >>() {
                                Ok(value) => Some(value),
                                Err(e) => {
                                    return Err(Self::Error::#not_uuid_token_camel_case_stream {
                                        #not_uuid_token_lower_case_stream: e,
                                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                    });
                                },
                            },
                            None => None,
                        };
                        #(#fields_assignments_token_stream)*
                        Ok(Self {
                            #(#self_init_fields_token_stream),*
                        })
                    }
                }
            }
        };
        //
        let impl_std_convert_from_operation_payload_for_operation_payload_with_serialize_deserialize_token_stream = {
            let fields_assignments_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                let field_ident = element.field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                quote::quote!{
                    let #field_ident = value.#field_ident;
                }
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
                impl std::convert::From<#operation_payload_camel_case_token_stream> for #operation_payload_with_serialize_deserialize_camel_case_token_stream {
                    fn from(value: #operation_payload_camel_case_token_stream) -> Self {
                        let #id_field_ident = match value.#id_field_ident {
                            Some(value) => Some(value.into_iter()
                                .map(|element|#crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream::from(element))
                                .collect::<std::vec::Vec<#crate_server_postgres_uuid_wrapper_possible_uuid_wrapper_token_stream>>()),
                            None => None,
                        };
                        #(#fields_assignments_token_stream)*
                        Self{
                            #(#self_init_fields_token_stream),*
                        }
                    }
                }
            }
        };
        //
        let try_operation_error_named_token_stream = {
            let try_operation_request_error_camel_case_token_stream = generate_try_operation_request_error_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            quote::quote!{
                #error_named_derive_token_stream
                pub enum #try_operation_error_named_camel_case_token_stream {
                    #request_error_camel_case_token_stream {
                        #eo_error_occurence_attribute_token_stream
                        #request_error_lower_case_token_stream: #try_operation_request_error_camel_case_token_stream,
                        #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                    },
                    #http_request_error_named_serde_json_to_string_variant_token_stream,
                }
            }
        };
        // println!("{try_operation_error_named_token_stream}");
        let try_operation_error_with_middleware_error_variants_token_stream = {
            let desirable_attribute = proc_macro_helpers::attribute::Attribute::Tvfrr200Ok;
            let try_operation_response_variants_camel_case_stringified = generate_try_operation_response_variants_camel_case_stringified(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified
            );
            let try_operation_response_variants_camel_case_token_stream = generate_try_operation_response_variants_camel_case_token_stream(
                &try_operation_response_variants_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_camel_case_token_stream = generate_try_operation_with_serialize_deserialize_camel_case_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_response_variants_desirable_attribute_token_stream = generate_try_operation_response_variants_desirable_attribute_token_stream(
                &try_camel_case_stringified,
                &operation_name_camel_case_stringified,
                &response_variants_camel_case_stringified,
                &desirable_attribute,
                &proc_macro_name_ident_stringified
            );
            let try_operation_request_error_token_stream = generate_try_operation_request_error_token_stream(
                &try_operation_camel_case_stringified,
                &request_error_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let try_operation_with_serialize_deserialize_token_stream = generate_try_operation_with_serialize_deserialize_token_stream(
                &try_operation_camel_case_stringified,
                &with_serialize_deserialize_camel_case_stringified,
                &proc_macro_name_ident_stringified
            );
            let common_error_variant_attribute_vec_handle = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &common_error_variant_attribute_vec
            );
            let json_body_logic_error_variants_vec_token_stream = generate_error_variants_vec_token_stream(
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream, //KekwWithSerializeDeserialize
                &proc_macro_name_ident_stringified,
                &json_body_logic_error_variants_vec_handle
            );
            let type_variants_from_request_response_vec = {
                let mut type_variants_from_request_response = std::vec::Vec::new();//todo calculate capacity
                for element in common_error_variant_attribute_vec_handle {
                    type_variants_from_request_response.push(element);
                }
                for element in json_body_logic_error_variants_vec_token_stream  {
                    type_variants_from_request_response.push(element);
                }
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &not_unique_primary_key_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &not_unique_name_vec_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &not_unique_color_vec_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &bind_query_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &no_payload_fields_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &no_payload_parameters_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &non_existing_primary_keys_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &non_existing_primary_keys_and_failed_rollback_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &primary_key_from_row_and_failed_rollback_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &commit_failed_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &query_and_rollback_failed_variant_attribute
                ));
                type_variants_from_request_response.push(crate::type_variants_from_request_response_generator::type_variants_from_request_response(
                    &try_operation_response_variants_camel_case_token_stream,
                    &try_operation_with_serialize_deserialize_camel_case_token_stream,
                    &proc_macro_name_ident_stringified,
                    &delete_many_with_body_payload_try_from_delete_many_with_body_payload_with_serialize_deserialize_variant_attribute
                ));
                type_variants_from_request_response
            };
            crate::type_variants_from_request_response_generator::type_variants_from_request_response_generator(
                desirable_attribute,
                &desirable_token_stream,
                &quote::quote!{()},
                &try_operation_camel_case_token_stream,
                &try_operation_response_variants_camel_case_stringified,
                &try_operation_response_variants_camel_case_token_stream,
                &try_operation_response_variants_desirable_attribute_token_stream,
                &try_operation_with_serialize_deserialize_camel_case_token_stream,
                &try_operation_request_error_token_stream,
                &try_operation_with_serialize_deserialize_token_stream,
                &operation_name_lower_case_stringified,
                &code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                &code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                &error_named_derive_token_stream,
                &eo_display_token_stream,
                &eo_display_foreign_type_token_stream,
                &eo_display_with_serialize_deserialize_token_stream,
                &derive_debug_serialize_deserialize_token_stream,
                type_variants_from_request_response_vec,
                false,
                &proc_macro_name_ident_stringified,
            )
        };
        // println!("{try_operation_error_with_middleware_error_variants_token_stream}");
        let http_request_token_stream = {
            let try_operation_lower_case_token_stream = {
                let try_operation_lower_case_stringified = format!("{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
                try_operation_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
            };
            let tvfrr_extraction_logic_token_stream = {
                let tvfrr_extraction_logic_stringified = format!("{tvfrr_extraction_logic_lower_case_stringified}_{try_lower_case_stringified}_{operation_name_lower_case_stringified}");
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
                pub async fn #try_operation_lower_case_token_stream<'a>(
                    #server_location_name_token_stream: #server_location_type_token_stream,
                    #parameters_lower_case_token_stream: #operation_parameters_camel_case_token_stream,
                ) -> Result<(), #try_operation_error_named_camel_case_token_stream> {
                    let #payload_lower_case_token_stream = match #serde_json_to_string_token_stream(
                        &#operation_payload_with_serialize_deserialize_camel_case_token_stream::from(#parameters_lower_case_token_stream.#payload_lower_case_token_stream)
                    ) {
                        Ok(value) => value,
                        Err(e) => {
                            return Err(#try_operation_error_named_camel_case_token_stream::#serde_json_to_string_variant_initialization_token_stream);
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
                        Err(e) => Err(#try_operation_error_named_camel_case_token_stream::#request_error_variant_initialization_token_stream),
                    }
                }
            }
        };
        // println!("{http_request_token_stream}");
        let route_handler_token_stream = {
            let operation_lower_case_token_stream = operation_name_lower_case_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {operation_name_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
            let try_operation_token_stream = {
                let from_log_and_return_error_token_stream = crate::from_log_and_return_error::from_log_and_return_error(
                    &try_operation_camel_case_token_stream,
                    &error_log_call_token_stream,
                    &try_operation_response_variants_token_stream,
                );
                let check_for_none_token_stream = crate::check_for_none::check_for_none(
                    &fields_named,
                    &id_field,
                    &proc_macro_name_ident_stringified,
                    dot_space,
                    &try_operation_response_variants_token_stream,
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
                                let mut vec = std::vec::Vec::with_capacity(#id_field_ident.len());
                                let mut #not_unique_primary_keys_name_token_stream = std::vec::Vec::with_capacity(#id_field_ident.len());
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
                                let error = #try_operation_camel_case_token_stream::#not_unique_primary_key_variant_initialization_token_stream;
                                #error_log_call_token_stream
                                return #try_operation_response_variants_token_stream::from(error);
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
                        .collect::<std::vec::Vec<#crate_server_postgres_uuid_wrapper_uuid_wrapper_token_stream>>()
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
                            .map(|element| element.clone().into_inner())
                            .collect::<std::vec::Vec<#sqlx_types_uuid_token_stream>>()
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
                        &query_and_rollback_failed_variant_initialization_token_stream,
                        &primary_key_uuid_wrapper_try_from_sqlx_row_name_token_stream,
                        &from_log_and_return_error_token_stream,
                        &rollback_error_name_token_stream,
                        &primary_key_from_row_and_failed_rollback_variant_initialization_token_stream,
                        &non_existing_primary_keys_name_token_stream,
                        &expected_updated_primary_keys_name_token_stream,
                        &primary_key_vec_name_token_stream,
                        &rollback_token_stream,
                        &non_existing_primary_keys_variant_initialization_token_stream,
                        &non_existing_primary_keys_and_failed_rollback_variant_initialization_token_stream,
                        &postgres_transaction_token_stream,
                        &commit_token_stream,
                        &try_operation_response_variants_token_stream,
                        &desirable_token_stream,
                        &try_operation_camel_case_token_stream,
                        &commit_failed_variant_initialization_token_stream,
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
                                    let mut vec = std::vec::Vec::with_capacity(#id_field_ident.len());
                                    let mut #not_unique_primary_keys_name_token_stream = std::vec::Vec::with_capacity(#id_field_ident.len());
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
                                    let error = #try_operation_camel_case_token_stream::#not_unique_primary_key_variant_initialization_token_stream;
                                    #error_log_call_token_stream
                                    return #try_operation_response_variants_token_stream::from(error);
                                }
                            }
                        };
                        let filter_unique_parameters_other_columns_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                            let field_ident = element.field.ident.clone()
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
                            quote::quote!{
                                let #field_handle_token_stream = match #parameters_lower_case_token_stream.#payload_lower_case_token_stream.#field_ident {
                                    Some(value) => {
                                        let is_unique = {
                                            let mut vec = std::vec::Vec::with_capacity(value.len());
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
                                                    let mut vec = std::vec::Vec::with_capacity(value.len());
                                                    let mut #not_unique_field_vec_lower_case_token_stream = std::vec::Vec::with_capacity(value.len());
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
                                                let error = #try_operation_camel_case_token_stream::#not_unique_field_vec_vec_pascal_token_stream {
                                                    #not_unique_field_vec_lower_case_token_stream,
                                                    #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                                };
                                                #error_log_call_token_stream
                                                return #try_operation_response_variants_token_stream::from(error);
                                            }
                                        }
                                    },
                                    None => None
                                };
                            }
                        });
                        quote::quote!{
                            #filter_unique_parameters_primary_key_token_stream
                            #(#filter_unique_parameters_other_columns_token_stream)*
                        }
                    };
                    let query_string_token_stream = {
                        let additional_parameters_modification_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                            let field_ident = element.field.ident.clone()
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
                            quote::quote!{
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
                                            return #try_operation_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
                                        },
                                    }
                                }
                            }
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
                                            let mut additional_parameters = #std_string_string_token_stream::default();
                                            for element in #id_field_ident {
                                                match #crate_server_postgres_bind_query_bind_query_try_increment_token_stream(
                                                    element,
                                                    &mut increment,
                                                ) {
                                                    Ok(_) => {
                                                        additional_parameters.push_str(&format!("${increment},"));
                                                    }
                                                    Err(e) => {
                                                        return #try_operation_response_variants_token_stream::#bind_query_variant_initialization_token_stream;
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
                                    let mut additional_parameters = #std_string_string_token_stream::default();
                                    #(#additional_parameters_modification_token_stream)*
                                    #additional_parameters_id_modification_token_stream
                                    additional_parameters
                                }
                            )
                        }
                    };
                    let binded_query_token_stream = {
                        let binded_query_modifications_token_stream = fields_named_wrappers_excluding_primary_key.iter().map(|element|{
                            let field_ident = element.field.ident.clone()
                                .unwrap_or_else(|| {
                                    panic!("{proc_macro_name_ident_stringified} field.ident is None")
                                });
                            let field_handle_token_stream = {
                                let field_handle_stringified = format!("{field_ident}_handle");
                                field_handle_stringified.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {field_handle_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            };
                            quote::quote!{
                                if let Some(value) = #field_handle_token_stream {
                                    query = #crate_server_postgres_bind_query_bind_query_bind_value_to_query_token_stream(value, query);
                                }
                            }
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
                        &try_operation_response_variants_token_stream,
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
            // println!("{try_operation_token_stream}");
            quote::quote!{
                pub async fn #operation_lower_case_token_stream<'a>(
                    #app_info_state_name_token_stream: #axum_extract_state_token_stream<#app_info_state_path>,
                    #payload_extraction_result_lower_case_token_stream: Result<
                        #axum_json_token_stream<#operation_payload_with_serialize_deserialize_camel_case_token_stream>,
                        #axum_extract_rejection_json_rejection_token_stream,
                    >,
                ) -> #impl_axum_response_into_response_token_stream {
                    let #parameters_lower_case_token_stream = #operation_parameters_camel_case_token_stream {
                        #payload_lower_case_token_stream: match #crate_server_routes_helpers_json_extractor_error_json_value_result_extractor_token_stream::<
                            #operation_payload_with_serialize_deserialize_camel_case_token_stream,
                            #try_operation_response_variants_token_stream,
                        >::#try_extract_value_token_stream(#payload_extraction_result_lower_case_token_stream, &#app_info_state_name_token_stream)
                        {
                            Ok(value) => match #operation_payload_camel_case_token_stream::try_from(value) {
                                Ok(value) => value,
                                Err(e) => {
                                    let error = #try_operation_camel_case_token_stream::#operation_payload_try_from_operation_payload_with_serialize_deserialize_camel_case_token_stream {
                                        #operation_payload_try_from_operation_payload_with_serialize_deserialize_lower_case_token_stream: e,
                                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                                    };
                                    #error_log_call_token_stream
                                    return #try_operation_response_variants_token_stream::from(error);
                                }
                            },
                            Err(err) => {
                                return err;
                            }
                        },
                    };
                    println!("{:#?}", #parameters_lower_case_token_stream);
                    {
                        #try_operation_token_stream
                    }
                }
            }
        };
        // println!("{route_handler_token_stream}");
        quote::quote!{
            #parameters_token_stream
            #payload_token_stream
            #operation_payload_with_serialize_deserialize_token_stream
            #operation_payload_try_from_operation_payload_with_serialize_deserialize_error_named_token_stream
            #impl_std_convert_try_from_operation_payload_with_serialize_deserialize_for_operation_payload_token_stream
            #impl_std_convert_from_operation_payload_for_operation_payload_with_serialize_deserialize_token_stream
            #try_operation_error_named_token_stream
            #try_operation_error_with_middleware_error_variants_token_stream
            #http_request_token_stream
            #route_handler_token_stream
        }
    };
    // proc_macro_helpers::write_token_stream_into_file::write_token_stream_into_file(
    //     &proc_macro_name,
    //     &delete_many_with_body_token_stream,
    //     &proc_macro_name_ident_stringified
    // );
    let common_token_stream = quote::quote! {
        #table_name_declaration_token_stream
        #struct_options_token_stream
        #from_ident_for_ident_options_token_stream
        #(#structs_variants_token_stream)*
        #(#impl_std_convert_try_from_ident_options_for_struct_variants_token_stream)*
        #column_token_stream
        #column_select_token_stream
        #primary_key_uuid_wrapper_try_from_sqlx_row_token_stream
        #deserialize_ident_order_by_token_stream
        #order_by_wrapper_token_stream
        #allow_methods_token_stream
        #ident_column_read_permission_token_stream
    };
    // proc_macro_helpers::write_token_stream_into_file::write_token_stream_into_file(
    //     &proc_macro_name,
    //     &common_token_stream,
    //     &proc_macro_name_ident_stringified
    // );
    let gen = quote::quote! {
        #common_token_stream

        #create_many_token_stream
        #create_one_token_stream
        #read_one_token_stream
        #read_many_with_body_token_stream
        #update_one_token_stream
        #update_many_token_stream
        #delete_one_token_stream
        #delete_many_with_body_token_stream
    };
    // if ident == "" {
    //    println!("{gen}");
    // }
    gen.into()
}

fn generate_try_operation_camel_case_stringified(
    try_camel_case_stringified: &str,
    operation_name_camel_case_stringified: &str,
) -> std::string::String {
    format!("{try_camel_case_stringified}{operation_name_camel_case_stringified}")
}

fn generate_try_operation_camel_case_token_stream(
    try_operation_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    try_operation_camel_case_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_try_operation_response_variants_token_stream(
    try_camel_case_stringified: &str,
    operation_name_camel_case_stringified: &str,
    response_variants_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = format!("{try_camel_case_stringified}{operation_name_camel_case_stringified}{response_variants_camel_case_stringified}");
    value.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_try_operation_error_named_camel_case_token_stream(
    try_camel_case_stringified: &str,
    operation_name_camel_case_stringified: &str,
    error_named_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = format!("{try_camel_case_stringified}{operation_name_camel_case_stringified}{error_named_camel_case_stringified}");
    value.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_operation_parameters_camel_case_token_stream(
    operation_name_camel_case_stringified: &str,
    parameters_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = format!("{operation_name_camel_case_stringified}{parameters_camel_case_stringified}");
    value.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_path_camel_case_token_stream(
    original_name_camel_case_stringified: &str,
    path_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = format!("{original_name_camel_case_stringified}{path_camel_case_stringified}");
    value.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE)) 
}

fn generate_path_with_serialize_deserialize_camel_case_token_stream(
    original_name_camel_case_stringified: &str,
    path_camel_case_stringified: &str,
    with_serialize_deserialize_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str,
) -> proc_macro2::TokenStream {
    let value = format!("{original_name_camel_case_stringified}{path_camel_case_stringified}{with_serialize_deserialize_camel_case_stringified}");
    value.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_path_try_from_path_with_serialize_deserialize_stringified(
    original_name_camel_case_stringified: &str,
    path_camel_case_stringified: &str,
    try_from_camel_case_stringified: &str,
    with_serialize_deserialize_camel_case_stringified: &str,
) -> std::string::String {
    format!("{original_name_camel_case_stringified}{path_camel_case_stringified}{try_from_camel_case_stringified}{original_name_camel_case_stringified}{path_camel_case_stringified}{with_serialize_deserialize_camel_case_stringified}")
}

fn generate_path_try_from_path_with_serialize_deserialize_error_named_token_stream(
    path_try_from_path_with_serialize_deserialize_camel_case_stringified: &str,
    error_named_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = format!("{path_try_from_path_with_serialize_deserialize_camel_case_stringified}{error_named_camel_case_stringified}");
    value.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_path_try_from_path_with_serialize_deserialize_camel_case_token_stream(
    path_try_from_path_with_serialize_deserialize_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    path_try_from_path_with_serialize_deserialize_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {path_try_from_path_with_serialize_deserialize_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_path_try_from_path_with_serialize_deserialize_lower_case_token_stream(
    path_try_from_path_with_serialize_deserialize_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&path_try_from_path_with_serialize_deserialize_camel_case_stringified.to_string());
    value.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_query_camel_case_token_stream(
    original_name_camel_case_stringified: &str,
    query_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = format!("{original_name_camel_case_stringified}{query_camel_case_stringified}");
    value.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE)) 
}

fn generate_query_with_serialize_deserialize_camel_case_token_stream(
    original_name_camel_case_stringified: &str,
    query_camel_case_stringified: &str,
    with_serialize_deserialize_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = format!("{original_name_camel_case_stringified}{query_camel_case_stringified}{with_serialize_deserialize_camel_case_stringified}");
    value.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE)) 
}

fn generate_operation_payload_element_camel_case_token_stream(
    operation_name_camel_case_stringified: &str,
    payload_element_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = format!("{operation_name_camel_case_stringified}{payload_element_camel_case_stringified}");
    value.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))   
}

fn generate_operation_payload_camel_case_token_stream(
    operation_name_camel_case_stringified: &str,
    payload_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = format!("{operation_name_camel_case_stringified}{payload_camel_case_stringified}");
    value.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

//
fn generate_try_operation_request_error_camel_case_token_stream(
    try_camel_case_stringified: &str,
    operation_name_camel_case_stringified: &str,
    request_error_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let try_operation_request_error_camel_case_stringified = format!("{try_camel_case_stringified}{operation_name_camel_case_stringified}{request_error_camel_case_stringified}");
    try_operation_request_error_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_request_error_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}
//

fn generate_payload_with_serialize_deserialize_camel_case_token_stream(
    original_name_camel_case_stringified: &str,
    payload_camel_case_stringified: &str,
    with_serialize_deserialize_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = format!("{original_name_camel_case_stringified}{payload_camel_case_stringified}{with_serialize_deserialize_camel_case_stringified}");
    value.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_payload_try_from_payload_with_serialize_deserialize_stringified(
    original_name_camel_case_stringified: &str,
    payload_camel_case_stringified: &str,
    try_from_camel_case_stringified: &str,
    with_serialize_deserialize_camel_case_stringified: &str,
) -> std::string::String {
    format!("{original_name_camel_case_stringified}{payload_camel_case_stringified}{try_from_camel_case_stringified}{original_name_camel_case_stringified}{payload_camel_case_stringified}{with_serialize_deserialize_camel_case_stringified}")
}

fn generate_payload_try_from_payload_with_serialize_deserialize_error_named_camel_case_token_stream(
    payload_try_from_payload_with_serialize_deserialize_camel_case_stringified: &str,
    error_named_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = format!("{payload_try_from_payload_with_serialize_deserialize_camel_case_stringified}{error_named_camel_case_stringified}");
    value.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))   
}

fn generate_payload_try_from_payload_with_serialize_deserialize_camel_case_token_stream(
    payload_try_from_payload_with_serialize_deserialize_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    payload_try_from_payload_with_serialize_deserialize_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {payload_try_from_payload_with_serialize_deserialize_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_payload_try_from_payload_with_serialize_deserialize_lower_case_token_stream(
    payload_try_from_payload_with_serialize_deserialize_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&payload_try_from_payload_with_serialize_deserialize_camel_case_stringified);
    value.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_payload_element_with_serialize_deserialize_camel_case_token_stream(
    original_name_camel_case_stringified: &str,
    payload_element_with_serialize_deserialize_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = format!("{original_name_camel_case_stringified}{payload_element_with_serialize_deserialize_camel_case_stringified}");
    value.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_payload_element_try_from_payload_element_with_serialize_deserialize_camel_case_stringified(
    original_name_camel_case_stringified: &str,
    payload_element_camel_case_stringified: &str,
    try_from_camel_case_stringified: &str,
    payload_element_with_serialize_deserialize_camel_case_stringified: &str
) -> std::string::String {
    format!(
        "{original_name_camel_case_stringified}{payload_element_camel_case_stringified}{try_from_camel_case_stringified}{original_name_camel_case_stringified}{payload_element_with_serialize_deserialize_camel_case_stringified}"
    )
}

fn generate_payload_element_try_from_payload_element_with_serialize_deserialize_camel_case_token_stream(
    payload_element_try_from_payload_element_with_serialize_deserialize_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    payload_element_try_from_payload_element_with_serialize_deserialize_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {payload_element_try_from_payload_element_with_serialize_deserialize_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_payload_element_try_from_payload_element_with_serialize_deserialize_lower_case_token_stream(
    payload_element_try_from_payload_element_with_serialize_deserialize_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&payload_element_try_from_payload_element_with_serialize_deserialize_camel_case_stringified);
    value.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_payload_element_try_from_payload_element_with_serialize_deserialize_error_named_camel_case_token_stream(
    payload_element_try_from_payload_element_with_serialize_deserialize_camel_case_stringified: &str,
    error_named_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let value = format!(
        "{payload_element_try_from_payload_element_with_serialize_deserialize_camel_case_stringified}{error_named_camel_case_stringified}"
    );
    value.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {value} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

//
fn generate_try_operation_response_variants_camel_case_stringified(
    try_camel_case_stringified: &str,
    operation_name_camel_case_stringified: &str,
    response_variants_camel_case_stringified: &str
) -> std::string::String {
    format!("{try_camel_case_stringified}{operation_name_camel_case_stringified}{response_variants_camel_case_stringified}")
}

fn generate_try_operation_response_variants_camel_case_token_stream(
    try_operation_response_variants_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    try_operation_response_variants_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_response_variants_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_try_operation_with_serialize_deserialize_camel_case_token_stream(
    try_camel_case_stringified: &str,
    operation_name_camel_case_stringified: &str,
    with_serialize_deserialize_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let try_operation_with_serialize_deserialize_camel_case_stringified = format!("{try_camel_case_stringified}{operation_name_camel_case_stringified}{with_serialize_deserialize_camel_case_stringified}");
    try_operation_with_serialize_deserialize_camel_case_stringified.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_with_serialize_deserialize_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_try_operation_response_variants_desirable_attribute_token_stream(
    try_camel_case_stringified: &str,
    operation_name_camel_case_stringified: &str,
    response_variants_camel_case_stringified: &str,
    desirable_attribute: &proc_macro_helpers::attribute::Attribute,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let try_operation_response_variants_desirable_attribute_stringified =
        format!("{try_camel_case_stringified}{operation_name_camel_case_stringified}{response_variants_camel_case_stringified}{desirable_attribute}");
    try_operation_response_variants_desirable_attribute_stringified
    .parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_response_variants_desirable_attribute_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_try_operation_request_error_token_stream(
    try_operation_camel_case_stringified: &str,
    request_error_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let try_operation_request_error_stringified = format!("{try_operation_camel_case_stringified}{request_error_camel_case_stringified}");
    try_operation_request_error_stringified
    .parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_request_error_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

fn generate_try_operation_with_serialize_deserialize_token_stream(
    try_operation_camel_case_stringified: &str,
    with_serialize_deserialize_camel_case_stringified: &str,
    proc_macro_name_ident_stringified: &str
) -> proc_macro2::TokenStream {
    let try_operation_with_serialize_deserialize_stringified =
        format!("{try_operation_camel_case_stringified}{with_serialize_deserialize_camel_case_stringified}");
    try_operation_with_serialize_deserialize_stringified
    .parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_with_serialize_deserialize_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
}

#[derive(
    Debug, 
    enum_extension::EnumExtension,
    strum_macros::EnumIter,
    PartialEq,
    Eq,
)]
enum SupportedAttributeType {
    // Bool,
    // Bytea,
    // Char,
    // Name,
    // Int8,
    // Int2,
    // Int4,
    // Text,
    // Oid,
    // Json,
    // JsonArray,
    // Point,
    // Lseg,
    // Path,
    // Box,
    // Polygon,
    // Line,
    // LineArray,
    // Cidr,
    // CidrArray,
    // Float4,
    // Float8,
    // Unknown,
    // Circle,
    // CircleArray,
    // Macaddr8,
    // Macaddr8Array,
    // Macaddr,
    // Inet,
    // BoolArray,
    // ByteaArray,
    // CharArray,
    // NameArray,
    // Int2Array,
    // Int4Array,
    // TextArray,
    // BpcharArray,
    // VarcharArray,
    // Int8Array,
    // PointArray,
    // LsegArray,
    // PathArray,
    // BoxArray,
    // Float4Array,
    // Float8Array,
    // PolygonArray,
    // OidArray,
    // MacaddrArray,
    // InetArray,
    // Bpchar,
    // Varchar,
    // Date,
    // Time,
    // Timestamp,
    // TimestampArray,
    // DateArray,
    // TimeArray,
    // Timestamptz,
    // TimestamptzArray,
    // Interval,
    // IntervalArray,
    // NumericArray,
    // Timetz,
    // TimetzArray,
    // Bit,
    // BitArray,
    // Varbit,
    // VarbitArray,
    // Numeric,
    // Record,
    // RecordArray,
    // Uuid,
    // UuidArray,
    // Jsonb,
    // JsonbArray,
    // Int4Range,
    // Int4RangeArray,
    // NumRange,
    // NumRangeArray,
    // TsRange,
    // TsRangeArray,
    // TstzRange,
    // TstzRangeArray,
    // DateRange,
    // DateRangeArray,
    // Int8Range,
    // Int8RangeArray,
    // Jsonpath,
    // JsonpathArray,
    // Money,
    // MoneyArray,

    // // https://www.postgresql.org/docs/9.3/datatype-pseudo.html
    // Void,

    // // A realized user-defined type. When a connection sees a DeclareXX variant it resolves
    // // into this one before passing it along to `accepts` or inside of `Value` objects.
    // Custom(Arc<PgCustomType>),

    // // From [`PgTypeInfo::with_name`]
    // DeclareWithName(UStr),

    // // NOTE: Do we want to bring back type declaration by ID? It's notoriously fragile but
    // //       someone may have a user for it
    // DeclareWithOid(Oid),

    /////////////////////////////
    Bool,
    Char,
    Smallint,
    Smallserial,
    Int2,
    Int,
    Serial, 
    Int4,
    Bigint,
    Bigserial, 
    Int8,
    Real, 
    Float4,
    DoublePrecision,
    Float8,
    Varchar,
    Charn, //CHAR(N) wtf????
    Text,
    Name,
    Bytea,
    Void,
    Interval,
    Int8range,
    Int4range,
    Tsrange,
    Tstzrange,
    Daterange,
    Numrange,
    Money,
    Ltree,
    Lquery,

    Numeric,

    Timestamptz,
    Timestamp,
    Date,
    Time,
    Timetz,
    Uuid,

    Inet,
    Cidr,

    Macaddr,

    Bit,
    Varbit,

    Json,
    Jsonb
}

impl std::fmt::Display for SupportedAttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Bool => write!(f, "generate_postgresql_crud_bool"),
            Self::Char => write!(f, "generate_postgresql_crud_char"),
            Self::Smallint => write!(f, "generate_postgresql_crud_smallint"),
            Self::Smallserial => write!(f, "generate_postgresql_crud_smallserial"),
            Self::Int2 => write!(f, "generate_postgresql_crud_int2"),
            Self::Int => write!(f, "generate_postgresql_crud_int"),
            Self::Serial => write!(f, "generate_postgresql_crud_serial"), 
            Self::Int4 => write!(f, "generate_postgresql_crud_int4"),
            Self::Bigint => write!(f, "generate_postgresql_crud_bigint"),
            Self::Bigserial => write!(f, "generate_postgresql_crud_bigserial"), 
            Self::Int8 => write!(f, "generate_postgresql_crud_int8"),
            Self::Real => write!(f, "generate_postgresql_crud_real"), 
            Self::Float4 => write!(f, "generate_postgresql_crud_float4"),
            Self::DoublePrecision => write!(f, "generate_postgresql_crud_double_precision"),
            Self::Float8 => write!(f, "generate_postgresql_crud_float8"),
            Self::Varchar => write!(f, "generate_postgresql_crud_varchar"),
            Self::Charn => write!(f, "generate_postgresql_crud_charn"), //CHAR(N) wtf????
            Self::Text => write!(f, "generate_postgresql_crud_text"),
            Self::Name => write!(f, "generate_postgresql_crud_name"),
            Self::Bytea => write!(f, "generate_postgresql_crud_bytea"),
            Self::Void => write!(f, "generate_postgresql_crud_void"),
            Self::Interval => write!(f, "generate_postgresql_crud_interval"),
            Self::Int8range => write!(f, "generate_postgresql_crud_int8range"),
            Self::Int4range => write!(f, "generate_postgresql_crud_int4range"),
            Self::Tsrange => write!(f, "generate_postgresql_crud_tsrange"),
            Self::Tstzrange => write!(f, "generate_postgresql_crud_tstzrange"),
            Self::Daterange => write!(f, "generate_postgresql_crud_daterange"),
            Self::Numrange => write!(f, "generate_postgresql_crud_numrange"),
            Self::Money => write!(f, "generate_postgresql_crud_money"),
            Self::Ltree => write!(f, "generate_postgresql_crud_ltree"),
            Self::Lquery => write!(f, "generate_postgresql_crud_lquery"),

            Self::Numeric => write!(f, "generate_postgresql_crud_numeric"),

            Self::Timestamptz => write!(f, "generate_postgresql_crud_timestamptz"),
            Self::Timestamp => write!(f, "generate_postgresql_crud_timestamp"),
            Self::Date => write!(f, "generate_postgresql_crud_date"),
            Self::Time => write!(f, "generate_postgresql_crud_time"),
            Self::Timetz => write!(f, "generate_postgresql_crud_timetz"),
            Self::Uuid => write!(f, "generate_postgresql_crud_uuid"),

            Self::Inet => write!(f, "generate_postgresql_crud_inet"),
            Self::Cidr => write!(f, "generate_postgresql_crud_cidr"),

            Self::Macaddr => write!(f, "generate_postgresql_crud_macaddr"),

            Self::Bit => write!(f, "generate_postgresql_crud_bit"),
            Self::Varbit => write!(f, "generate_postgresql_crud_varbit"),

            Self::Json => write!(f, "generate_postgresql_crud_json"),
            Self::Jsonb => write!(f, "generate_postgresql_crud_jsonb"),
        }
    }
}

fn try_match_supported_attribute_type_with_supported_field_type(
    supported_attribute_type: &SupportedAttributeType,
    supported_field_type: &SupportedFieldType,
) -> bool {
    match (supported_attribute_type, supported_field_type) {
        (SupportedAttributeType::Bool, SupportedFieldType::StdPrimitiveBool) => true,
        (SupportedAttributeType::Char, SupportedFieldType::StdPrimitiveI8) => true,
        (SupportedAttributeType::Smallint, SupportedFieldType::StdPrimitiveI16) => true,
        (SupportedAttributeType::Smallserial, SupportedFieldType::StdPrimitiveI16) => true,
        (SupportedAttributeType::Int2, SupportedFieldType::StdPrimitiveI16) => true,
        (SupportedAttributeType::Int, SupportedFieldType::StdPrimitiveI32) => true,
        (SupportedAttributeType::Serial, SupportedFieldType::StdPrimitiveI16) => true,
        (SupportedAttributeType::Int4, SupportedFieldType::StdPrimitiveI16) => true,
        (SupportedAttributeType::Bigint, SupportedFieldType::StdPrimitiveI64) => true,
        (SupportedAttributeType::Bigserial, SupportedFieldType::StdPrimitiveI64) => true,
        (SupportedAttributeType::Int8, SupportedFieldType::StdPrimitiveI64) => true,
        (SupportedAttributeType::Real, SupportedFieldType::StdPrimitiveF32) => true,
        (SupportedAttributeType::Float4, SupportedFieldType::StdPrimitiveF32) => true,
        (SupportedAttributeType::DoublePrecision, SupportedFieldType::StdPrimitiveF64) => true,
        (SupportedAttributeType::Float8, SupportedFieldType::StdPrimitiveF64) => true,
        (SupportedAttributeType::Varchar, SupportedFieldType::StdPrimitiveStr) => true,
        (SupportedAttributeType::Varchar, SupportedFieldType::StdStringString) => true,
        (SupportedAttributeType::Charn, SupportedFieldType::StdPrimitiveStr) => true, //CHAR(N) wtf????
        (SupportedAttributeType::Charn, SupportedFieldType::StdStringString) => true, //CHAR(N) wtf????
        (SupportedAttributeType::Text, SupportedFieldType::StdPrimitiveStr) => true,
        (SupportedAttributeType::Text, SupportedFieldType::StdStringString) => true,
        (SupportedAttributeType::Name, SupportedFieldType::StdPrimitiveStr) => true,
        (SupportedAttributeType::Name, SupportedFieldType::StdStringString) => true,
        (SupportedAttributeType::Bytea, SupportedFieldType::StdPrimitiveArrayStdPrimitiveU8) => {
            true
        }
        (SupportedAttributeType::Bytea, SupportedFieldType::StdVecVecStdPrimitiveU8) => true,
        (SupportedAttributeType::Void, SupportedFieldType::StdPrimitiveUnit) => true,
        (SupportedAttributeType::Interval, SupportedFieldType::SqlxPostgresTypesPgInterval) => true,
        //
        // (SupportedAttributeType::Int8range, SupportedFieldType::) => true,
        // (SupportedAttributeType::Int4range, SupportedFieldType::) => true,
        // (SupportedAttributeType::Tsrange, SupportedFieldType::) => true,
        // (SupportedAttributeType::Tstzrange, SupportedFieldType::) => true,
        // (SupportedAttributeType::Daterange, SupportedFieldType::) => true,
        // (SupportedAttributeType::Numrange, SupportedFieldType::) => true,
        //
        // SqlxPostgresTypesPgRangeStdPrimitiveI32,
        // SqlxPostgresTypesPgRangeStdPrimitiveI64,
        // SqlxPostgresTypesPgRangeSqlxTypesDecimal,
        // SqlxPostgresTypesPgRangeSqlxTypesBigDecimal,
        // SqlxPostgresTypesPgRangeSqlxTypesTimeDate,
        // SqlxPostgresTypesPgRangeSqlxTypesChronoNaiveDate,
        // SqlxPostgresTypesPgRangeSqlxTypesChronoDateTimeSqlxTypesChronoFixedOffset,
        // SqlxPostgresTypesPgRangeSqlxTypesChronoDateTimeSqlxTypesChronoLocal,
        // SqlxPostgresTypesPgRangeSqlxTypesChronoDateTimeSqlxTypesChronoUtc,
        // SqlxPostgresTypesPgRangeSqlxTypesChronoNaiveDateTime,
        // SqlxPostgresTypesPgRangeSqlxTypesTimePrimitiveDateTime,
        // SqlxPostgresTypesPgRangeSqlxTypesTimeOffsetDateTime,
        //
        (SupportedAttributeType::Money, SupportedFieldType::SqlxPostgresTypesPgMoney) => true,
        (SupportedAttributeType::Ltree, SupportedFieldType::SqlxPostgresTypesPgLTree) => true,
        (SupportedAttributeType::Lquery, SupportedFieldType::SqlxPostgresTypesPgLQuery) => true,

        (SupportedAttributeType::Numeric, SupportedFieldType::SqlxTypesBigDecimal) => true,
        (SupportedAttributeType::Numeric, SupportedFieldType::SqlxTypesDecimal) => true,

        (
            SupportedAttributeType::Timestamptz,
            SupportedFieldType::SqlxTypesChronoDateTimeSqlxTypesChronoUtc,
        ) => true,
        (
            SupportedAttributeType::Timestamptz,
            SupportedFieldType::SqlxTypesChronoDateTimeSqlxTypesChronoLocal,
        ) => true,
        (SupportedAttributeType::Timestamptz, SupportedFieldType::SqlxTypesTimeOffsetDateTime) => {
            true
        }
        (SupportedAttributeType::Timestamp, SupportedFieldType::SqlxTypesChronoNaiveDateTime) => {
            true
        }
        (SupportedAttributeType::Timestamp, SupportedFieldType::SqlxTypesTimePrimitiveDateTime) => {
            true
        }
        (SupportedAttributeType::Date, SupportedFieldType::SqlxTypesChronoNaiveDate) => true,
        (SupportedAttributeType::Date, SupportedFieldType::SqlxTypesTimeDate) => true,
        (SupportedAttributeType::Time, SupportedFieldType::SqlxTypesChronoNaiveTime) => true,
        (SupportedAttributeType::Time, SupportedFieldType::SqlxTypesTimeTime) => true,
        // (SupportedAttributeType::Timetz, SupportedFieldType::) => true,//todo
        (SupportedAttributeType::Uuid, SupportedFieldType::SqlxTypesUuid) => true,

        (SupportedAttributeType::Inet, SupportedFieldType::SqlxTypesIpnetworkIpNetwork) => true,
        // (SupportedAttributeType::Inet, SupportedFieldType::IpAddr) => true,//todo
        (SupportedAttributeType::Cidr, SupportedFieldType::SqlxTypesIpnetworkIpNetwork) => true,
        // (SupportedAttributeType::Cidr, SupportedFieldType::IpAddr) => true,//todo
        (SupportedAttributeType::Macaddr, SupportedFieldType::SqlxTypesMacAddressMacAddress) => {
            true
        }
        (SupportedAttributeType::Bit, SupportedFieldType::SqlxTypesBitVecStdPrimitiveU32) => true, //maybe not correct
        (SupportedAttributeType::Varbit, SupportedFieldType::SqlxTypesBitVecStdPrimitiveU32) => {
            true
        } //maybe not correct
        // (SupportedAttributeType::Json, SupportedFieldType::) => true,//todo
        // (SupportedAttributeType::Jsonb, SupportedFieldType::) => true,//todo
        _ => false,
    }
}

impl std::str::FromStr for SupportedAttributeType {
    type Err = std::string::String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "generate_postgresql_crud_bool" => Ok(Self::Bool),
            "generate_postgresql_crud_char" => Ok(Self::Char),
            "generate_postgresql_crud_smallint" => Ok(Self::Smallint),
            "generate_postgresql_crud_smallserial" => Ok(Self::Smallserial),
            "generate_postgresql_crud_int2" => Ok(Self::Int2),
            "generate_postgresql_crud_int" => Ok(Self::Int),
            "generate_postgresql_crud_serial" => Ok(Self::Serial), 
            "generate_postgresql_crud_int4" => Ok(Self::Int4),
            "generate_postgresql_crud_bigint" => Ok(Self::Bigint),
            "generate_postgresql_crud_bigserial" => Ok(Self::Bigserial), 
            "generate_postgresql_crud_int8" => Ok(Self::Int8),
            "generate_postgresql_crud_real" => Ok(Self::Real), 
            "generate_postgresql_crud_float4" => Ok(Self::Float4),
            "generate_postgresql_crud_double_precision" => Ok(Self::DoublePrecision),
            "generate_postgresql_crud_float8" => Ok(Self::Float8),
            "generate_postgresql_crud_varchar" => Ok(Self::Varchar),
            "generate_postgresql_crud_charn" => Ok(Self::Charn), //CHAR(N) wtf????
            "generate_postgresql_crud_text" => Ok(Self::Text),
            "generate_postgresql_crud_name" => Ok(Self::Name),
            "generate_postgresql_crud_bytea" => Ok(Self::Bytea),
            "generate_postgresql_crud_void" => Ok(Self::Void),
            "generate_postgresql_crud_interval" => Ok(Self::Interval),
            "generate_postgresql_crud_int8range" => Ok(Self::Int8range),
            "generate_postgresql_crud_int4range" => Ok(Self::Int4range),
            "generate_postgresql_crud_tsrange" => Ok(Self::Tsrange),
            "generate_postgresql_crud_tstzrange" => Ok(Self::Tstzrange),
            "generate_postgresql_crud_daterange" => Ok(Self::Daterange),
            "generate_postgresql_crud_numrange" => Ok(Self::Numrange),
            "generate_postgresql_crud_money" => Ok(Self::Money),
            "generate_postgresql_crud_ltree" => Ok(Self::Ltree),
            "generate_postgresql_crud_lquery" => Ok(Self::Lquery),

            "generate_postgresql_crud_numeric" => Ok(Self::Numeric),

            "generate_postgresql_crud_timestamptz" => Ok(Self::Timestamptz),
            "generate_postgresql_crud_timestamp" => Ok(Self::Timestamp),
            "generate_postgresql_crud_date" => Ok(Self::Date),
            "generate_postgresql_crud_time" => Ok(Self::Time),
            "generate_postgresql_crud_timetz" => Ok(Self::Timetz),
            "generate_postgresql_crud_uuid" => Ok(Self::Uuid),

            "generate_postgresql_crud_inet" => Ok(Self::Inet),
            "generate_postgresql_crud_cidr" => Ok(Self::Cidr),

            "generate_postgresql_crud_macaddr" => Ok(Self::Macaddr),

            "generate_postgresql_crud_bit" => Ok(Self::Bit),
            "generate_postgresql_crud_varbit" => Ok(Self::Varbit),

            "generate_postgresql_crud_json" => Ok(Self::Json),
            "generate_postgresql_crud_jsonb" => Ok(Self::Jsonb),
            _ => Err(format!(
                "unsupported field attribute name: {value}, {:?}",
                Self::into_array().into_iter().map(|element|element.to_string()).collect::<std::vec::Vec<std::string::String>>()
            ))
        }
    }
}

// struct Field
//for now its better to check types manually to remove potential problems with token generation https://docs.rs/sqlx/0.7.2/sqlx/trait.Type.html
#[derive(
    Debug, 
    enum_extension::EnumExtension,
    strum_macros::EnumIter,
    PartialEq,
    Eq,
)]
enum SupportedFieldType {
    StdPrimitiveI16,
    StdPrimitiveStr,
    StdPrimitiveI64,
    StdPrimitiveI32,
    StdPrimitiveF64,
    StdPrimitiveF32,
    StdStringString,
    StdPrimitiveI8,
    StdPrimitiveBool,
    StdVecVecStdPrimitiveU8,
    StdPrimitiveArrayStdPrimitiveU8,
    StdPrimitiveUnit,
    //
    SqlxTypesDecimal,
    SqlxTypesBigDecimal,
    SqlxTypesTimeTime,
    SqlxTypesTimeDate,
    SqlxTypesChronoNaiveDate,
    SqlxTypesChronoNaiveDateTime,
    SqlxTypesChronoNaiveTime,
    SqlxTypesTimeOffsetDateTime,
    SqlxTypesTimePrimitiveDateTime,
    CoreTimeDuration,//todo maybe its std::time::Duration or core::time::Duration or both?
    SqlxTypesChronoDateTimeSqlxTypesChronoFixedOffset,
    SqlxTypesChronoDateTimeSqlxTypesChronoLocal,
    SqlxTypesChronoDateTimeSqlxTypesChronoUtc,
    SqlxTypesUuid,
    SqlxTypesIpnetworkIpNetwork,
    SqlxTypesMacAddressMacAddress,
    SqlxPostgresTypesPgInterval,
    SqlxPostgresTypesPgMoney,
    SqlxPostgresTypesPgLQuery,
    SqlxPostgresTypesPgLTree,
    SqlxPostgresTypesOid,
    SqlxTypesBitVecStdPrimitiveU32,
    SqlxPostgresTypesTimeTzPgTimeTzSqlxTypesTimeTimeSqlxTypesTimeUtcOffset,
    SqlxPostgresTypesTimeTzPgTimeTzSqlxTypesChronoNaiveTimeSqlxTypesChronoFixedOffset,
    StdBoxedBoxBorrowStdPrimitiveStr,//std::boxed::Box<&std::primitive::str>
    StdBorrowCowAnonymousLifetimeStdPrimitiveStr,//std::borrow::Cow<'_, std::primitive::str>
    //
    SqlxPostgresTypesPgRangeStdPrimitiveI32,
    SqlxPostgresTypesPgRangeStdPrimitiveI64,
    SqlxPostgresTypesPgRangeSqlxTypesDecimal,
    SqlxPostgresTypesPgRangeSqlxTypesBigDecimal,
    SqlxPostgresTypesPgRangeSqlxTypesTimeDate,
    SqlxPostgresTypesPgRangeSqlxTypesChronoNaiveDate,
    SqlxPostgresTypesPgRangeSqlxTypesChronoDateTimeSqlxTypesChronoFixedOffset,
    SqlxPostgresTypesPgRangeSqlxTypesChronoDateTimeSqlxTypesChronoLocal,
    SqlxPostgresTypesPgRangeSqlxTypesChronoDateTimeSqlxTypesChronoUtc,
    SqlxPostgresTypesPgRangeSqlxTypesChronoNaiveDateTime,
    SqlxPostgresTypesPgRangeSqlxTypesTimePrimitiveDateTime,
    SqlxPostgresTypesPgRangeSqlxTypesTimeOffsetDateTime,

    //todo this implementations

// impl Type<Postgres> for IpAddr // std::net::IpAddr
// where
//     IpNetwork: Type<Postgres>,
// impl<T, DB> Type<DB> for Option<T>
// where
//     T: Type<DB>,
//     DB: Database,

// impl<T, DB> Type<DB> for &T
// where
//     T: Type<DB> + ?Sized,
//     DB: Database,

// impl<DB> Type<DB> for Value
// where
//     Json<Value>: Type<DB>,
//     DB: Database,

// impl<DB> Type<DB> for RawValue
// where
//     Json<&'a RawValue>: for<'a> Type<DB>,
//     DB: Database,

// impl<T> Type<Postgres> for Vec<T, Global>
// where
//     T: PgHasArrayType,

// impl<T, const N: usize> Type<Postgres> for [T; N]
// where
//     T: PgHasArrayType,

// impl<T> Type<Postgres> for Json<T>

// impl<T> Type<Postgres> for [T]
// where
//     T: PgHasArrayType,

// impl<T1> Type<Postgres> for (T1,)

// impl<T1, T2> Type<Postgres> for (T1, T2)

// impl<T1, T2, T3> Type<Postgres> for (T1, T2, T3)

// impl<T1, T2, T3, T4> Type<Postgres> for (T1, T2, T3, T4)

// impl<T1, T2, T3, T4, T5> Type<Postgres> for (T1, T2, T3, T4, T5)

// impl<T1, T2, T3, T4, T5, T6> Type<Postgres> for (T1, T2, T3, T4, T5, T6)

// impl<T1, T2, T3, T4, T5, T6, T7> Type<Postgres> for (T1, T2, T3, T4, T5, T6, T7)

// impl<T1, T2, T3, T4, T5, T6, T7, T8> Type<Postgres> for (T1, T2, T3, T4, T5, T6, T7, T8)

// impl<T1, T2, T3, T4, T5, T6, T7, T8, T9> Type<Postgres> for (T1, T2, T3, T4, T5, T6, T7, T8, T9)

    // Bool,
    // StdPrimitiveI8,
    // StdPrimitiveI16,
    // StdPrimitiveI32,
    // StdPrimitiveI64,
    // StdPrimitiveF32,
    // StdPrimitiveF64,
    // BorrowStdPrimitiveStr, 
    // StdStringString,
    // BorrowU8Array, 
    // StdVecVecU8,
    // Unit,
    // SqlxPostgresTypesPgInterval,
    // SqlxPostgresTypesPgRangeGeneric,
    // SqlxPostgresTypesPgMoney,
    // SqlxPostgresTypesPgLTree,
    // SqlxPostgresTypesPgLQuery,
    // BigdecimalBigDecimal,
    // RustDecimalDecimal,

    // ChronoDateTimeUtcGeneric,
    // ChronoDateTimeLocalGeneric,
    // ChronoNaiveDateTime,
    // ChronoNaiveDate,
    // ChronoNaiveTime,
    // ChronoPgTimeTzArray,//todo find out from what crate this type


    // TimePrimitiveDateTime,
    // TimeOffsetDateTime,
    // TimeDate,
    // TimeTime,
    // TimePgTimeTzArray,//todo find out from what crate this type


    // UuidUuid,


    // IpnetworkIpNetwork,
    // StdNetIpAddr,

    // MacAddressMacAddress,

    // BitVecBitVec,

    // Json,//tood full path
    // SerdeJsonValue,
    // BorrowSerdeJsonValueRawValue
}

impl std::fmt::Display for SupportedFieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::StdPrimitiveI16 => write!(f, "std::primitive::i16"),
            Self::StdPrimitiveStr => write!(f, "std::primitive::str"),//todo borrow?
            Self::StdPrimitiveI64 => write!(f, "std::primitive::i64"),
            Self::StdPrimitiveI32 => write!(f, "std::primitive::i32"),
            Self::StdPrimitiveF64 => write!(f, "std::primitive::f64"),
            Self::StdPrimitiveF32 => write!(f, "std::primitive::f32"),
            Self::StdStringString => write!(f, "std::string::String"),
            Self::StdPrimitiveI8 => write!(f, "std::primitive::i8"),
            Self::StdPrimitiveBool => write!(f, "std::primitive::bool"),
            Self::StdVecVecStdPrimitiveU8 => write!(f, "std::vec::Vec<std::primitive::u8>"),
            Self::StdPrimitiveArrayStdPrimitiveU8 => write!(f, "[std::primitive::u8]"),
            Self::StdPrimitiveUnit => write!(f, "()"),

            Self::SqlxTypesDecimal => write!(f, "sqlx::types::Decimal"),
            Self::SqlxTypesBigDecimal => write!(f, "sqlx::types::BigDecimal"),
            Self::SqlxTypesTimeTime => write!(f, "sqlx::types::time::Time"),
            Self::SqlxTypesTimeDate => write!(f, "sqlx::types::time::Date"),
            Self::SqlxTypesChronoNaiveDate => write!(f, "sqlx::types::chrono::NaiveDate"),
            Self::SqlxTypesChronoNaiveDateTime => write!(f, "sqlx::types::chrono::NaiveDateTime"),
            Self::SqlxTypesChronoNaiveTime => write!(f, "sqlx::types::chrono::NaiveTime"),
            Self::SqlxTypesTimeOffsetDateTime => write!(f, "sqlx::types::time::OffsetDateTime"),
            Self::SqlxTypesTimePrimitiveDateTime => write!(f, "sqlx::types::time::PrimitiveDateTime"),
            Self::CoreTimeDuration => write!(f, "core::time::Duration"),//todo maybe its std::time::Duration or core::time::Duration or both?
            Self::SqlxTypesChronoDateTimeSqlxTypesChronoFixedOffset => write!(f, "sqlx::types::chrono::DateTime<sqlx::types::chrono::FixedOffset>"),
            Self::SqlxTypesChronoDateTimeSqlxTypesChronoLocal => write!(f, "sqlx::types::chrono::DateTime<sqlx::types::chrono::Local>"),
            Self::SqlxTypesChronoDateTimeSqlxTypesChronoUtc => write!(f, "sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>"),
            Self::SqlxTypesUuid => write!(f, "sqlx::types::Uuid"),
            Self::SqlxTypesIpnetworkIpNetwork => write!(f, "sqlx::types::ipnetwork::IpNetwork"),
            Self::SqlxTypesMacAddressMacAddress => write!(f, "sqlx::types::mac_address::MacAddress"),
            Self::SqlxPostgresTypesPgInterval => write!(f, "sqlx_postgres::types::PgInterval"),
            Self::SqlxPostgresTypesPgMoney => write!(f, "sqlx_postgres::types::PgMoney"),
            Self::SqlxPostgresTypesPgLQuery => write!(f, "sqlx_postgres::types::PgLQuery"),
            Self::SqlxPostgresTypesPgLTree => write!(f, "sqlx_postgres::types::PgLTree"),
            Self::SqlxPostgresTypesOid => write!(f, "sqlx_postgres::types::Oid"),
            Self::SqlxTypesBitVecStdPrimitiveU32 => write!(f, "sqlx::types::BitVec<std::primitive::u32>"),
            Self::SqlxPostgresTypesTimeTzPgTimeTzSqlxTypesTimeTimeSqlxTypesTimeUtcOffset => write!(f, "sqlx_postgres::types::time_tz::PgTimeTz<sqlx::types::time::Time,sqlx::types::time::UtcOffset>"),
            Self::SqlxPostgresTypesTimeTzPgTimeTzSqlxTypesChronoNaiveTimeSqlxTypesChronoFixedOffset => write!(f, "sqlx_postgres::types::time_tz::PgTimeTz<sqlx::types::chrono::NaiveTime,sqlx::types::chrono::FixedOffset>"),
            Self::StdBoxedBoxBorrowStdPrimitiveStr => write!(f, "std::boxed::Box<&std::primitive::str>"),
            Self::StdBorrowCowAnonymousLifetimeStdPrimitiveStr => write!(f, "std::borrow::Cow<'_, std::primitive::str>"),

            Self::SqlxPostgresTypesPgRangeStdPrimitiveI32 => write!(f, "sqlx_postgres::types::PgRange<std::primitive::i32>"),
            Self::SqlxPostgresTypesPgRangeStdPrimitiveI64 => write!(f, "sqlx_postgres::types::PgRange<std::primitive::i64>"),
            Self::SqlxPostgresTypesPgRangeSqlxTypesDecimal => write!(f, "sqlx_postgres::types::PgRange<sqlx::types::Decimal>"),
            Self::SqlxPostgresTypesPgRangeSqlxTypesBigDecimal => write!(f, "sqlx_postgres::types::PgRange<sqlx::types::BigDecimal>"),
            Self::SqlxPostgresTypesPgRangeSqlxTypesTimeDate => write!(f, "sqlx_postgres::types::PgRange<sqlx::types::TimeDate>"),
            Self::SqlxPostgresTypesPgRangeSqlxTypesChronoNaiveDate => write!(f, "sqlx_postgres::types::PgRange<sqlx::types::chrono::NaiveDate>"),
            Self::SqlxPostgresTypesPgRangeSqlxTypesChronoDateTimeSqlxTypesChronoFixedOffset => write!(f, "sqlx_postgres::types::PgRange<sqlx::types::chrono::DateTime,sqlx::types::chrono::FixedOffset>"),
            Self::SqlxPostgresTypesPgRangeSqlxTypesChronoDateTimeSqlxTypesChronoLocal => write!(f, "sqlx_postgres::types::PgRange<sqlx::types::chrono::DateTime,sqlx::types::ChronoLocal>"),
            Self::SqlxPostgresTypesPgRangeSqlxTypesChronoDateTimeSqlxTypesChronoUtc => write!(f, "sqlx_postgres::types::PgRange<sqlx::types::chrono::DateTime,sqlx::types::chrono::Utc>"),
            Self::SqlxPostgresTypesPgRangeSqlxTypesChronoNaiveDateTime => write!(f, "sqlx_postgres::types::PgRange<sqlx::types::chrono::NaiveDateTime>"),
            Self::SqlxPostgresTypesPgRangeSqlxTypesTimePrimitiveDateTime => write!(f, "sqlx_postgres::types::PgRange<sqlx::types::time::PrimitiveDateTime>"),
            Self::SqlxPostgresTypesPgRangeSqlxTypesTimeOffsetDateTime => write!(f, "sqlx_postgres::types::PgRange<sqlx::types::time::OffsetDateTime>"),

            // //todo some of types equal few ruest token streams not only one  https://docs.rs/sqlx-postgres/0.7.2/sqlx_postgres/types/index.html https://docs.rs/sqlx/0.7.2/sqlx/trait.Type.html
            // Self::Bool => write!(f, "std::primitive::bool"),
            // Self::StdPrimitiveI8 => write!(f, "std::primitive::i8"),
            // Self::StdPrimitiveI16 => write!(f, "std::primitive::i16"),
            // Self::StdPrimitiveI32 => write!(f, "std::primitive::i32"),
            // Self::StdPrimitiveI64 => write!(f, "std::primitive::i64"),
            // Self::StdPrimitiveF32 => write!(f, "std::primitive::f32"),
            // Self::StdPrimitiveF64 => write!(f, "std::primitive::f64"),
            // Self::BorrowStdPrimitiveStr => write!(f, "&std::primitive::str"), 
            // Self::StdStringString => write!(f, "std::string::String"),
            // Self::BorrowU8Array => write!(f, "&[std::primitive::i8]"), 
            // Self::StdVecVecU8 => write!(f, "std::vec::Vec<std::primitive::i8>"),
            // Self::Unit => write!(f, "()"),
            // Self::SqlxPostgresTypesPgInterval => write!(f, ""),
            // Self::SqlxPostgresTypesPgRangeGeneric => write!(f, ""),
            // Self::SqlxPostgresTypesPgMoney => write!(f, ""),
            // Self::SqlxPostgresTypesPgLTree => write!(f, ""),
            // Self::SqlxPostgresTypesPgLQuery => write!(f, ""),
            // Self::BigdecimalBigDecimal => write!(f, ""),
            // Self::RustDecimalDecimal => write!(f, ""),

            // Self::ChronoDateTimeUtcGeneric => write!(f, ""),
            // Self::ChronoDateTimeLocalGeneric => write!(f, ""),
            // Self::ChronoNaiveDateTime => write!(f, ""),
            // Self::ChronoNaiveDate => write!(f, ""),
            // Self::ChronoNaiveTime => write!(f, ""),
            // Self::ChronoPgTimeTzArray => write!(f, ""),//todo find out from what crate this type


            // Self::TimePrimitiveDateTime => write!(f, ""),
            // Self::TimeOffsetDateTime => write!(f, ""),
            // Self::TimeDate => write!(f, ""),
            // Self::TimeTime => write!(f, ""),
            // Self::TimePgTimeTzArray => write!(f, ""),//todo find out from what crate this type


            // Self::UuidUuid => write!(f, ""),


            // Self::IpnetworkIpNetwork => write!(f, ""),
            // Self::StdNetIpAddr => write!(f, ""),

            // Self::MacAddressMacAddress => write!(f, ""),

            // Self::BitVecBitVec => write!(f, ""),

            // Self::Json => write!(f, ""),//tood full path
            // Self::SerdeJsonValue => write!(f, ""),
            // Self::BorrowSerdeJsonValueRawValue => write!(f, "")
        }
    }
}

impl std::str::FromStr for SupportedFieldType {
    type Err = std::string::String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "std::primitive::i16" => Ok(Self::StdPrimitiveI16),
            "std::primitive::str" => Ok(Self::StdPrimitiveStr),//todo - maybe not allow str?
            "std::primitive::i64" => Ok(Self::StdPrimitiveI64),
            "std::primitive::i32" => Ok(Self::StdPrimitiveI32),
            "std::primitive::f64" => Ok(Self::StdPrimitiveF64),
            "std::primitive::f32" => Ok(Self::StdPrimitiveF32),
            "std::string::String" => Ok(Self::StdStringString),
            "std::primitive::i8" => Ok(Self::StdPrimitiveI8),
            "std::primitive::bool" => Ok(Self::StdPrimitiveBool),
            "std::vec::Vec<std::primitive::u8>" => Ok(Self::StdVecVecStdPrimitiveU8),
            "[std::primitive::u8]" => Ok(Self::StdPrimitiveArrayStdPrimitiveU8),
            "()" => Ok(Self::StdPrimitiveUnit),

            "sqlx::types::Decimal" => Ok(Self::SqlxTypesDecimal),
            "sqlx::types::BigDecimal" => Ok(Self::SqlxTypesBigDecimal),
            "sqlx::types::time::Time" => Ok(Self::SqlxTypesTimeTime),
            "sqlx::types::time::Date" => Ok(Self::SqlxTypesTimeDate),
            "sqlx::types::chrono::NaiveDate" => Ok(Self::SqlxTypesChronoNaiveDate),
            "sqlx::types::chrono::NaiveDateTime" => Ok(Self::SqlxTypesChronoNaiveDateTime),
            "sqlx::types::chrono::NaiveTime" => Ok(Self::SqlxTypesChronoNaiveTime),
            "sqlx::types::time::OffsetDateTime" => Ok(Self::SqlxTypesTimeOffsetDateTime),
            "sqlx::types::time::PrimitiveDateTime" => Ok(Self::SqlxTypesTimePrimitiveDateTime),
            "core::time::Duration" => Ok(Self::CoreTimeDuration),//todo maybe its std::time::Duration or core::time::Duration or both?
            "sqlx::types::chrono::DateTime<sqlx::types::chrono::FixedOffset>" => Ok(Self::SqlxTypesChronoDateTimeSqlxTypesChronoFixedOffset),
            "sqlx::types::chrono::DateTime<sqlx::types::chrono::Local>" => Ok(Self::SqlxTypesChronoDateTimeSqlxTypesChronoLocal),
            "sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>" => Ok(Self::SqlxTypesChronoDateTimeSqlxTypesChronoUtc),
            "sqlx::types::Uuid" => Ok(Self::SqlxTypesUuid),
            "sqlx::types::ipnetwork::IpNetwork" => Ok(Self::SqlxTypesIpnetworkIpNetwork),
            "sqlx::types::mac_address::MacAddress" => Ok(Self::SqlxTypesMacAddressMacAddress),
            "sqlx_postgres::types::PgInterval" => Ok(Self::SqlxPostgresTypesPgInterval),
            "sqlx_postgres::types::PgMoney" => Ok(Self::SqlxPostgresTypesPgMoney),
            "sqlx_postgres::types::PgLQuery" => Ok(Self::SqlxPostgresTypesPgLQuery),
            "sqlx_postgres::types::PgLTree" => Ok(Self::SqlxPostgresTypesPgLTree),
            "sqlx_postgres::types::Oid" => Ok(Self::SqlxPostgresTypesOid),
            "sqlx::types::BitVec<std::primitive::u32>" => Ok(Self::SqlxTypesBitVecStdPrimitiveU32),
            "sqlx_postgres::types::time_tz::PgTimeTz<sqlx::types::time::Time,sqlx::types::time::UtcOffset>" => Ok(Self::SqlxPostgresTypesTimeTzPgTimeTzSqlxTypesTimeTimeSqlxTypesTimeUtcOffset),
            "sqlx_postgres::types::time_tz::PgTimeTz<sqlx::types::chrono::NaiveTime,sqlx::types::chrono::FixedOffset" => Ok(Self::SqlxPostgresTypesTimeTzPgTimeTzSqlxTypesChronoNaiveTimeSqlxTypesChronoFixedOffset),
            "std::boxed::Box<&std::primitive::str>" => Ok(Self::StdBoxedBoxBorrowStdPrimitiveStr),
            "std::borrow::Cow<'_, std::primitive::str>" => Ok(Self::StdBorrowCowAnonymousLifetimeStdPrimitiveStr),

            "sqlx_postgres::types::PgRange<std::primitive::i32>" => Ok(Self::SqlxPostgresTypesPgRangeStdPrimitiveI32),
            "sqlx_postgres::types::PgRange<std::primitive::i64>" => Ok(Self::SqlxPostgresTypesPgRangeStdPrimitiveI64),
            "sqlx_postgres::types::PgRange<sqlx::types::Decimal>" => Ok(Self::SqlxPostgresTypesPgRangeSqlxTypesDecimal),
            "sqlx_postgres::types::PgRange<sqlx::types::BigDecimal>" => Ok(Self::SqlxPostgresTypesPgRangeSqlxTypesBigDecimal),
            "sqlx_postgres::types::PgRange<sqlx::types::TimeDate>" => Ok(Self::SqlxPostgresTypesPgRangeSqlxTypesTimeDate),
            "sqlx_postgres::types::PgRange<sqlx::types::chrono::NaiveDate>" => Ok(Self::SqlxPostgresTypesPgRangeSqlxTypesChronoNaiveDate),
            "sqlx_postgres::types::PgRange<sqlx::types::chrono::DateTime,sqlx::types::chrono::FixedOffset>" => Ok(Self::SqlxPostgresTypesPgRangeSqlxTypesChronoDateTimeSqlxTypesChronoFixedOffset),
            "sqlx_postgres::types::PgRange<sqlx::types::chrono::DateTime,sqlx::types::ChronoLocal>" => Ok(Self::SqlxPostgresTypesPgRangeSqlxTypesChronoDateTimeSqlxTypesChronoLocal),
            "sqlx_postgres::types::PgRange<sqlx::types::chrono::DateTime,sqlx::types::chrono::Utc>" => Ok(Self::SqlxPostgresTypesPgRangeSqlxTypesChronoDateTimeSqlxTypesChronoUtc),
            "sqlx_postgres::types::PgRange<sqlx::types::chrono::NaiveDateTime>" => Ok(Self::SqlxPostgresTypesPgRangeSqlxTypesChronoNaiveDateTime),
            "sqlx_postgres::types::PgRange<sqlx::types::time::PrimitiveDateTime>" => Ok(Self::SqlxPostgresTypesPgRangeSqlxTypesTimePrimitiveDateTime),
            "" => Ok(Self::SqlxPostgresTypesPgRangeSqlxTypesTimeOffsetDateTime),

            // "std::primitive::bool" => Self::Bool,
            // "std::primitive::i8" => Self::StdPrimitiveI8,
            // "std::primitive::i16" => Self::StdPrimitiveI16,
            // "std::primitive::i32" => Self::StdPrimitiveI32,
            // "std::primitive::i64" => Self::StdPrimitiveI64,
            // "std::primitive::f32" => Self::StdPrimitiveF32,
            // "std::primitive::f64" => Self::StdPrimitiveF64,
            // "&std::primitive::str" => Self::BorrowStdPrimitiveStr, 
            // "std::string::String" => Self::StdStringString,
            // "&[std::primitive::i8]" => Self::BorrowU8Array, 
            // "std::vec::Vec<std::primitive::i8>" => Self::StdVecVecU8,
            // "()" => Self::Unit,
            // "sqlx_postgres::types::PgInterval" => Self::SqlxPostgresTypesPgInterval,
            // "sqlx_postgres::types::PgRange<sqlx::types::BigDecimal>" => Self::SqlxPostgresTypesPgRangeGeneric,
            // "sqlx_postgres::types::PgRange<sqlx::types::time::Date>" => Self::SqlxPostgresTypesPgRangeGeneric,
            // "sqlx_postgres::types::PgRange<sqlx::types::chrono::DateTime<Tz>>" => Self::SqlxPostgresTypesPgRangeGeneric,//todo generic parameter
            // "sqlx_postgres::types::PgRange<sqlx::types::Decimal>" => Self::SqlxPostgresTypesPgRangeGeneric,
            // "sqlx_postgres::types::PgRange<sqlx::types::chrono::NaiveDate>" => Self::SqlxPostgresTypesPgRangeGeneric,
            // "sqlx_postgres::types::PgRange<sqlx::types::chrono::NaiveDateTime>" => Self::SqlxPostgresTypesPgRangeGeneric,
            // "sqlx_postgres::types::PgRange<sqlx::types::time::OffsetDateTime>" => Self::SqlxPostgresTypesPgRangeGeneric,
            // "sqlx_postgres::types::PgRange<sqlx::types::time::PrimitiveDateTime>" => Self::SqlxPostgresTypesPgRangeGeneric,
            // "sqlx_postgres::types::PgRange<std::primitive::i32>" => Self::SqlxPostgresTypesPgRangeGeneric,
            // "sqlx_postgres::types::PgRange<std::primitive::i64>" => Self::SqlxPostgresTypesPgRangeGeneric,
            // "sqlx_postgres::types::PgMoney" => Self::SqlxPostgresTypesPgMoney,
            // "sqlx_postgres::types::PgLTree" => Self::SqlxPostgresTypesPgLTree,
            // "sqlx_postgres::types::PgLQuery" => Self::SqlxPostgresTypesPgLQuery,
            // "" => Self::BigdecimalBigDecimal,
            // "" => Self::RustDecimalDecimal,

            // "" => Self::ChronoDateTimeUtcGeneric,
            // "" => Self::ChronoDateTimeLocalGeneric,
            // "" => Self::ChronoNaiveDateTime,
            // "" => Self::ChronoNaiveDate,
            // "" => Self::ChronoNaiveTime,
            // "" => Self::ChronoPgTimeTzArray,//todo find out from what crate this type


            // "" => Self::TimePrimitiveDateTime,
            // "" => Self::TimeOffsetDateTime,
            // "" => Self::TimeDate,
            // "" => Self::TimeTime,
            // "" => Self::TimePgTimeTzArray,//todo find out from what crate this type


            // "" => Self::UuidUuid,


            // "" => Self::IpnetworkIpNetwork,
            // "" => Self::StdNetIpAddr,

            // "" => Self::MacAddressMacAddress,

            // "" => Self::BitVecBitVec,

            // "" => Self::Json,//tood full path
            // "" => Self::SerdeJsonValue,
            // "" => Self::BorrowSerdeJsonValueRawValue,
            _ => Err(format!("")),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct FieldNamedWrapperExcludingPrimaryKey {
    field: syn::Field,
    supported_attribute_type: SupportedAttributeType,
    supported_field_type: SupportedFieldType
}

fn generate_error_variants_vec_token_stream<'a>(
    try_operation_response_variants_camel_case_token_stream: &proc_macro2::TokenStream,
    try_operation_with_serialize_deserialize_camel_case_token_stream: &proc_macro2::TokenStream, //KekwWithSerializeDeserialize
    proc_macro_name_ident_stringified: &std::string::String,
    error_variant_attribute: &std::vec::Vec<&'a crate::type_variants_from_request_response_generator::ErrorVariantAttribute>,
) -> std::vec::Vec<(
    &'a crate::type_variants_from_request_response_generator::ErrorVariantAttribute,
    proc_macro2::TokenStream, //try_operation_token_stream
    proc_macro2::TokenStream, //enum_with_serialize_deserialize_logic_token_stream
    proc_macro2::TokenStream, //from_logic_token_stream
    proc_macro2::TokenStream, //impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream
    proc_macro2::TokenStream, //impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream
    proc_macro2::TokenStream, //enum_status_codes_checker_name_logic_token_stream
    proc_macro2::TokenStream, //axum_response_into_response_logic_token_stream
)> {
    error_variant_attribute.iter().map(|element|crate::type_variants_from_request_response_generator::type_variants_from_request_response(
        &try_operation_response_variants_camel_case_token_stream,
        &try_operation_with_serialize_deserialize_camel_case_token_stream,
        &proc_macro_name_ident_stringified,
        &element
    )).collect()
}

fn construct_syn_variant(
    tvfrr_status_attribute: proc_macro_helpers::attribute::Attribute,
    variant_name: &str,
    code_occurence_field: &syn::Field,
    fields: std::vec::Vec<(proc_macro_helpers::error_occurence::named_attribute::NamedAttribute, &str, syn::punctuated::Punctuated::<syn::PathSegment, syn::token::Colon2>)>
) -> syn::Variant {
    syn::Variant {
        attrs: vec![
            syn::Attribute {
                pound_token: syn::token::Pound {
                    spans: [proc_macro2::Span::call_site()],
                },
                style: syn::AttrStyle::Outer,
                bracket_token: syn::token::Bracket {
                    span: proc_macro2::Span::call_site(),
                },
                path: syn::Path {
                    leading_colon: None,
                    segments: {
                        let mut handle = syn::punctuated::Punctuated::new();
                        handle.push(syn::PathSegment {
                            ident: proc_macro2::Ident::new(&tvfrr_status_attribute.to_string(), proc_macro2::Span::call_site()),
                            arguments: syn::PathArguments::None,
                        });
                       handle
                    },
                },
                tokens: proc_macro2::TokenStream::new(),
            },
        ],
        ident: syn::Ident::new(variant_name, proc_macro2::Span::call_site()),
        fields: syn::Fields::Named(
            syn::FieldsNamed {
                brace_token: syn::token::Brace {
                    span: proc_macro2::Span::call_site(),
                },
                named: {
                    let mut handle = fields.into_iter().fold(syn::punctuated::Punctuated::new(), |mut acc, element| {
                        acc.push_value(
                            syn::Field {
                                attrs: vec![
                                    syn::Attribute {
                                        pound_token: syn::token::Pound {
                                            spans: [proc_macro2::Span::call_site()],
                                        },
                                        style: syn::AttrStyle::Outer,
                                        bracket_token: syn::token::Bracket {
                                            span: proc_macro2::Span::call_site(),
                                        },
                                        path: syn::Path {
                                            leading_colon: None,
                                            segments: {
                                                let mut handle = syn::punctuated::Punctuated::new();
                                                handle.push(
                                                    syn::PathSegment {
                                                        ident: proc_macro2::Ident::new(&element.0.to_string(), proc_macro2::Span::call_site()),
                                                        arguments: syn::PathArguments::None,
                                                    }
                                                );
                                                handle
                                            },
                                        },
                                        tokens: proc_macro2::TokenStream::new(),
                                    },
                                ],
                                vis: syn::Visibility::Inherited,
                                ident: Some(
                                    syn::Ident::new(&element.1, proc_macro2::Span::call_site())
                                ),
                                colon_token: Some(
                                    syn::token::Colon {
                                        spans: [proc_macro2::Span::call_site()],
                                    },
                                ),
                                ty: syn::Type::Path(
                                    syn::TypePath {
                                        qself: None,
                                        path: syn::Path {
                                            leading_colon: None,
                                            segments: element.2
                                        },
                                    },
                                ),
                            }
                        );
                        acc.push_punct(
                            syn::token::Comma {
                                spans: [proc_macro2::Span::call_site()],
                            }
                        );
                        acc
                    });
                    handle.push_value(code_occurence_field.clone());
                    handle
                },
            },
        ),
        discriminant: None,
    }
}