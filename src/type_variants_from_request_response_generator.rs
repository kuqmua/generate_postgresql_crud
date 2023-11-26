pub fn type_variants_from_request_response_generator(
    desirable_attribute: proc_macro_helpers::attribute::Attribute,
    ident: &syn::Ident,
    ident_lower_case_stringified: &std::string::String,
    try_operation_camel_case_token_stream: &proc_macro2::TokenStream,
    try_operation_response_variants_token_stream: &proc_macro2::TokenStream, //KekwResponseVariants
    try_operation_response_variants_desirable_attribute_token_stream: &proc_macro2::TokenStream,
    operation_lower_case_stringified: &std::string::String,
    desirable_token_stream: &proc_macro2::TokenStream,
    desirable_type_token_stream: &proc_macro2::TokenStream, //std::vec::Vec<crate::server::postgres::uuid_wrapper::PossibleUuidWrapper>
    proc_macro_name_ident_stringified: &std::string::String,
    code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream: &proc_macro2::TokenStream,
    code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream: &proc_macro2::TokenStream,
    ident_with_serialize_deserialize_camel_case_token_stream: &proc_macro2::TokenStream,
    operation_with_serialize_deserialize_camel_case_token_stream: &proc_macro2::TokenStream,
    error_named_derive_token_stream: &proc_macro2::TokenStream,
    eo_display_attribute_token_stream: &proc_macro2::TokenStream,
    eo_display_foreign_type_token_stream: &proc_macro2::TokenStream,
    eo_display_with_serialize_deserialize_token_stream: &proc_macro2::TokenStream,
    derive_debug_serialize_deserialize_token_stream: &proc_macro2::TokenStream,
    //
    type_variants_from_request_response: std::vec::Vec<(
        proc_macro_helpers::attribute::Attribute, //attribute
        std::vec::Vec<proc_macro2::TokenStream>, //enum_with_serialize_deserialize_logic_token_stream
        std::vec::Vec<proc_macro2::TokenStream>, //from_logic_token_stream
        std::vec::Vec<proc_macro2::TokenStream>, //impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream
        std::vec::Vec<proc_macro2::TokenStream>, //impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream
        std::vec::Vec<proc_macro2::TokenStream>, //enum_status_codes_checker_name_logic_token_stream
        std::vec::Vec<proc_macro2::TokenStream>, //axum_response_into_response_logic_token_stream
    )>,
    generated_status_code_enums_with_from_impls_logic_token_stream: &proc_macro2::TokenStream,
    try_from_response_logic_token_stream_token_stream: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {

    let http_status_code_quote_token_stream = desirable_attribute.to_http_status_code_quote();
    let type_variants_from_request_response_len = type_variants_from_request_response.len();
    let ident_request_error_camel_case_token_stream = proc_macro_helpers::type_variants_from_request_response::generate_ident_request_error_camel_case_token_stream(
        &ident,
        &proc_macro_name_ident_stringified,
    );
    let try_operation_request_error_token_stream = {
        let try_operation_request_error_stringified =
            format!("{try_operation_camel_case_token_stream}RequestError");
        try_operation_request_error_stringified
        .parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_request_error_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let try_operation_with_serialize_deserialize_token_stream = {
        let try_operation_with_serialize_deserialize_stringified =
            format!("{try_operation_camel_case_token_stream}WithSerializeDeserialize");
        try_operation_with_serialize_deserialize_stringified
        .parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {try_operation_with_serialize_deserialize_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let try_from_response_lower_case_token_stream = proc_macro_helpers::type_variants_from_request_response::generate_try_from_response_ident_lower_case_token_stream(
        &ident_lower_case_stringified,
        &proc_macro_name_ident_stringified,
    );
    let crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream =
        quote::quote! {crate::common::api_request_unexpected_error::ApiRequestUnexpectedError};
    let crate_common_api_request_unexpected_error_response_text_result_token_stream =
        quote::quote! {crate::common::api_request_unexpected_error::ResponseTextResult};
    let ident_response_variants_desirable_attribute_token_stream = {
        let ident_response_variants_desirable_attribute_stringified =
            format!("{ident}ResponseVariants{desirable_attribute}");
        ident_response_variants_desirable_attribute_stringified
        .parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ident_response_variants_desirable_attribute_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };

    

    // let (
    //     attribute,
    //     enum_with_serialize_deserialize_logic_token_stream,
    //     from_logic_token_stream,
    //     impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream,
    //     impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream,
    //     enum_status_codes_checker_name_logic_token_stream,
    //     axum_response_into_response_logic_token_stream,
    // ) = type_variants_from_request_response.into_iter().fold(
    //     (
    //         std::vec::Vec::with_capacity(type_variants_from_request_response_len),
    //         std::vec::Vec::with_capacity(type_variants_from_request_response_len),
    //         std::vec::Vec::with_capacity(type_variants_from_request_response_len),
    //         std::vec::Vec::with_capacity(type_variants_from_request_response_len),
    //         std::vec::Vec::with_capacity(type_variants_from_request_response_len),
    //         std::vec::Vec::with_capacity(type_variants_from_request_response_len),
    //         std::vec::Vec::with_capacity(type_variants_from_request_response_len),
    //     ),
    //     |mut acc, element| {
    //         acc.0.push(element.0);
    //         acc.1.push(element.1);
    //         acc.2.push(element.2);
    //         acc.3.push(element.3);
    //         acc.4.push(element.4);
    //         acc.5.push(element.5);
    //         acc.6.push(element.6);
    //         acc
    //     },
    // );
    let enum_with_serialize_deserialize_logic_token_stream_handle_token_stream = {
        let enum_with_serialize_deserialize_logic_mapped_token_stream =
            type_variants_from_request_response
                .iter()
                .map(
                    |(
                        _, //attribute
                        enum_with_serialize_deserialize_logic_token_stream,
                        _, //from_logic_token_stream
                        _, //impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream
                        _, //impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream
                        _, //enum_status_codes_checker_name_logic_token_stream
                        _, //axum_response_into_response_logic_token_stream
                    )| quote::quote! {#(#enum_with_serialize_deserialize_logic_token_stream),*},
                )
                .collect::<Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            #derive_debug_serialize_deserialize_token_stream
            pub enum #try_operation_response_variants_token_stream {
                #desirable_token_stream(#desirable_type_token_stream),
                // Configuration {
                //     configuration_box_dyn_error: std::string::String,
                //     #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                // }
                #(#enum_with_serialize_deserialize_logic_mapped_token_stream),*
            }
        }
    };
    let from_logic_token_stream_handle_token_stream = {
        let from_logic_token_stream_mapped_token_stream = type_variants_from_request_response
            .iter()
            .map(
                |(
                    _, //attribute
                    _, //enum_with_serialize_deserialize_logic_token_stream
                    from_logic_token_stream,
                    _, //impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream
                    _, //impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream
                    _, //enum_status_codes_checker_name_logic_token_stream
                    _, //axum_response_into_response_logic_token_stream
                )| quote::quote! {#(#from_logic_token_stream),*},
            )
            .collect::<Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            impl std::convert::From<#try_operation_camel_case_token_stream> for #try_operation_response_variants_token_stream {
                fn from(value: #try_operation_camel_case_token_stream) -> Self {
                    match value.into_serialize_deserialize_version() {
                        // KekwWithSerializeDeserialize::Configuration {
                        //     configuration_box_dyn_error,
                        //     code_occurence,
                        // } => Self::Configuration {
                        //     configuration_box_dyn_error,
                        //     code_occurence,
                        // },
                        #(#from_logic_token_stream_mapped_token_stream),*
                    }
                }
            }
        }
    };
    let impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream_handle_token_stream = {
        let impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream_handle_mapped_token_stream = type_variants_from_request_response
            .iter()
            .map(
                |(
                    _, //attribute
                    _, //enum_with_serialize_deserialize_logic_token_stream
                    _, //from_logic_token_stream
                    impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream,
                    _, //impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream
                    _, //enum_status_codes_checker_name_logic_token_stream
                    _, //axum_response_into_response_logic_token_stream
                )| quote::quote! {#(#impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream),*},
            )
            .collect::<Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            impl std::convert::From<&#try_operation_response_variants_token_stream> for http::StatusCode {
                fn from(value: &#try_operation_response_variants_token_stream) -> Self {
                    match value {
                        #try_operation_response_variants_token_stream::#desirable_token_stream(_) => #http_status_code_quote_token_stream,//http::StatusCode::CREATED
                        // KekwResponseVariants::Configuration {
                        //     configuration_box_dyn_error: _,
                        //     code_occurence: _,
                        // } => http::StatusCode::INTERNAL_SERVER_ERROR,
                        #(#impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream_handle_mapped_token_stream),*
                    }
                }
            }
        }
    };
    let generated_status_code_enums_with_from_impls_logic_token_stream_handle_token_stream = {
        quote::quote! {
            #derive_debug_serialize_deserialize_token_stream
            enum #try_operation_response_variants_desirable_attribute_token_stream {
                #desirable_token_stream(#desirable_type_token_stream),
            }
            impl std::convert::From<#try_operation_response_variants_desirable_attribute_token_stream> for #try_operation_response_variants_token_stream {
                fn from(value: #try_operation_response_variants_desirable_attribute_token_stream) -> Self {
                    match value {
                        #try_operation_response_variants_desirable_attribute_token_stream::#desirable_token_stream(i) => Self::#desirable_token_stream(i),
                    }
                }
            }




            // #[derive(Debug, serde::Serialize, serde::Deserialize)]
            // enum KekwResponseVariantsTvfrr500InternalServerError {
            //     Configuration {
            //         configuration_box_dyn_error: std::string::String,
            //         #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
            //     },
            // }
            // impl std::convert::From<KekwResponseVariantsTvfrr500InternalServerError> for #try_operation_response_variants_token_stream {
            //     fn from(value: KekwResponseVariantsTvfrr500InternalServerError) -> Self {
            //         match value {
            //             KekwResponseVariantsTvfrr500InternalServerError::Configuration {
            //                 configuration_box_dyn_error,
            //                 code_occurence,
            //             } => Self::Configuration {
            //                 configuration_box_dyn_error,
            //                 code_occurence,
            //             },
            //         }
            //     }
            // }
            #generated_status_code_enums_with_from_impls_logic_token_stream
        }
    };
    let try_from_response_logic_token_stream_handle_token_stream = {
        quote::quote! {
            // async fn #try_from_response_lower_case_token_stream(
            //     response: reqwest::Response,
            // ) -> Result<
            //     #try_operation_response_variants_token_stream,
            //     #crate_common_api_request_unexpected_error_api_request_unexpected_error_token_stream,
            // > {
            //     let status_code = response.status();
            //     let headers = response.headers().clone();
            //     if status_code == #http_status_code_quote_token_stream {
            //         match response.text().await {
            //             Ok(response_text) => match serde_json::from_str::<#ident_response_variants_desirable_attribute_token_stream>(&response_text) {
            //                 Ok(value) => Ok(#try_operation_response_variants_token_stream::from(value)),
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
            #try_from_response_logic_token_stream_token_stream
        }
    };
    let impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream_handle_token_stream = {
        let impl_try_from_ident_response_variants_token_stream_for_desirable_logic_handle_mapped_token_stream = type_variants_from_request_response
            .iter()
            .map(
                |(
                    _, //attribute
                    _, //enum_with_serialize_deserialize_logic_token_stream
                    _, //from_logic_token_stream
                    _, //impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream
                    impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream,
                    _, //enum_status_codes_checker_name_logic_token_stream
                    _, //axum_response_into_response_logic_token_stream
                )| quote::quote! {#(#impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream),*},
            )
            .collect::<Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            impl TryFrom<#try_operation_response_variants_token_stream> for #desirable_type_token_stream {
                type Error = #operation_with_serialize_deserialize_camel_case_token_stream;
                fn try_from(value: #try_operation_response_variants_token_stream) -> Result<Self, Self::Error> {
                    match value {
                        #try_operation_response_variants_token_stream::#desirable_token_stream(i) => Ok(i),
                        // #try_operation_response_variants_token_stream::Configuration {
                        //     configuration_box_dyn_error,
                        //     code_occurence,
                        // } => Err(KekwWithSerializeDeserialize::Configuration {
                        //     configuration_box_dyn_error,
                        //     code_occurence,
                        // }),
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
                    #eo_display_attribute_token_stream
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
                    #eo_display_attribute_token_stream
                    status_code: http::StatusCode,
                    #eo_display_foreign_type_token_stream
                    headers: reqwest::header::HeaderMap,
                    #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                },
                DeserializeResponse {
                    #eo_display_attribute_token_stream
                    serde: serde_json::Error,
                    #eo_display_attribute_token_stream
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
        // let tvfrr_extraction_logic_lower_case_token_stream = proc_macro_helpers::type_variants_from_request_response::generate_tvfrr_extraction_logic_lower_case_token_stream(
        //     &ident_lower_case_stringified,
        //     &proc_macro_name_ident_stringified,
        // );
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
        quote::quote! {
            async fn #tvfrr_extraction_logic_try_operation_lower_case_token_stream<'a>(
                future: impl std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
            ) -> Result<
                #desirable_type_token_stream,
                #try_operation_request_error_token_stream,
            > {
                match future.await {
                    Ok(response) => match #try_from_response_try_operation_lower_case_token_stream(response).await {
                        Ok(variants) => match #desirable_type_token_stream::try_from(variants){
                            Ok(value) => Ok(value),
                            Err(e) => Err(#try_operation_request_error_token_stream::ExpectedType {
                                expected_type: e,
                                #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                            }),
                        },
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
        // let enum_status_codes_checker_camel_case_token_stream = proc_macro_helpers::type_variants_from_request_response::generate_enum_status_codes_checker_camel_case_token_stream(
        //     &ident,
        //     proc_macro_name_ident_stringified,
        // );
        // TryCreateManyStatusCodesChecker
        let enum_status_codes_checker_camel_case_token_stream = {
            let enum_status_codes_checker_camel_case_stringified = format!("{try_operation_camel_case_token_stream}StatusCodesChecker");
            enum_status_codes_checker_camel_case_stringified
            .parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {enum_status_codes_checker_camel_case_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
        };
        let enum_status_codes_checker_name_logic_token_stream_handle_mapped_token_stream =
            type_variants_from_request_response
                .iter()
                .map(
                    |(
                        _, //attribute
                        _, //enum_with_serialize_deserialize_logic_token_stream
                        _, //from_logic_token_stream
                        _, //impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream
                        _, //impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream
                        enum_status_codes_checker_name_logic_token_stream,
                        _, //axum_response_into_response_logic_token_stream
                    )| quote::quote! {#(#enum_status_codes_checker_name_logic_token_stream)*},
                )
                .collect::<Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            pub enum #enum_status_codes_checker_camel_case_token_stream {
                // ConfigurationTvfrr500InternalServerError,
                #(#enum_status_codes_checker_name_logic_token_stream_handle_mapped_token_stream)*
            }
        }
    };
    let axum_response_into_response_logic_token_stream_handle_token_stream = {
        let axum_response_into_response_logic_token_stream_handle_mapped_token_stream =
            type_variants_from_request_response
                .iter()
                .map(
                    |(
                        _, //attribute
                        _, //enum_with_serialize_deserialize_logic_token_stream
                        _, //from_logic_token_stream
                        _, //impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream
                        _, //impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream
                        _, //enum_status_codes_checker_name_logic_token_stream
                        axum_response_into_response_logic_token_stream,
                    )| quote::quote! {#(#axum_response_into_response_logic_token_stream),*},
                )
                .collect::<Vec<proc_macro2::TokenStream>>();
        quote::quote! {
            impl axum::response::IntoResponse for #try_operation_response_variants_token_stream {
                fn into_response(self) -> axum::response::Response {
                    match &self {
                        #try_operation_response_variants_token_stream::#desirable_token_stream(_) => {
                            let mut res = axum::Json(self).into_response();
                            *res.status_mut() = #http_status_code_quote_token_stream;//http::StatusCode::CREATED
                            res
                        }
                        // #try_operation_response_variants_token_stream::Configuration {
                        //     configuration_box_dyn_error: _,
                        //     code_occurence: _,
                        // } => {
                        //     let mut res = axum::Json(self).into_response();
                        //     *res.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
                        //     res
                        // }
                        #(#axum_response_into_response_logic_token_stream_handle_mapped_token_stream),*
                    }
                }
            }
        }
    };
    // println!("{}");
    quote::quote! {
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
