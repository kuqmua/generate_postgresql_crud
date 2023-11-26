#[derive(Clone)]
pub struct ErrorVariantAttribute {
    pub error_variant_attribute: proc_macro_helpers::attribute::Attribute,
    pub error_variant: ErrorVariant,
}
#[derive(Clone)]
pub struct ErrorVariant {
    pub error_variant_ident: proc_macro2::TokenStream,
    pub error_variant_fields: std::vec::Vec<ErrorVariantField>,
}

#[derive(Clone)]
pub struct ErrorVariantField {
    pub error_occurence_attribute: proc_macro2::TokenStream,
    pub field_name: proc_macro2::TokenStream,
    pub field_type: proc_macro2::TokenStream,
}

pub fn type_variants_from_request_response(
    try_operation_response_variants_token_stream: &proc_macro2::TokenStream,
    operation_with_serialize_deserialize_camel_case_token_stream: &proc_macro2::TokenStream, //KekwWithSerializeDeserialize
    ident_response_variants_token_stream: &proc_macro2::TokenStream, //KekwResponseVariants
    proc_macro_name_ident_stringified: &std::string::String,
    error_variant_attribute: &ErrorVariantAttribute,
) -> (
    proc_macro_helpers::attribute::Attribute, //attribute
    std::vec::Vec<proc_macro2::TokenStream>,  //enum_with_serialize_deserialize_logic_token_stream
    std::vec::Vec<proc_macro2::TokenStream>,  //from_logic_token_stream
    std::vec::Vec<proc_macro2::TokenStream>, //impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream
    std::vec::Vec<proc_macro2::TokenStream>, //impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream
    std::vec::Vec<proc_macro2::TokenStream>, //enum_status_codes_checker_name_logic_token_stream
    std::vec::Vec<proc_macro2::TokenStream>, //axum_response_into_response_logic_token_stream
) {
    let variant_ident_attribute_camel_case_token_stream = {
        let variant_ident_attribute_camel_case_stringified = format!(
            "{}{}",
            error_variant_attribute.error_variant.error_variant_ident,
            error_variant_attribute.error_variant_attribute
        );
        variant_ident_attribute_camel_case_stringified
        .parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {variant_ident_attribute_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let http_status_code_quote_token_stream = error_variant_attribute.error_variant_attribute.to_http_status_code_quote();
    let fields_name_mapped_into_token_stream = error_variant_attribute.error_variant.error_variant_fields
        .iter()
        .map(|element| {
            let field_name_token_stream = &element.field_name;
            quote::quote! {#field_name_token_stream}
        })
        .collect::<std::vec::Vec<proc_macro2::TokenStream>>();
    let fields_anonymous_types_mapped_into_token_stream = error_variant_attribute.error_variant.error_variant_fields
        .iter()
        .map(|element| {
            let field_name_token_stream = &element.field_name;
            quote::quote! {#field_name_token_stream: _}
        })
        .collect::<std::vec::Vec<proc_macro2::TokenStream>>();
    let fields_mapped_into_token_stream = error_variant_attribute.error_variant.error_variant_fields
        .iter()
        .map(|element| {
            let field_name_token_stream = &element.field_name;
            let field_type_token_stream = &element.field_type;
            quote::quote! {#field_name_token_stream: #field_type_token_stream}
        })
        .collect::<std::vec::Vec<proc_macro2::TokenStream>>();
    let variant_ident = &error_variant_attribute.error_variant.error_variant_ident;
    let enum_with_serialize_deserialize_logic_token_stream = {
        vec![quote::quote! {
            #variant_ident {
                #(#fields_mapped_into_token_stream),*
            }
        }]
    };
    let from_logic_token_stream = {
        vec![quote::quote! {
            #operation_with_serialize_deserialize_camel_case_token_stream::#variant_ident {
                #(#fields_name_mapped_into_token_stream),*
            } => Self::#variant_ident {
                #(#fields_name_mapped_into_token_stream),*
            }
        }]
    };
    let impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream = {
        vec![quote::quote! {
            #try_operation_response_variants_token_stream::#variant_ident {
                #(#fields_anonymous_types_mapped_into_token_stream),*
            } => #http_status_code_quote_token_stream
        }]
    };
    let impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream = {
        vec![quote::quote! {
                #ident_response_variants_token_stream::#variant_ident {
                    #(#fields_name_mapped_into_token_stream),*
                } => Err(#operation_with_serialize_deserialize_camel_case_token_stream::#variant_ident {
                    #(#fields_name_mapped_into_token_stream),*
                })
        }]
    };
    let enum_status_codes_checker_name_logic_token_stream = {
        vec![quote::quote! {
            #variant_ident_attribute_camel_case_token_stream,
        }]
    };
    let axum_response_into_response_logic_token_stream = {
        vec![quote::quote! {
            #ident_response_variants_token_stream::#variant_ident {
                #(#fields_anonymous_types_mapped_into_token_stream),*
            } => {
                let mut res = axum::Json(self).into_response();
                *res.status_mut() = #http_status_code_quote_token_stream;
                res
            }
        }]
    };
    (
        error_variant_attribute.error_variant_attribute.clone(),
        enum_with_serialize_deserialize_logic_token_stream,
        from_logic_token_stream,
        impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream,
        impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream,
        enum_status_codes_checker_name_logic_token_stream,
        axum_response_into_response_logic_token_stream,
    )
}

pub fn generate_status_code_enums_with_from_impls_logic_token_stream(
    derive_debug_serialize_deserialize_token_stream: &proc_macro2::TokenStream, //#[derive(Debug, serde::Serialize, serde::Deserialize)]
    ident_response_variants_stringified: &std::string::String,
    ident_response_variants_token_stream: &proc_macro2::TokenStream,
    try_operation_response_variants_token_stream: &proc_macro2::TokenStream,
    vec_status_codes: std::vec::Vec<ErrorVariantAttribute>,
    proc_macro_name_ident_stringified: &std::string::String,
    desirable_name_token_stream: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let vec_status_codes_len = vec_status_codes.len();
    let status_code_enums_with_from_impls_logic_token_stream = vec_status_codes.into_iter().fold(
        std::collections::HashMap::<proc_macro_helpers::attribute::Attribute, std::vec::Vec<ErrorVariant>>::with_capacity(vec_status_codes_len),
        |mut acc, element| {
            match acc.get_mut(&element.error_variant_attribute) {
                Some(value) => {
                    value.push(element.error_variant);
                },
                None => {
                    acc.insert(element.error_variant_attribute, vec![element.error_variant]);
                }
            }
            acc
        },
    ).into_iter().map(|(key,value)|{
        let ident_response_variants_attribute_token_stream = {
            let ident_response_variants_attribute_stingified = format!("{ident_response_variants_stringified}{key}");
            ident_response_variants_attribute_stingified
            .parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ident_response_variants_attribute_stingified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let try_operation_response_variants_attribute_token_stream = {
            let try_operation_response_variants_attribute_stingified = format!("{try_operation_response_variants_token_stream}{key}");
            try_operation_response_variants_attribute_stingified
            .parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_response_variants_attribute_stingified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let enum_variants_token_stream = value.iter().map(|element|{
            let error_variant_ident = &element.error_variant_ident;
            let fields_mapped_into_token_stream = element.error_variant_fields.iter().map(|element| {
                let field_name_token_stream = &element.field_name;
                let field_type_token_stream = &element.field_type;
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
            impl std::convert::From<#try_operation_response_variants_attribute_token_stream> for #try_operation_response_variants_token_stream {
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
}

pub fn generate_try_from_response_logic_token_stream(
    response_without_body: bool,
    desirable_name_token_stream: &proc_macro2::TokenStream,
    ident_lower_case_stringified: &std::string::String,
    ident_response_variants_stringified: &std::string::String,
    ident_response_variants_token_stream: &proc_macro2::TokenStream,
    try_operation_response_variants_token_stream: &proc_macro2::TokenStream,
    try_create_many_response_variants_token_stream: &proc_macro2::TokenStream,
    operation_lower_case_stringified: &std::string::String,
    desirable_attribute: &proc_macro_helpers::attribute::Attribute,
    proc_macro_name_ident_stringified: &std::string::String,
    vec_status_codes: std::vec::Vec<ErrorVariantAttribute>,
) -> proc_macro2::TokenStream {
    let vec_status_codes_len = vec_status_codes.len();
    let hashmap_unique_status_codes = vec_status_codes.into_iter().fold(
        std::collections::HashMap::<proc_macro_helpers::attribute::Attribute, std::vec::Vec<ErrorVariant>>::with_capacity(vec_status_codes_len),
        |mut acc, element| {
            match acc.get_mut(&element.error_variant_attribute) {
                Some(value) => {
                    value.push(element.error_variant);
                },
                None => {
                    acc.insert(element.error_variant_attribute, vec![element.error_variant]);
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
    let unique_status_codes = hashmap_unique_status_codes.iter().map(|(key, _)|key).collect::<Vec<&proc_macro_helpers::attribute::Attribute>>();
    let desirable_enum_name = {
        let status_code_enum_name_stingified = format!("{try_operation_response_variants_token_stream}{desirable_attribute}");
        status_code_enum_name_stingified
        .parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {status_code_enum_name_stingified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let http_status_code_quote_token_stream = desirable_attribute.to_http_status_code_quote();
    let crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream =
        quote::quote! {crate::common::api_request_unexpected_error::ApiRequestUnexpectedError};
    let crate_common_api_request_unexpected_error_response_text_result_token_stream =
        quote::quote! {crate::common::api_request_unexpected_error::ResponseTextResult};
    let ident_response_variants_attribute_token_stream = {
        let ident_response_variants_attribute_stingified =
            format!("{ident_response_variants_stringified}{desirable_attribute}");
        ident_response_variants_attribute_stingified
        .parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ident_response_variants_attribute_stingified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let api_request_unexpected_error_module_path_token_stream = quote::quote! { crate::common::api_request_unexpected_error };
    let api_request_unexpected_error_path_token_stream = quote::quote! { #api_request_unexpected_error_module_path_token_stream::ApiRequestUnexpectedError };
    let try_from_response_operation_lower_case_token_stream = proc_macro_helpers::type_variants_from_request_response::generate_try_from_response_ident_lower_case_token_stream(
        &operation_lower_case_stringified,
        &proc_macro_name_ident_stringified,
    );
    let status_code_enums_try_from = {
        let mut is_last_element_found = false;
        let desirable_status_code_case_token_stream = match response_without_body {
            true => quote::quote! {
                Ok(#ident_response_variants_token_stream::#desirable_name_token_stream(()))
            },
            false => quote::quote! {
                match response.text().await {
                    Ok(response_text) => match serde_json::from_str::<#desirable_enum_name>(&response_text){
                        Ok(value) => Ok(#try_operation_response_variants_token_stream::from(value)), 
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
        .for_each(
            |(index, status_code_attribute)| 
        {
            let status_code_enum_name_stringified = format!("{try_operation_response_variants_token_stream}{status_code_attribute}");
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
                                        Ok(value) => Ok(#try_operation_response_variants_token_stream::from(value)), 
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
    //
    quote::quote! {
        // async fn try_from_response_kekw(
        //     response: reqwest::Response,
        // ) -> Result<
        //     #ident_response_variants_token_stream,
        //     #crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream,
        // > {
        //     let status_code = response.status();
        //     let headers = response.headers().clone();
        //     if status_code == #http_status_code_quote_token_stream {
        //         match response.text().await {
        //             Ok(response_text) => match serde_json::from_str::<#ident_response_variants_attribute_token_stream>(&response_text) {
        //                 Ok(value) => Ok(#ident_response_variants_token_stream::from(value)),
        //                 Err(e) => Err(#crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream::DeserializeBody {
        //                     serde: e,
        //                     status_code,
        //                     headers,
        //                     response_text
        //                 }),
        //             },
        //             Err(e) => Err(#crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream::FailedToGetResponseText {
        //                 reqwest: e,
        //                 status_code,
        //                 headers,
        //             }),
        //         }
        //     } else {
        //         match response.text().await {
        //             Ok(response_text) => Err(#crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream::StatusCode {
        //                 status_code,
        //                 headers,
        //                 response_text_result: #crate_common_api_request_unexpected_error_response_text_result_token_stream::ResponseText(response_text)
        //             }),
        //             Err(e) => Err(#crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream::StatusCode {
        //                 status_code,
        //                 headers,
        //                 response_text_result: #crate_common_api_request_unexpected_error_response_text_result_token_stream::ReqwestError(e),
        //             }),
        //         }
        //     }
        // }
        async fn #try_from_response_operation_lower_case_token_stream(response: reqwest::Response) -> Result<#try_operation_response_variants_token_stream, #api_request_unexpected_error_path_token_stream> {
            let status_code = response.status();
            let headers = response.headers().clone();
            #(#status_code_enums_try_from)*
        }
    }
}
