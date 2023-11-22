pub struct ErrorVariantField {
    field_name: proc_macro2::TokenStream,
    field_type: proc_macro2::TokenStream,
}

fn type_variants_from_request_response(
    ident_with_serialize_deserialize_camel_case_token_stream: &proc_macro2::TokenStream, //KekwWithSerializeDeserialize
    ident_response_variants_token_stream: &proc_macro2::TokenStream, //KekwResponseVariants
    attribute: proc_macro_helpers::attribute::Attribute,
    variant_ident: &proc_macro2::TokenStream, //Configuration
    proc_macro_name_ident_stringified: &std::string::String,
    fields: std::vec::Vec<ErrorVariantField>,
) -> (
    proc_macro_helpers::attribute::Attribute, //attribute
    std::vec::Vec<proc_macro2::TokenStream>,  //enum_with_serialize_deserialize_logic_token_stream
    std::vec::Vec<proc_macro2::TokenStream>,  //from_logic_token_stream
    std::vec::Vec<proc_macro2::TokenStream>, //impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream
    std::vec::Vec<proc_macro2::TokenStream>, //try_from_response_logic_token_stream
    std::vec::Vec<proc_macro2::TokenStream>, //impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream
    std::vec::Vec<proc_macro2::TokenStream>, //enum_status_codes_checker_name_logic_token_stream
    std::vec::Vec<proc_macro2::TokenStream>, //axum_response_into_response_logic_token_stream
) {
    let variant_ident_attribute_camel_case_token_stream = {
        let variant_ident_attribute_camel_case_stringified = format!("{variant_ident}{attribute}");
        variant_ident_attribute_camel_case_stringified
        .parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {variant_ident_attribute_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let http_status_code_quote_token_stream = attribute.to_http_status_code_quote();
    let fields_name_mapped_into_token_stream = fields
        .iter()
        .map(|element| {
            let field_name_token_stream = &element.field_name;
            quote::quote! {#field_name_token_stream}
        })
        .collect::<std::vec::Vec<proc_macro2::TokenStream>>();
    let fields_anonymous_types_mapped_into_token_stream = fields
        .iter()
        .map(|element| {
            let field_name_token_stream = &element.field_name;
            quote::quote! {#field_name_token_stream: _}
        })
        .collect::<std::vec::Vec<proc_macro2::TokenStream>>();
    // let fields_type_mapped_into_token_stream = fields.iter().map(|element|{
    //     let field_type_token_stream = &element.field_type;
    //     quote::quote!{#field_type_token_stream}
    // }).collect::<std::vec::Vec<proc_macro2::TokenStream>>();
    let fields_mapped_into_token_stream = fields
        .iter()
        .map(|element| {
            let field_name_token_stream = &element.field_name;
            let field_type_token_stream = &element.field_type;
            quote::quote! {#field_name_token_stream: #field_type_token_stream}
        })
        .collect::<std::vec::Vec<proc_macro2::TokenStream>>();
    let enum_with_serialize_deserialize_logic_token_stream = {
        vec![quote::quote! {
            #variant_ident {
                #(#fields_mapped_into_token_stream),*
            }
        }]
    };
    let from_logic_token_stream = {
        vec![quote::quote! {
            #ident_with_serialize_deserialize_camel_case_token_stream::#variant_ident {
                #(#fields_name_mapped_into_token_stream),*
            } => Self::#variant_ident {
                #(#fields_name_mapped_into_token_stream),*
            }
        }]
    };
    let impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream = {
        vec![quote::quote! {
            #ident_response_variants_token_stream::#variant_ident {
                #(#fields_anonymous_types_mapped_into_token_stream),*
            } => #http_status_code_quote_token_stream
        }]
    };
    let try_from_response_logic_token_stream = {
        //todo else if for each different status code
        vec![quote::quote! {
            // async fn try_from_response_kekw(
            //     response: reqwest::Response,
            // ) -> Result<
            //     #ident_response_variants_token_stream,
            //     crate::common::api_request_unexpected_error::ApiRequestUnexpectedError,
            // > {
            //     let status_code = response.status();
            //     let headers = response.headers().clone();
            //     if status_code == http::StatusCode::CREATED {
            //         match response.text().await {
            //             Ok(response_text) => match serde_json::from_str::<KekwResponseVariantsTvfrr201Created>(&response_text) {
            //                 Ok(value) => Ok(#ident_response_variants_token_stream::from(value)),
            //                 Err(e) => Err(crate::common::api_request_unexpected_error::ApiRequestUnexpectedError::DeserializeBody {
            //                     serde: e,
            //                     status_code,
            //                     headers,
            //                     response_text
            //                 }),
            //             },
            //             Err(e) => Err(crate::common::api_request_unexpected_error::ApiRequestUnexpectedError::FailedToGetResponseText {
            //                 reqwest: e,
            //                 status_code,
            //                 headers,
            //             }),
            //         }
            //     } else {
            //         match response.text().await {
            //             Ok(response_text) => Err(crate::common::api_request_unexpected_error::ApiRequestUnexpectedError::StatusCode {
            //                 status_code,
            //                 headers,
            //                 response_text_result: crate::common::api_request_unexpected_error::ResponseTextResult::ResponseText(response_text)
            //             }),
            //             Err(e) => Err(crate::common::api_request_unexpected_error::ApiRequestUnexpectedError::StatusCode {
            //                 status_code,
            //                 headers,
            //                 response_text_result: crate::common::api_request_unexpected_error::ResponseTextResult::ReqwestError(e),
            //             }),
            //         }
            //     }
            // }
        }]
    };
    let impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream = {
        vec![quote::quote! {
                #ident_response_variants_token_stream::#variant_ident {
                    #(#fields_name_mapped_into_token_stream),*
                } => Err(#ident_with_serialize_deserialize_camel_case_token_stream::#variant_ident {
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
        attribute,
        //
        enum_with_serialize_deserialize_logic_token_stream,
        from_logic_token_stream,
        impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream,
        try_from_response_logic_token_stream,
        impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream,
        enum_status_codes_checker_name_logic_token_stream,
        axum_response_into_response_logic_token_stream,
    )
}

pub struct ErrorVariantAttribute {
    error_variant_attribute: proc_macro_helpers::attribute::Attribute,
    error_variant: ErrorVariant,
}

pub struct ErrorVariant {
    error_variant_ident: proc_macro2::TokenStream,
    error_variant_fields: std::vec::Vec<ErrorVariantField>,
}

fn generate_status_code_enums_with_from_impls_logic_token_stream(
    derive_debug_serialize_deserialize_token_stream: &proc_macro2::TokenStream, //#[derive(Debug, serde::Serialize, serde::Deserialize)]
    ident_response_variants_stringified: &std::string::String,
    ident_response_variants_token_stream: &proc_macro2::TokenStream,
    vec_status_codes: std::vec::Vec<ErrorVariantAttribute>,
    proc_macro_name_ident_stringified: &std::string::String,
    desirable_attribute: proc_macro_helpers::attribute::Attribute,
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
                #ident_response_variants_attribute_token_stream::#error_variant_ident {
                    #(#fields_name_mapped_into_token_stream),*
                } => Self::Configuration {
                    #(#fields_name_mapped_into_token_stream),*
                }
            }
        });
        quote::quote!{
            #derive_debug_serialize_deserialize_token_stream
            enum #ident_response_variants_attribute_token_stream {
                #(#enum_variants_token_stream),*
            }
            impl std::convert::From<#ident_response_variants_attribute_token_stream> for #ident_response_variants_token_stream {
                fn from(value: #ident_response_variants_attribute_token_stream) -> Self {
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

fn generate_try_from_response_logic_token_stream(
    ident_response_variants_stringified: &std::string::String,
    ident_response_variants_token_stream: &proc_macro2::TokenStream,
    attribute: proc_macro_helpers::attribute::Attribute,
    proc_macro_name_ident_stringified: &std::string::String,
) -> proc_macro2::TokenStream {
    let http_status_code_quote_token_stream = attribute.to_http_status_code_quote();
    let crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream =
        quote::quote! {crate::common::api_request_unexpected_error::ApiRequestUnexpectedError};
    let crate_common_api_request_unexpected_error_response_text_result_token_stream =
        quote::quote! {crate::common::api_request_unexpected_error::ResponseTextResult};
    let ident_response_variants_attribute_token_stream = {
        let ident_response_variants_attribute_stingified =
            format!("{ident_response_variants_stringified}{attribute}");
        ident_response_variants_attribute_stingified
        .parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ident_response_variants_attribute_stingified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    // let variant_ident_attribute_camel_case_token_stream = {
    //     let variant_ident_attribute_camel_case_stringified = format!("{variant_ident}{attribute}");
    //     variant_ident_attribute_camel_case_stringified
    //     .parse::<proc_macro2::TokenStream>()
    //     .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {variant_ident_attribute_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    // };
    quote::quote! {
        async fn try_from_response_kekw(
            response: reqwest::Response,
        ) -> Result<
            #ident_response_variants_token_stream,
            #crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream,
        > {
            let status_code = response.status();
            let headers = response.headers().clone();
            if status_code == #http_status_code_quote_token_stream {
                match response.text().await {
                    Ok(response_text) => match serde_json::from_str::<#ident_response_variants_attribute_token_stream>(&response_text) {
                        Ok(value) => Ok(#ident_response_variants_token_stream::from(value)),
                        Err(e) => Err(#crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream::DeserializeBody {
                            serde: e,
                            status_code,
                            headers,
                            response_text
                        }),
                    },
                    Err(e) => Err(#crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream::FailedToGetResponseText {
                        reqwest: e,
                        status_code,
                        headers,
                    }),
                }
            } else {
                match response.text().await {
                    Ok(response_text) => Err(#crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream::StatusCode {
                        status_code,
                        headers,
                        response_text_result: #crate_common_api_request_unexpected_error_response_text_result_token_stream::ResponseText(response_text)
                    }),
                    Err(e) => Err(#crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream::StatusCode {
                        status_code,
                        headers,
                        response_text_result: #crate_common_api_request_unexpected_error_response_text_result_token_stream::ReqwestError(e),
                    }),
                }
            }
        }
    }
}
