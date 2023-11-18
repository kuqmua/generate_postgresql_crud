fn type_variants_from_request_response_generator(
    desirable_attribute: proc_macro_helpers::attribute::Attribute,
    ident: &syn::Ident,
    ident_lower_case_stringified: &std::string::String,
    ident_response_variants_token_stream: &proc_macro2::TokenStream,//KekwResponseVariants
    desirable_token_stream: &proc_macro2::TokenStream,
    desirable_type_token_stream: &proc_macro2::TokenStream,//std::vec::Vec<crate::server::postgres::uuid_wrapper::PossibleUuidWrapper>
    proc_macro_name_ident_stringified: &std::string::String,
    code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream: &proc_macro2::TokenStream,
    code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream: &proc_macro2::TokenStream,
    with_serialize_deserialize_camel_case_stringified: &std::string::String,
    //
    type_variants_from_request_response: std::vec::Vec<impl crate::type_variants_from_request_response::TypeVariantsFromRequestResponse>,
    // attribute: proc_macro_helpers::attribute::Attribute,
    // enum_with_serialize_deserialize_logic_token_stream: std::vec::Vec<proc_macro2::TokenStream>,
    // from_logic_token_stream: std::vec::Vec<proc_macro2::TokenStream>,
    // impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream: std::vec::Vec<proc_macro2::TokenStream>,
    // generated_status_code_enums_with_from_impls_logic_token_stream: proc_macro2::TokenStream,
    // try_from_response_logic_token_stream: proc_macro2::TokenStream,
    // impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream: proc_macro2::TokenStream,
    // enum_status_codes_checker_name_logic_token_stream: proc_macro2::TokenStream,
    // axum_response_into_response_logic_token_stream: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let ident_with_serialize_deserialize_camel_case_token_stream = {
        let ident_request_error_stringified = format!("{ident}{with_serialize_deserialize_camel_case_stringified}");
        ident_request_error_stringified
        .parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {ident_request_error_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
    };
    let http_status_code_quote_token_stream = desirable_attribute.to_http_status_code_quote();
    let type_variants_from_request_response_len = type_variants_from_request_response.len();
    let ident_request_error_token_stream = proc_macro_helpers::type_variants_from_request_response::generate_ident_request_error_token_stream(
        &ident,
        &proc_macro_name_ident_stringified,
    );
    let try_from_response_lower_case_token_stream = proc_macro_helpers::type_variants_from_request_response::generate_try_from_response_lower_case_token_stream(
        &ident_lower_case_stringified,
        &proc_macro_name_ident_stringified,
    );
    let (
        attributes,
        enum_with_serialize_deserialize_logic_token_stream,
        from_logic_token_stream,
        impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream,
        generated_status_code_enums_with_from_impls_logic_token_stream,
        try_from_response_logic_token_stream,
        impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream,
        enum_status_codes_checker_name_logic_token_stream,
        axum_response_into_response_logic_token_stream,
    ) = type_variants_from_request_response.into_iter()
    .fold((
        std::vec::Vec::with_capacity(type_variants_from_request_response_len),
        std::vec::Vec::with_capacity(type_variants_from_request_response_len),
        std::vec::Vec::with_capacity(type_variants_from_request_response_len),
        std::vec::Vec::with_capacity(type_variants_from_request_response_len),
        std::vec::Vec::with_capacity(type_variants_from_request_response_len),
        std::vec::Vec::with_capacity(type_variants_from_request_response_len),
        std::vec::Vec::with_capacity(type_variants_from_request_response_len),
        std::vec::Vec::with_capacity(type_variants_from_request_response_len),
        std::vec::Vec::with_capacity(type_variants_from_request_response_len),
    ), |mut acc, element| {
        acc.0.push(element.attribute());
        acc.1.push(element.enum_with_serialize_deserialize_logic_token_stream());
        acc.2.push(element.from_logic_token_stream());
        acc.3.push(element.impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream());
        acc.4.push(element.generated_status_code_enums_with_from_impls_logic_token_stream());
        acc.5.push(element.try_from_response_logic_token_stream());
        acc.6.push(element.impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream());
        acc.7.push(element.enum_status_codes_checker_name_logic_token_stream());
        acc.8.push(element.axum_response_into_response_logic_token_stream());
        acc
    });
    let enum_with_serialize_deserialize_logic_token_stream_handle_token_stream = {
        quote::quote!{
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            pub enum #ident_response_variants_token_stream {
                #desirable_token_stream(#desirable_type_token_stream),
                // Configuration {
                //     configuration_box_dyn_error: std::string::String,
                //     #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                // }
                #(#enum_with_serialize_deserialize_logic_token_stream),*
            }
        }
    };
    let from_logic_token_stream_handle_token_stream = {
        quote::quote!{
            impl std::convert::From<#ident> for #ident_response_variants_token_stream {
                fn from(value: #ident) -> Self {
                    match value.into_serialize_deserialize_version() {
                        // KekwWithSerializeDeserialize::Configuration {
                        //     configuration_box_dyn_error,
                        //     code_occurence,
                        // } => Self::Configuration {
                        //     configuration_box_dyn_error,
                        //     code_occurence,
                        // },
                        #(#from_logic_token_stream),*
                    }
                }
            }
        }
    };
    let impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream_handle_token_stream = {
        quote::quote!{
            impl std::convert::From<&#ident_response_variants_token_stream> for http::StatusCode {
                fn from(value: &#ident_response_variants_token_stream) -> Self {
                    match value {
                        #ident_response_variants_token_stream::#desirable_token_stream(_) => #http_status_code_quote_token_stream,//http::StatusCode::CREATED
                        // KekwResponseVariants::Configuration {
                        //     configuration_box_dyn_error: _,
                        //     code_occurence: _,
                        // } => http::StatusCode::INTERNAL_SERVER_ERROR,
                        #(#impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream),*
                    }
                }
            }
        }
    };
    let generated_status_code_enums_with_from_impls_logic_token_stream_handle_token_stream = {
        quote::quote!{
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            enum KekwResponseVariantsTvfrr201Created {
                #desirable_token_stream(#desirable_type_token_stream),
            }
            impl std::convert::From<KekwResponseVariantsTvfrr201Created> for #ident_response_variants_token_stream {
                fn from(value: KekwResponseVariantsTvfrr201Created) -> Self {
                    match value {
                        KekwResponseVariantsTvfrr201Created::#desirable_token_stream(i) => Self::#desirable_token_stream(i),
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
            // impl std::convert::From<KekwResponseVariantsTvfrr500InternalServerError> for #ident_response_variants_token_stream {
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
        }
    };
    let try_from_response_logic_token_stream_handle_token_stream = {
        quote::quote!{
            async fn #try_from_response_lower_case_token_stream(
                response: reqwest::Response,
            ) -> Result<
                #ident_response_variants_token_stream,
                crate::common::api_request_unexpected_error::ApiRequestUnexpectedError,
            > {
                let status_code = response.status();
                let headers = response.headers().clone();
                if status_code == http::StatusCode::CREATED {
                    match response.text().await {
                        Ok(response_text) => match serde_json::from_str::<KekwResponseVariantsTvfrr201Created>(&response_text) {
                            Ok(value) => Ok(#ident_response_variants_token_stream::from(value)), 
                            Err(e) => Err(crate::common::api_request_unexpected_error::ApiRequestUnexpectedError::DeserializeBody { 
                                serde: e, 
                                status_code, 
                                headers, 
                                response_text 
                            }),
                        }, 
                        Err(e) => Err(crate::common::api_request_unexpected_error::ApiRequestUnexpectedError::FailedToGetResponseText { 
                            reqwest: e,
                            status_code,
                            headers,
                        }),
                    }
                } else {
                    match response.text().await {
                        Ok(response_text) => Err(crate::common::api_request_unexpected_error::ApiRequestUnexpectedError::StatusCode {
                            status_code,
                            headers,
                            response_text_result: crate::common::api_request_unexpected_error::ResponseTextResult::ResponseText(response_text)
                        }), 
                        Err(e) => Err(crate::common::api_request_unexpected_error::ApiRequestUnexpectedError::StatusCode {
                            status_code, 
                            headers, 
                            response_text_result: crate::common::api_request_unexpected_error::ResponseTextResult::ReqwestError(e),
                        }),
                    }
                }
            }
        }
    };
    let impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream_handle_token_stream = {
        quote::quote!{
            impl TryFrom<#ident_response_variants_token_stream> for #desirable_type_token_stream {
                type Error = #ident_with_serialize_deserialize_camel_case_token_stream;
                fn try_from(value: #ident_response_variants_token_stream) -> Result<Self, Self::Error> {
                    match value {
                        #ident_response_variants_token_stream::#desirable_token_stream(i) => Ok(i),
                        // #ident_response_variants_token_stream::Configuration {
                        //     configuration_box_dyn_error,
                        //     code_occurence,
                        // } => Err(KekwWithSerializeDeserialize::Configuration {
                        //     configuration_box_dyn_error,
                        //     code_occurence,
                        // }),
                    }
                }
            }
        }
    };
    let ident_request_error_logic_token_stream_handle_token_stream = {
        quote::quote!{
            #[derive(Debug, thiserror::Error, error_occurence::ErrorOccurence)]
            pub enum #ident_request_error_token_stream {
                ExpectedType {
                    #[eo_display_with_serialize_deserialize]
                    expected_type: #ident_with_serialize_deserialize_camel_case_token_stream,
                    #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                },
                UnexpectedStatusCode {
                    #[eo_display]
                    status_code: http::StatusCode,
                    #[eo_display_foreign_type]
                    headers: reqwest::header::HeaderMap,
                    #[eo_display_foreign_type]
                    response_text_result: crate::common::api_request_unexpected_error::ResponseTextResult,
                    #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                },
                FailedToGetResponseText {
                    #[eo_display_foreign_type]
                    reqwest: reqwest::Error,
                    #[eo_display]
                    status_code: http::StatusCode,
                    #[eo_display_foreign_type]
                    headers: reqwest::header::HeaderMap,
                    #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                },
                DeserializeResponse {
                    #[eo_display]
                    serde: serde_json::Error,
                    #[eo_display]
                    status_code: http::StatusCode,
                    #[eo_display_foreign_type]
                    headers: reqwest::header::HeaderMap,
                    #[eo_display_with_serialize_deserialize]
                    response_text: std::string::String,
                    #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                },
                Reqwest {
                    #[eo_display_foreign_type]
                    reqwest: reqwest::Error,
                    #code_occurence_lower_case_double_dot_space_crate_common_code_occurence_code_occurence_token_stream,
                },
            }
        }
    };
    let extraction_logic_token_stream_handle_token_stream = {
        let tvfrr_extraction_logic_lower_case_token_stream = proc_macro_helpers::type_variants_from_request_response::generate_tvfrr_extraction_logic_lower_case_token_stream(
            &ident_lower_case_stringified,
            &proc_macro_name_ident_stringified,
        );
        quote::quote!{
            async fn #tvfrr_extraction_logic_lower_case_token_stream<'a>(
                future: impl std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
            ) -> Result<
                #desirable_type_token_stream,
                #ident_request_error_token_stream,
            > {
                match future.await {
                    Ok(response) => match #try_from_response_lower_case_token_stream(response).await {
                        Ok(variants) => match std::vec::Vec::<crate::server::postgres::uuid_wrapper::PossibleUuidWrapper>::try_from(variants){
                            Ok(value) => Ok(value), 
                            Err(e) => Err(#ident_request_error_token_stream::ExpectedType {
                                expected_type: e, 
                                #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                            }),
                        },
                        Err(e) => match e {
                            crate::common::api_request_unexpected_error::ApiRequestUnexpectedError::StatusCode { 
                                status_code, 
                                headers, 
                                response_text_result, 
                            } => Err(#ident_request_error_token_stream::UnexpectedStatusCode {
                                status_code, 
                                headers, 
                                response_text_result, 
                                #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream
                            }),
                            crate::common::api_request_unexpected_error::ApiRequestUnexpectedError::FailedToGetResponseText { 
                                reqwest,
                                status_code,
                                headers 
                            } => Err(#ident_request_error_token_stream::FailedToGetResponseText {
                                reqwest,
                                status_code,
                                headers,
                                #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream
                            }),
                            crate::common::api_request_unexpected_error::ApiRequestUnexpectedError::DeserializeBody { 
                                serde,
                                status_code,
                                headers,
                                response_text,
                            } => Err(#ident_request_error_token_stream::DeserializeResponse {
                                serde,
                                status_code,
                                headers,
                                response_text,
                                #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream
                            }),
                        },
                    }, 
                    Err(e) => Err(#ident_request_error_token_stream::Reqwest {
                        reqwest: e,
                        #code_occurence_lower_case_crate_code_occurence_tufa_common_macro_call_token_stream,
                    }),
                }
            }
        }
    };
    let enum_status_codes_checker_name_logic_token_stream_handle_token_stream = {
        let enum_status_codes_checker_camel_case_token_stream = proc_macro_helpers::type_variants_from_request_response::generate_enum_status_codes_checker_camel_case_token_stream(
            &ident,
            proc_macro_name_ident_stringified,
        );
        quote::quote!{
            pub enum #enum_status_codes_checker_camel_case_token_stream {
                // ConfigurationTvfrr500InternalServerError,
            }
        }
    };
    let axum_response_into_response_logic_token_stream_handle_token_stream = {
        quote::quote!{
            impl axum::response::IntoResponse for #ident_response_variants_token_stream {
                fn into_response(self) -> axum::response::Response {
                    match &self {
                        #ident_response_variants_token_stream::#desirable_token_stream(_) => {
                            let mut res = axum::Json(self).into_response();
                            *res.status_mut() = #http_status_code_quote_token_stream;//http::StatusCode::CREATED
                            res
                        }
                        // #ident_response_variants_token_stream::Configuration {
                        //     configuration_box_dyn_error: _,
                        //     code_occurence: _,
                        // } => {
                        //     let mut res = axum::Json(self).into_response();
                        //     *res.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
                        //     res
                        // }
                    }
                }
            }
        }
    };
    quote::quote!{
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