pub fn type_variants_from_request_response_generator(
    desirable_attribute: proc_macro_helpers::attribute::Attribute,
    desirable_token_stream: &proc_macro2::TokenStream,
    desirable_type_token_stream: &proc_macro2::TokenStream, //std::vec::Vec<crate::server::postgres::uuid_wrapper::PossibleUuidWrapper>
    try_operation_camel_case_token_stream: &proc_macro2::TokenStream,
    try_operation_response_variants_camel_case_stringified: &std::string::String,
    try_operation_response_variants_camel_case_token_stream: &proc_macro2::TokenStream, //KekwResponseVariants
    try_operation_response_variants_desirable_attribute_token_stream: &proc_macro2::TokenStream,
    operation_with_serialize_deserialize_camel_case_token_stream: &proc_macro2::TokenStream,
    try_operation_request_error_token_stream: &proc_macro2::TokenStream,
    try_operation_with_serialize_deserialize_token_stream: &proc_macro2::TokenStream,
    operation_lower_case_stringified: &std::string::String,
    code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream: &proc_macro2::TokenStream,
    code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream: &proc_macro2::TokenStream,
    error_named_derive_token_stream: &proc_macro2::TokenStream,
    eo_display_token_stream: &proc_macro2::TokenStream,
    eo_display_foreign_type_token_stream: &proc_macro2::TokenStream,
    eo_display_with_serialize_deserialize_token_stream: &proc_macro2::TokenStream,
    derive_debug_serialize_deserialize_token_stream: &proc_macro2::TokenStream,
    type_variants_from_request_response_syn_variants: std::vec::Vec<&syn::Variant>,
    // ident_response_variants_token_stream: &proc_macro2::TokenStream,
    is_response_with_body: bool,
    proc_macro_name_ident_stringified: &std::string::String,
) -> proc_macro2::TokenStream {
    #[derive(Debug, Clone)]
    struct ErrorVariantAttribute<'a> {
        pub error_variant_attribute: proc_macro_helpers::attribute::Attribute,
        pub error_variant: ErrorVariant<'a>,
    }
    #[derive(Debug, Clone)]
    struct ErrorVariant<'a> {
        pub error_variant_ident: &'a syn::Ident,
        pub error_variant_fields: std::vec::Vec<ErrorVariantField>,
    }
    #[derive(Debug, Clone)]
    struct ErrorVariantField {
        pub field_name: syn::Ident,
        pub error_occurence_attribute: proc_macro2::TokenStream,
        pub field_type_original: proc_macro2::TokenStream,
        pub field_type_with_serialize_deserialize: proc_macro2::TokenStream,
    }
    let code_occurence_camel_case = format!("Code{}", proc_macro_helpers::error_occurence::hardcode::OCCURENCE_CAMEL_CASE);
    let code_occurence_lower_case = proc_macro_helpers::to_lower_snake_case::ToLowerSnakeCase::to_lower_snake_case(&code_occurence_camel_case).to_lowercase();
    let http_status_code_quote_token_stream = desirable_attribute.to_http_status_code_quote();
    let vec_status_codes_len = type_variants_from_request_response_syn_variants.len();
    let crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream = quote::quote! {crate::common::api_request_unexpected_error::ApiRequestUnexpectedError};
    let crate_common_api_request_unexpected_error_response_text_result_token_stream = quote::quote! {crate::common::api_request_unexpected_error::ResponseTextResult};
    let try_operation_token_stream = {
        let try_operation_mapped_token_stream = type_variants_from_request_response_syn_variants.iter().map(|error_variant_attribute| {
            let variant_ident = &error_variant_attribute.ident;
            let fields_named = if let syn::Fields::Named(fields_named) = &error_variant_attribute.fields {
                fields_named
            }
            else {
                panic!("{proc_macro_name_ident_stringified} expected fields would be named");
            };
            let fields_mapped_into_token_stream = fields_named.named.iter().map(|field|{
                let field_ident = field.ident.clone().unwrap_or_else(|| panic!(
                    "{proc_macro_name_ident_stringified} field.ident {}",
                    proc_macro_helpers::error_occurence::hardcode::IS_NONE_STRINGIFIED
                ));
                let error_occurence_attribute = match field_ident == code_occurence_lower_case {
                    true => quote::quote! {},
                    false => {
                        let mut error_occurence_attribute: Option<proc_macro_helpers::error_occurence::named_attribute::NamedAttribute> = None;
                        for element in &field.attrs {
                            if let true = element.path.segments.len() == 1 {
                                let segment = element.path.segments.first().unwrap_or_else(|| {panic!("{proc_macro_name_ident_stringified} element.path.segments.get(0) is None")});
                                if let Ok(value) = {
                                    use std::str::FromStr;
                                    proc_macro_helpers::error_occurence::named_attribute::NamedAttribute::from_str(&segment.ident.to_string())
                                } {
                                    match error_occurence_attribute {
                                        Some(value) => panic!("{proc_macro_name_ident_stringified} duplicated attributes ({}) are not supported", value.to_string()),
                                        None => {
                                            error_occurence_attribute = Some(value);
                                        }
                                    }
                                }
                            }
                        }
                        match error_occurence_attribute {
                            Some(value) => value.to_attribute_view_token_stream(),
                            None => panic!("{proc_macro_name_ident_stringified} {variant_ident} no supported attribute"),
                        }
                    }
                };
                let field_type = &field.ty;
                quote::quote! {
                    #error_occurence_attribute
                    #field_ident: #field_type
                }
            }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
            quote::quote! {
                #variant_ident {
                    #(#fields_mapped_into_token_stream),*
                }
            }
        }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            #[derive(
                Debug,
                thiserror::Error,
                error_occurence::ErrorOccurence,
                from_sqlx_postgres_error::FromSqlxPostgresError,
            )]
            pub enum #try_operation_camel_case_token_stream {
                #(#try_operation_mapped_token_stream),*
            }
        }
    };
    let enum_with_serialize_deserialize_logic_token_stream_handle_token_stream = {
        let enum_with_serialize_deserialize_logic_mapped_token_stream = type_variants_from_request_response_syn_variants.iter().map(|error_variant_attribute| {
            let variant_ident = &error_variant_attribute.ident;
            let fields_named = if let syn::Fields::Named(fields_named) = &error_variant_attribute.fields {
                fields_named
            }
            else {
                panic!("{proc_macro_name_ident_stringified} expected fields would be named");
            };
            let fields_mapped_into_token_stream = fields_named.named.iter().map(|field|{
                let field_ident = field.ident.clone().unwrap_or_else(|| panic!(
                    "{proc_macro_name_ident_stringified} field.ident {}",
                    proc_macro_helpers::error_occurence::hardcode::IS_NONE_STRINGIFIED
                ));
                let field_type_with_serialize_deserialize = match field_ident == code_occurence_lower_case {
                    true => {
                        let code_occurence_type_token_stream = {
                            if let syn::Type::Path(type_path) = &field.ty {
                                let mut code_occurence_type_repeat_checker = false;
                                let code_occurence_segments_stringified_handle = type_path.path.segments.iter()
                                .fold(String::from(""), |mut acc, path_segment| {
                                    let path_segment_ident = &path_segment.ident;
                                    match *path_segment_ident == code_occurence_camel_case {
                                        true => {
                                            if code_occurence_type_repeat_checker {
                                                panic!("{proc_macro_name_ident_stringified} code_occurence_ident detected more than one {code_occurence_camel_case} inside type path");
                                            }
                                            acc.push_str(&path_segment_ident.to_string());
                                            code_occurence_type_repeat_checker = true;
                                        },
                                        false => acc.push_str(&format!("{path_segment_ident}::")),
                                    }
                                    acc
                                });
                                if !code_occurence_type_repeat_checker {
                                    panic!("{proc_macro_name_ident_stringified} no {code_occurence_camel_case} named field");
                                }
                                code_occurence_segments_stringified_handle.parse::<proc_macro2::TokenStream>()
                                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {code_occurence_segments_stringified_handle} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                            }
                            else {
                                let syn_type_path_stringified = proc_macro_helpers::error_occurence::hardcode::syn_type_path_stringified();
                                panic!(
                                    "{proc_macro_name_ident_stringified} {code_occurence_lower_case} {} {syn_type_path_stringified}",
                                    proc_macro_helpers::error_occurence::hardcode::SUPPORTS_ONLY_STRINGIFIED
                                );
                            }
                        };
                        code_occurence_type_token_stream
                    },
                    false => {
                        let attribute = {
                            let mut option_attribute = None;
                            field.attrs.iter().for_each(|attr|{
                                if let true = attr.path.segments.len() == 1 {
                                    let error_message = format!("{proc_macro_name_ident_stringified} two or more supported attributes!");
                                    let attr_ident = match attr.path.segments.iter().next() {
                                        Some(path_segment) => &path_segment.ident,
                                        None => panic!("attr.path.segments.iter().next() is None"),
                                    };
                                    match {
                                        use std::str::FromStr;
                                        proc_macro_helpers::error_occurence::named_attribute::NamedAttribute::from_str(&attr_ident.to_string())
                                    } {
                                        Ok(value) => {
                                            if let true = option_attribute.is_some() {
                                                panic!("{error_message}");
                                            }
                                            else {
                                                option_attribute = Some(value);
                                            }
                                        },
                                        Err(_) => ()//other attributes are not for this proc_macro
                                    }
                                }//other attributes are not for this proc_macro
                            });
                            option_attribute.unwrap_or_else(|| panic!(
                                "{proc_macro_name_ident_stringified} option attribute {}",
                                proc_macro_helpers::error_occurence::hardcode::IS_NONE_STRINGIFIED
                            ))
                        };
                        let supported_container = proc_macro_helpers::error_occurence::generate_with_serialize_deserialize_version::generate_supported_container(
                            &field,
                            &proc_macro_name_ident_stringified,
                        );
                        let field_type_with_serialize_deserialize = proc_macro_helpers::error_occurence::generate_with_serialize_deserialize_version::generate_field_type_with_serialize_deserialize_version(
                            attribute,
                            supported_container,
                            &proc_macro_name_ident_stringified,
                        );
                        field_type_with_serialize_deserialize
                    },
                };
                quote::quote! {#field_ident: #field_type_with_serialize_deserialize}
            }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
            quote::quote! {
                #variant_ident {
                    #(#fields_mapped_into_token_stream),*
                }
            }
        }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();      
        quote::quote! {
            #derive_debug_serialize_deserialize_token_stream
            pub enum #try_operation_response_variants_camel_case_token_stream {
                #desirable_token_stream(#desirable_type_token_stream),
                #(#enum_with_serialize_deserialize_logic_mapped_token_stream),*
            }
        }
    };
    let from_logic_token_stream_handle_token_stream = {
        let from_logic_token_stream_mapped_token_stream = type_variants_from_request_response_syn_variants.iter().map(|error_variant_attribute| {
            let variant_ident = &error_variant_attribute.ident;
            let fields_named = if let syn::Fields::Named(fields_named) = &error_variant_attribute.fields {
                fields_named
            }
            else {
                panic!("{proc_macro_name_ident_stringified} expected fields would be named");
            };
            let fields_name_mapped_into_token_stream = fields_named.named.iter().map(|field|{
                let field_ident = field.ident.clone().unwrap_or_else(|| panic!(
                    "{proc_macro_name_ident_stringified} field.ident {}",
                    proc_macro_helpers::error_occurence::hardcode::IS_NONE_STRINGIFIED
                ));
                quote::quote! {#field_ident}
            }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
            quote::quote! {
                #operation_with_serialize_deserialize_camel_case_token_stream::#variant_ident {
                    #(#fields_name_mapped_into_token_stream),*
                } => Self::#variant_ident {
                    #(#fields_name_mapped_into_token_stream),*
                }
            }
        }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            impl std::convert::From<#try_operation_camel_case_token_stream> for #try_operation_response_variants_camel_case_token_stream {
                fn from(value: #try_operation_camel_case_token_stream) -> Self {
                    match value.into_serialize_deserialize_version() {
                        #(#from_logic_token_stream_mapped_token_stream),*
                    }
                }
            }
        }
    };
    let impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream_handle_token_stream = {
        let impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream_handle_mapped_token_stream = type_variants_from_request_response_syn_variants
            .iter()
            .map(|error_variant_attribute| {
                let variant_ident = &error_variant_attribute.ident;
                let fields_named = if let syn::Fields::Named(fields_named) = &error_variant_attribute.fields {
                    fields_named
                }
                else {
                    panic!("{proc_macro_name_ident_stringified} expected fields would be named");
                };
                let fields_anonymous_types_mapped_into_token_stream = fields_named.named.iter().map(|field|{
                    let field_ident = field.ident.clone().unwrap_or_else(|| panic!(
                        "{proc_macro_name_ident_stringified} field.ident {}",
                        proc_macro_helpers::error_occurence::hardcode::IS_NONE_STRINGIFIED
                    ));
                    quote::quote! {#field_ident: _}
                }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
                quote::quote! {
                    #try_operation_response_variants_camel_case_token_stream::#variant_ident {
                        #(#fields_anonymous_types_mapped_into_token_stream),*
                    } => #http_status_code_quote_token_stream
                }
            }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            impl std::convert::From<&#try_operation_response_variants_camel_case_token_stream> for http::StatusCode {
                fn from(value: &#try_operation_response_variants_camel_case_token_stream) -> Self {
                    match value {
                        #try_operation_response_variants_camel_case_token_stream::#desirable_token_stream(_) => #http_status_code_quote_token_stream,
                        #(#impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream_handle_mapped_token_stream),*
                    }
                }
            }
        }
    };
    let generated_status_code_enums_with_from_impls_logic_token_stream_handle_token_stream = {
        let generated_status_code_enums_with_from_impls_logic_token_stream = {
            let status_code_enums_with_from_impls_logic_token_stream = type_variants_from_request_response_syn_variants.iter().fold(
                std::collections::HashMap::<proc_macro_helpers::attribute::Attribute, std::vec::Vec<ErrorVariant>>::with_capacity(vec_status_codes_len),
                |mut acc, element| {
                    let variant_ident = &element.ident;
                    let error_variant_attribute = proc_macro_helpers::attribute::Attribute::try_from(element)
                    .unwrap_or_else(|e| {panic!("{proc_macro_name_ident_stringified} variant {variant_ident} failed: {e}")});
                    let fields_named = if let syn::Fields::Named(fields_named) = &element.fields {
                        fields_named
                    }
                    else {
                        panic!("{proc_macro_name_ident_stringified} expected fields would be named");
                    };
                    let error_variant_fields = fields_named.named.iter().map(|field|{
                        let field_ident = field.ident.clone().unwrap_or_else(|| panic!(
                            "{proc_macro_name_ident_stringified} field.ident {}",
                            proc_macro_helpers::error_occurence::hardcode::IS_NONE_STRINGIFIED
                        ));
                        let error_occurence_attribute = match field_ident == code_occurence_lower_case {
                            true => quote::quote! {},
                            false => {
                                let mut error_occurence_attribute: Option<proc_macro_helpers::error_occurence::named_attribute::NamedAttribute> = None;
                                for element in &field.attrs {
                                    if let true = element.path.segments.len() == 1 {
                                        let segment = element.path.segments.first().unwrap_or_else(|| {panic!("{proc_macro_name_ident_stringified} element.path.segments.get(0) is None")});
                                        if let Ok(value) = {
                                            use std::str::FromStr;
                                            proc_macro_helpers::error_occurence::named_attribute::NamedAttribute::from_str(&segment.ident.to_string())
                                        } {
                                            match error_occurence_attribute {
                                                Some(value) => panic!("{proc_macro_name_ident_stringified} duplicated attributes ({}) are not supported", value.to_string()),
                                                None => {
                                                    error_occurence_attribute = Some(value);
                                                }
                                            }
                                        }
                                    }
                                }
                                match error_occurence_attribute {
                                    Some(value) => value.to_attribute_view_token_stream(),
                                    None => panic!("{proc_macro_name_ident_stringified} {variant_ident} no supported attribute"),
                                }
                            }
                        };
                        let field_type_original = &field.ty;
                        let field_type_with_serialize_deserialize = match field_ident == code_occurence_lower_case {
                            true => {
                                let code_occurence_type_token_stream = {
                                    if let syn::Type::Path(type_path) = &field.ty {
                                        let mut code_occurence_type_repeat_checker = false;
                                        let code_occurence_segments_stringified_handle = type_path.path.segments.iter()
                                        .fold(String::from(""), |mut acc, path_segment| {
                                            let path_segment_ident = &path_segment.ident;
                                            match *path_segment_ident == code_occurence_camel_case {
                                                true => {
                                                    if code_occurence_type_repeat_checker {
                                                        panic!("{proc_macro_name_ident_stringified} code_occurence_ident detected more than one {code_occurence_camel_case} inside type path");
                                                    }
                                                    acc.push_str(&path_segment_ident.to_string());
                                                    code_occurence_type_repeat_checker = true;
                                                },
                                                false => acc.push_str(&format!("{path_segment_ident}::")),
                                            }
                                            acc
                                        });
                                        if !code_occurence_type_repeat_checker {
                                            panic!("{proc_macro_name_ident_stringified} no {code_occurence_camel_case} named field");
                                        }
                                        code_occurence_segments_stringified_handle.parse::<proc_macro2::TokenStream>()
                                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {code_occurence_segments_stringified_handle} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                    }
                                    else {
                                        let syn_type_path_stringified = proc_macro_helpers::error_occurence::hardcode::syn_type_path_stringified();
                                        panic!(
                                            "{proc_macro_name_ident_stringified} {code_occurence_lower_case} {} {syn_type_path_stringified}",
                                            proc_macro_helpers::error_occurence::hardcode::SUPPORTS_ONLY_STRINGIFIED
                                        );
                                    }
                                };
                                code_occurence_type_token_stream
                            },
                            false => {
                                let attribute = {
                                    let mut option_attribute = None;
                                    field.attrs.iter().for_each(|attr|{
                                        if let true = attr.path.segments.len() == 1 {
                                            let error_message = format!("{proc_macro_name_ident_stringified} two or more supported attributes!");
                                            let attr_ident = match attr.path.segments.iter().next() {
                                                Some(path_segment) => &path_segment.ident,
                                                None => panic!("attr.path.segments.iter().next() is None"),
                                            };
                                            match {
                                                use std::str::FromStr;
                                                proc_macro_helpers::error_occurence::named_attribute::NamedAttribute::from_str(&attr_ident.to_string())
                                            } {
                                                Ok(value) => {
                                                    if let true = option_attribute.is_some() {
                                                        panic!("{error_message}");
                                                    }
                                                    else {
                                                        option_attribute = Some(value);
                                                    }
                                                },
                                                Err(_) => ()//other attributes are not for this proc_macro
                                            }
                                        }//other attributes are not for this proc_macro
                                    });
                                    option_attribute.unwrap_or_else(|| panic!(
                                        "{proc_macro_name_ident_stringified} option attribute {}",
                                        proc_macro_helpers::error_occurence::hardcode::IS_NONE_STRINGIFIED
                                    ))
                                };
                                let supported_container = proc_macro_helpers::error_occurence::generate_with_serialize_deserialize_version::generate_supported_container(
                                    &field,
                                    &proc_macro_name_ident_stringified,
                                );
                                let field_type_with_serialize_deserialize = proc_macro_helpers::error_occurence::generate_with_serialize_deserialize_version::generate_field_type_with_serialize_deserialize_version(
                                    attribute,
                                    supported_container,
                                    &proc_macro_name_ident_stringified,
                                );
                                field_type_with_serialize_deserialize
                            },
                        };
                        ErrorVariantField {
                            field_name: field_ident.clone(),
                            error_occurence_attribute,
                            field_type_original: quote::quote! {#field_type_original},
                            field_type_with_serialize_deserialize,
                        }
                    }).collect::<Vec<ErrorVariantField>>();
                    let error_variant = ErrorVariant {
                        error_variant_ident: &variant_ident,
                        error_variant_fields,
                    };
                    match acc.get_mut(&error_variant_attribute) {
                        Some(value) => {
                            value.push(error_variant);
                        },
                        None => {
                            acc.insert(error_variant_attribute, vec![error_variant]);
                        }
                    }
                    acc
                },
            )
            .into_iter().map(|(key,value)|{
                let try_operation_response_variants_attribute_token_stream = {
                    let try_operation_response_variants_attribute_stingified = format!("{try_operation_response_variants_camel_case_stringified}{key}");
                    try_operation_response_variants_attribute_stingified
                    .parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_response_variants_attribute_stingified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                let enum_variants_token_stream = value.iter().map(|element|{
                    let error_variant_ident = &element.error_variant_ident;
                    let fields_mapped_into_token_stream = element.error_variant_fields.iter().map(|element| {
                        let field_name_token_stream = &element.field_name;
                        let field_type_token_stream = &element.field_type_with_serialize_deserialize;
                        quote::quote! {#field_name_token_stream: #field_type_token_stream}
                    }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
                    quote::quote!{
                        #error_variant_ident {
                            #(#fields_mapped_into_token_stream),*
                        }
                    }
                });
                let std_convert_from_match_variants_token_stream = value.iter().map(|element|{
                    let error_variant_ident = &element.error_variant_ident;
                    let fields_name_mapped_into_token_stream = element.error_variant_fields.iter().map(|element| {
                        let field_name_token_stream = &element.field_name;
                        quote::quote! {#field_name_token_stream}
                    }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
                    quote::quote!{
                        #try_operation_response_variants_attribute_token_stream::#error_variant_ident {
                            #(#fields_name_mapped_into_token_stream),*
                        } => Self::#error_variant_ident {
                            #(#fields_name_mapped_into_token_stream),*
                        }
                    }
                });
                quote::quote!{
                    #derive_debug_serialize_deserialize_token_stream
                    enum #try_operation_response_variants_attribute_token_stream {
                        #(#enum_variants_token_stream),*
                    }
                    impl std::convert::From<#try_operation_response_variants_attribute_token_stream> for #try_operation_response_variants_camel_case_token_stream {
                        fn from(value: #try_operation_response_variants_attribute_token_stream) -> Self {
                            match value {
                                #(#std_convert_from_match_variants_token_stream),*
                            }
                        }
                    }
                }
            });
            quote::quote! {
                #(#status_code_enums_with_from_impls_logic_token_stream)*
            }
        };
        quote::quote! {
            #derive_debug_serialize_deserialize_token_stream
            enum #try_operation_response_variants_desirable_attribute_token_stream {
                #desirable_token_stream(#desirable_type_token_stream),
            }
            impl std::convert::From<#try_operation_response_variants_desirable_attribute_token_stream> for #try_operation_response_variants_camel_case_token_stream {
                fn from(value: #try_operation_response_variants_desirable_attribute_token_stream) -> Self {
                    match value {
                        #try_operation_response_variants_desirable_attribute_token_stream::#desirable_token_stream(i) => Self::#desirable_token_stream(i),
                    }
                }
            }
            #generated_status_code_enums_with_from_impls_logic_token_stream
        }
    };
    let try_from_response_logic_token_stream_handle_token_stream = {
        let (
            unique_status_codes,
            unique_status_codes_len,
            unique_status_codes_len_minus_one
         ) = {
            let hashmap_unique_status_codes = type_variants_from_request_response_syn_variants.iter().fold(//todo maybe not need hashmap here? maybe just unique vec?
                std::collections::HashMap::<proc_macro_helpers::attribute::Attribute, std::vec::Vec<ErrorVariant>>::with_capacity(vec_status_codes_len),
                |mut acc, element| {
                    let variant_ident = &element.ident;
                    let error_variant_attribute = proc_macro_helpers::attribute::Attribute::try_from(element)
                    .unwrap_or_else(|e| {panic!("{proc_macro_name_ident_stringified} variant {variant_ident} failed: {e}")});
                    let fields_named = if let syn::Fields::Named(fields_named) = &element.fields {
                        fields_named
                    }
                    else {
                        panic!("{proc_macro_name_ident_stringified} expected fields would be named");
                    };
                    let error_variant_fields = fields_named.named.iter().map(|field|{
                        let field_ident = field.ident.clone().unwrap_or_else(|| panic!(
                            "{proc_macro_name_ident_stringified} field.ident {}",
                            proc_macro_helpers::error_occurence::hardcode::IS_NONE_STRINGIFIED
                        ));
                        let error_occurence_attribute = match field_ident == code_occurence_lower_case {
                            true => quote::quote! {},
                            false => {
                                let mut error_occurence_attribute: Option<proc_macro_helpers::error_occurence::named_attribute::NamedAttribute> = None;
                                for element in &field.attrs {
                                    if let true = element.path.segments.len() == 1 {
                                        let segment = element.path.segments.first().unwrap_or_else(|| {panic!("{proc_macro_name_ident_stringified} element.path.segments.get(0) is None")});
                                        if let Ok(value) = {
                                            use std::str::FromStr;
                                            proc_macro_helpers::error_occurence::named_attribute::NamedAttribute::from_str(&segment.ident.to_string())
                                        } {
                                            match error_occurence_attribute {
                                                Some(value) => panic!("{proc_macro_name_ident_stringified} duplicated attributes ({}) are not supported", value.to_string()),
                                                None => {
                                                    error_occurence_attribute = Some(value);
                                                }
                                            }
                                        }
                                    }
                                }
                                match error_occurence_attribute {
                                    Some(value) => value.to_attribute_view_token_stream(),
                                    None => panic!("{proc_macro_name_ident_stringified} {variant_ident} no supported attribute"),
                                }
                            }
                        };
                        let field_type_original = &field.ty;
                        let field_type_with_serialize_deserialize = match field_ident == code_occurence_lower_case {
                            true => {
                                let code_occurence_type_token_stream = {
                                    if let syn::Type::Path(type_path) = &field.ty {
                                        let mut code_occurence_type_repeat_checker = false;
                                        let code_occurence_segments_stringified_handle = type_path.path.segments.iter()
                                        .fold(String::from(""), |mut acc, path_segment| {
                                            let path_segment_ident = &path_segment.ident;
                                            match *path_segment_ident == code_occurence_camel_case {
                                                true => {
                                                    if code_occurence_type_repeat_checker {
                                                        panic!("{proc_macro_name_ident_stringified} code_occurence_ident detected more than one {code_occurence_camel_case} inside type path");
                                                    }
                                                    acc.push_str(&path_segment_ident.to_string());
                                                    code_occurence_type_repeat_checker = true;
                                                },
                                                false => acc.push_str(&format!("{path_segment_ident}::")),
                                            }
                                            acc
                                        });
                                        if !code_occurence_type_repeat_checker {
                                            panic!("{proc_macro_name_ident_stringified} no {code_occurence_camel_case} named field");
                                        }
                                        code_occurence_segments_stringified_handle.parse::<proc_macro2::TokenStream>()
                                        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {code_occurence_segments_stringified_handle} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                                    }
                                    else {
                                        let syn_type_path_stringified = proc_macro_helpers::error_occurence::hardcode::syn_type_path_stringified();
                                        panic!(
                                            "{proc_macro_name_ident_stringified} {code_occurence_lower_case} {} {syn_type_path_stringified}",
                                            proc_macro_helpers::error_occurence::hardcode::SUPPORTS_ONLY_STRINGIFIED
                                        );
                                    }
                                };
                                code_occurence_type_token_stream
                            },
                            false => {
                                let attribute = {
                                    let mut option_attribute = None;
                                    field.attrs.iter().for_each(|attr|{
                                        if let true = attr.path.segments.len() == 1 {
                                            let error_message = format!("{proc_macro_name_ident_stringified} two or more supported attributes!");
                                            let attr_ident = match attr.path.segments.iter().next() {
                                                Some(path_segment) => &path_segment.ident,
                                                None => panic!("attr.path.segments.iter().next() is None"),
                                            };
                                            match {
                                                use std::str::FromStr;
                                                proc_macro_helpers::error_occurence::named_attribute::NamedAttribute::from_str(&attr_ident.to_string())
                                            } {
                                                Ok(value) => {
                                                    if let true = option_attribute.is_some() {
                                                        panic!("{error_message}");
                                                    }
                                                    else {
                                                        option_attribute = Some(value);
                                                    }
                                                },
                                                Err(_) => ()//other attributes are not for this proc_macro
                                            }
                                        }//other attributes are not for this proc_macro
                                    });
                                    option_attribute.unwrap_or_else(|| panic!(
                                        "{proc_macro_name_ident_stringified} option attribute {}",
                                        proc_macro_helpers::error_occurence::hardcode::IS_NONE_STRINGIFIED
                                    ))
                                };
                                let supported_container = proc_macro_helpers::error_occurence::generate_with_serialize_deserialize_version::generate_supported_container(
                                    &field,
                                    &proc_macro_name_ident_stringified,
                                );
                                let field_type_with_serialize_deserialize = proc_macro_helpers::error_occurence::generate_with_serialize_deserialize_version::generate_field_type_with_serialize_deserialize_version(
                                    attribute,
                                    supported_container,
                                    &proc_macro_name_ident_stringified,
                                );
                                field_type_with_serialize_deserialize
                            },
                        };
                        ErrorVariantField {
                            field_name: field_ident,
                            error_occurence_attribute,
                            field_type_original: quote::quote! {#field_type_original},
                            field_type_with_serialize_deserialize,
                        }
                    }).collect::<Vec<ErrorVariantField>>();
                    let error_variant = ErrorVariant {
                        error_variant_ident: &variant_ident,
                        error_variant_fields,
                    };
                    match acc.get_mut(&error_variant_attribute) {
                        Some(value) => {
                            value.push(error_variant);
                        },
                        None => {
                            acc.insert(error_variant_attribute, vec![error_variant]);
                        }
                    }
                    acc
                },
            );
            let unique_status_codes_len = hashmap_unique_status_codes.len();
            if unique_status_codes_len < 1 {
                panic!("{proc_macro_name_ident_stringified} unique_status_codes_len < 1 {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE);
            }
            let unique_status_codes_len_minus_one = unique_status_codes_len - 1;
            let unique_status_codes = hashmap_unique_status_codes.into_iter().map(|(key, _)|key).collect::<std::vec::Vec<proc_macro_helpers::attribute::Attribute>>();
            (
                unique_status_codes,
                unique_status_codes_len,
                unique_status_codes_len_minus_one
            )
        };
        let desirable_enum_name = {
            let status_code_enum_name_stingified = format!("{try_operation_response_variants_camel_case_token_stream}{desirable_attribute}");
            status_code_enum_name_stingified
            .parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {status_code_enum_name_stingified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let api_request_unexpected_error_module_path_token_stream = quote::quote! { crate::common::api_request_unexpected_error };
        let api_request_unexpected_error_path_token_stream = quote::quote! { #api_request_unexpected_error_module_path_token_stream::ApiRequestUnexpectedError };
        let try_from_response_operation_lower_case_token_stream = {
            let ident_response_variants_attribute_stingified = format!("try_from_response_try_{operation_lower_case_stringified}");
            ident_response_variants_attribute_stingified
            .parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ident_response_variants_attribute_stingified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let status_code_enums_try_from = {
            let mut is_last_element_found = false;
            let desirable_status_code_case_token_stream = match is_response_with_body {
                true => quote::quote! {
                    match response.text().await {
                        Ok(response_text) => match serde_json::from_str::<#desirable_enum_name>(&response_text){
                            Ok(value) => Ok(#try_operation_response_variants_camel_case_token_stream::from(value)), 
                            Err(e) => Err(
                                #api_request_unexpected_error_path_token_stream::DeserializeBody{ 
                                    serde: e,
                                    status_code,
                                    headers,response_text
                                }
                            ),
                        },
                        Err(e) => Err(
                            #api_request_unexpected_error_path_token_stream::FailedToGetResponseText {
                                reqwest: e,
                                status_code,
                                headers,
                            }
                        ),
                    }
                },
                false => quote::quote! {//#ident_response_variants_token_stream
                    Ok(#try_operation_response_variants_camel_case_token_stream::#desirable_token_stream(()))
                },
            };
            let mut status_code_enums_try_from_variants = Vec::with_capacity(unique_status_codes_len + 1);
            status_code_enums_try_from_variants.push(quote::quote! {
                if status_code == #http_status_code_quote_token_stream {
                    #desirable_status_code_case_token_stream
                }
            });
            unique_status_codes
            .into_iter()
            .enumerate()
            .for_each(|(index, status_code_attribute)|{
                let status_code_enum_name_stringified = format!("{try_operation_response_variants_camel_case_token_stream}{status_code_attribute}");
                let status_code_enum_name_token_stream = status_code_enum_name_stringified
                    .parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {status_code_enum_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
                let http_status_code_token_stream = status_code_attribute.to_http_status_code_quote();
                match index == unique_status_codes_len_minus_one{
                    true => {
                        is_last_element_found = true;
                        status_code_enums_try_from_variants.push(quote::quote! {
                            else {
                                match response.text().await {
                                    Ok(response_text) => Err(
                                        #api_request_unexpected_error_path_token_stream::StatusCode {
                                            status_code,
                                            headers,
                                            response_text_result: #api_request_unexpected_error_module_path_token_stream::ResponseTextResult::ResponseText(response_text)
                                        },
                                    ),
                                    Err(e) => Err(
                                        #api_request_unexpected_error_path_token_stream::StatusCode {
                                            status_code,
                                            headers,
                                            response_text_result: #api_request_unexpected_error_module_path_token_stream::ResponseTextResult::ReqwestError(e),
                                        },
                                    ),
                                }
                            }
                        });
                    },
                    false => {
                        if let false = desirable_attribute == status_code_attribute {
                            status_code_enums_try_from_variants.push(quote::quote! {
                                else if status_code == #http_status_code_token_stream {
                                    match response.text().await {
                                        Ok(response_text) => match serde_json::from_str::<#status_code_enum_name_token_stream>(&response_text){
                                            Ok(value) => Ok(#try_operation_response_variants_camel_case_token_stream::from(value)), 
                                            Err(e) => Err(
                                                #api_request_unexpected_error_path_token_stream::DeserializeBody{ 
                                                    serde: e,
                                                    status_code,
                                                    headers,response_text
                                                }
                                            ),
                                        },
                                        Err(e) => Err(
                                            #api_request_unexpected_error_path_token_stream::FailedToGetResponseText {
                                                reqwest: e,
                                                status_code,
                                                headers,
                                            }
                                        ),
                                    }
                                }
                            });
                        }
                    },
                }
            });
            if let false = is_last_element_found {
                panic!("{proc_macro_name_ident_stringified} false = is_last_element_found");
            }
            status_code_enums_try_from_variants
        };
        quote::quote! {
            async fn #try_from_response_operation_lower_case_token_stream(response: reqwest::Response) -> Result<#try_operation_response_variants_camel_case_token_stream, #api_request_unexpected_error_path_token_stream> {
                let status_code = response.status();
                let headers = response.headers().clone();
                #(#status_code_enums_try_from)*
            }
        }
    };
    let impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream_handle_token_stream = {
        let impl_try_from_ident_response_variants_token_stream_for_desirable_logic_handle_mapped_token_stream = type_variants_from_request_response_syn_variants
            .iter()
            .map(
                |error_variant_attribute| {
                    let variant_ident = &error_variant_attribute.ident;
                    let fields_named = if let syn::Fields::Named(fields_named) = &error_variant_attribute.fields {
                        fields_named
                    }
                    else {
                        panic!("{proc_macro_name_ident_stringified} expected fields would be named");
                    };
                    let fields_name_mapped_into_token_stream = fields_named.named.iter().map(|field|{
                        let field_ident = field.ident.clone().unwrap_or_else(|| panic!(
                            "{proc_macro_name_ident_stringified} field.ident {}",
                            proc_macro_helpers::error_occurence::hardcode::IS_NONE_STRINGIFIED
                        ));
                        quote::quote! {#field_ident}
                    }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
                    quote::quote! {
                        #try_operation_response_variants_camel_case_token_stream::#variant_ident {
                            #(#fields_name_mapped_into_token_stream),*
                        } => Err(#operation_with_serialize_deserialize_camel_case_token_stream::#variant_ident {
                            #(#fields_name_mapped_into_token_stream),*
                        })
                    }
                },
            )
            .collect::<std::vec::Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            impl TryFrom<#try_operation_response_variants_camel_case_token_stream> for #desirable_type_token_stream {
                type Error = #operation_with_serialize_deserialize_camel_case_token_stream;
                fn try_from(value: #try_operation_response_variants_camel_case_token_stream) -> Result<Self, Self::Error> {
                    match value {
                        #try_operation_response_variants_camel_case_token_stream::#desirable_token_stream(i) => Ok(i),
                        #(#impl_try_from_ident_response_variants_token_stream_for_desirable_logic_handle_mapped_token_stream),*
                    }
                }
            }
        }
    };
    let ident_request_error_logic_token_stream_handle_token_stream = {
        quote::quote! {
            #error_named_derive_token_stream
            pub enum #try_operation_request_error_token_stream {
                ExpectedType {
                    #eo_display_with_serialize_deserialize_token_stream
                    expected_type: #try_operation_with_serialize_deserialize_token_stream,
                    #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                },
                UnexpectedStatusCode {
                    #eo_display_token_stream
                    status_code: http::StatusCode,
                    #eo_display_foreign_type_token_stream
                    headers: reqwest::header::HeaderMap,
                    #eo_display_foreign_type_token_stream
                    response_text_result: #crate_common_api_request_unexpected_error_response_text_result_token_stream,
                    #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                },
                FailedToGetResponseText {
                    #eo_display_foreign_type_token_stream
                    reqwest: reqwest::Error,
                    #eo_display_token_stream
                    status_code: http::StatusCode,
                    #eo_display_foreign_type_token_stream
                    headers: reqwest::header::HeaderMap,
                    #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                },
                DeserializeResponse {
                    #eo_display_token_stream
                    serde: serde_json::Error,
                    #eo_display_token_stream
                    status_code: http::StatusCode,
                    #eo_display_foreign_type_token_stream
                    headers: reqwest::header::HeaderMap,
                    #eo_display_with_serialize_deserialize_token_stream
                    response_text: std::string::String,
                    #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                },
                Reqwest {
                    #eo_display_foreign_type_token_stream
                    reqwest: reqwest::Error,
                    #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                },
            }
        }
    };
    let extraction_logic_token_stream_handle_token_stream = {
        let tvfrr_extraction_logic_try_operation_lower_case_token_stream = {
            let tvfrr_extraction_logic_try_operation_lower_case_stringified =
                format!("tvfrr_extraction_logic_try_{operation_lower_case_stringified}");
            tvfrr_extraction_logic_try_operation_lower_case_stringified
            .parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {tvfrr_extraction_logic_try_operation_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_from_response_try_operation_lower_case_token_stream = {
            let try_from_response_try_operation_lower_case_stringified =
                format!("try_from_response_try_{operation_lower_case_stringified}");
            try_from_response_try_operation_lower_case_stringified
            .parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_from_response_try_operation_lower_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_from_response_try_operation_lower_case_token_stream_result_ok_token_stream = match is_response_with_body {
            true => quote::quote!{
                match #desirable_type_token_stream::try_from(variants){
                    Ok(value) => Ok(value),
                    Err(e) => Err(#try_operation_request_error_token_stream::ExpectedType {
                        expected_type: e,
                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                    }),
                }
            },
            false => quote::quote!{
                Ok(#desirable_type_token_stream)
            },
        };
        quote::quote! {
            async fn #tvfrr_extraction_logic_try_operation_lower_case_token_stream<'a>(
                future: impl std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
            ) -> Result<
                #desirable_type_token_stream,
                #try_operation_request_error_token_stream,
            > {
                match future.await {
                    Ok(response) => match #try_from_response_try_operation_lower_case_token_stream(response).await {
                        Ok(variants) => #try_from_response_try_operation_lower_case_token_stream_result_ok_token_stream,
                        Err(e) => match e {
                            #crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream::StatusCode {
                                status_code,
                                headers,
                                response_text_result,
                            } => Err(#try_operation_request_error_token_stream::UnexpectedStatusCode {
                                status_code,
                                headers,
                                response_text_result,
                                #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream
                            }),
                            #crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream::FailedToGetResponseText {
                                reqwest,
                                status_code,
                                headers
                            } => Err(#try_operation_request_error_token_stream::FailedToGetResponseText {
                                reqwest,
                                status_code,
                                headers,
                                #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream
                            }),
                            #crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream::DeserializeBody {
                                serde,
                                status_code,
                                headers,
                                response_text,
                            } => Err(#try_operation_request_error_token_stream::DeserializeResponse {
                                serde,
                                status_code,
                                headers,
                                response_text,
                                #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream
                            }),
                        },
                    },
                    Err(e) => Err(#try_operation_request_error_token_stream::Reqwest {
                        reqwest: e,
                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                    }),
                }
            }
        }
    };
    let enum_status_codes_checker_name_logic_token_stream_handle_token_stream = {
        let enum_status_codes_checker_camel_case_token_stream = {
            let enum_status_codes_checker_camel_case_stringified = format!("{try_operation_camel_case_token_stream}StatusCodesChecker");
            enum_status_codes_checker_camel_case_stringified
            .parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {enum_status_codes_checker_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let enum_status_codes_checker_name_logic_token_stream_handle_mapped_token_stream = type_variants_from_request_response_syn_variants.iter().map(|error_variant_attribute| {
                let variant_ident = &error_variant_attribute.ident;
                let error_variant_attribute = proc_macro_helpers::attribute::Attribute::try_from(error_variant_attribute)
                .unwrap_or_else(|e| {panic!("{proc_macro_name_ident_stringified} variant {variant_ident} failed: {e}")});
                let variant_ident_attribute_camel_case_token_stream = {
                    let variant_ident_attribute_camel_case_stringified = format!("{variant_ident}{error_variant_attribute}");
                    variant_ident_attribute_camel_case_stringified
                    .parse::<proc_macro2::TokenStream>()
                    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {variant_ident_attribute_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                };
                quote::quote! {
                    #variant_ident_attribute_camel_case_token_stream,
                }
            },
        ).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            pub enum #enum_status_codes_checker_camel_case_token_stream {
                #(#enum_status_codes_checker_name_logic_token_stream_handle_mapped_token_stream)*
            }
        }
    };
    let axum_response_into_response_logic_token_stream_handle_token_stream = {
        let axum_response_into_response_logic_token_stream_handle_mapped_token_stream = type_variants_from_request_response_syn_variants.iter().map(
            |error_variant_attribute| {
                let variant_ident = &error_variant_attribute.ident;
                let fields_named = if let syn::Fields::Named(fields_named) = &error_variant_attribute.fields {
                    fields_named
                }
                else {
                    panic!("{proc_macro_name_ident_stringified} expected fields would be named");
                };
                let fields_anonymous_types_mapped_into_token_stream = fields_named.named.iter().map(|field|{
                    let field_ident = field.ident.clone().unwrap_or_else(|| panic!(
                        "{proc_macro_name_ident_stringified} field.ident {}",
                        proc_macro_helpers::error_occurence::hardcode::IS_NONE_STRINGIFIED
                    ));
                    quote::quote! {#field_ident: _}
                }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
                quote::quote! {
                    #try_operation_response_variants_camel_case_token_stream::#variant_ident {
                        #(#fields_anonymous_types_mapped_into_token_stream),*
                    } => {
                        let mut res = axum::Json(self).into_response();
                        *res.status_mut() = #http_status_code_quote_token_stream;
                        res
                    }
                }
            }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            impl axum::response::IntoResponse for #try_operation_response_variants_camel_case_token_stream {
                fn into_response(self) -> axum::response::Response {
                    match &self {
                        #try_operation_response_variants_camel_case_token_stream::#desirable_token_stream(_) => {
                            let mut res = axum::Json(self).into_response();
                            *res.status_mut() = #http_status_code_quote_token_stream;//http::StatusCode::CREATED
                            res
                        }
                        #(#axum_response_into_response_logic_token_stream_handle_mapped_token_stream),*
                    }
                }
            }
        }
    };
    // println!("{}");
    quote::quote! {
        #try_operation_token_stream
        #enum_with_serialize_deserialize_logic_token_stream_handle_token_stream
        #from_logic_token_stream_handle_token_stream
        #impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream_handle_token_stream
        #generated_status_code_enums_with_from_impls_logic_token_stream_handle_token_stream
        #try_from_response_logic_token_stream_handle_token_stream
        #impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream_handle_token_stream
        #ident_request_error_logic_token_stream_handle_token_stream
        #extraction_logic_token_stream_handle_token_stream
        #enum_status_codes_checker_name_logic_token_stream_handle_token_stream
        #axum_response_into_response_logic_token_stream_handle_token_stream
    }
}

pub fn construct_syn_variant(
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
                            ident: proc_macro2::Ident::new(&tvfrr_status_attribute.to_string_lower_case(), proc_macro2::Span::call_site()),
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