pub trait TypeVariantsFromRequestResponse {
    fn attribute(&self) -> proc_macro_helpers::attribute::Attribute;
    //
    fn enum_with_serialize_deserialize_logic_token_stream(&self) -> proc_macro2::TokenStream;
    fn from_logic_token_stream(&self) -> proc_macro2::TokenStream;
    fn impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream(&self) -> proc_macro2::TokenStream;
    fn generated_status_code_enums_with_from_impls_logic_token_stream(&self) -> proc_macro2::TokenStream;
    fn try_from_response_logic_token_stream(&self) -> proc_macro2::TokenStream;
    fn impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream(&self) -> proc_macro2::TokenStream;
    fn enum_status_codes_checker_name_logic_token_stream(&self) -> proc_macro2::TokenStream;
    fn axum_response_into_response_logic_token_stream(&self) -> proc_macro2::TokenStream;
}

struct Configuration{}

impl TypeVariantsFromRequestResponse for Configuration {
    fn attribute(&self) -> proc_macro_helpers::attribute::Attribute {
        proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError
    }
    //
    fn enum_with_serialize_deserialize_logic_token_stream(&self) -> proc_macro2::TokenStream {
        quote::quote!{
            #[derive(Debug, serde :: Serialize, serde :: Deserialize)]
            pub enum KekwResponseVariants {
                Desirable(std::vec::Vec<crate::server::postgres::uuid_wrapper::PossibleUuidWrapper>),
                Configuration {
                    configuration_box_dyn_error: std::string::String,
                    code_occurence: crate::common::code_occurence::CodeOccurence,
                },
            }
        }
    }
    fn from_logic_token_stream(&self) -> proc_macro2::TokenStream {
        quote::quote!{
            impl std::convert::From<Kekw> for KekwResponseVariants {
                fn from(val: Kekw) -> Self {
                    match val.into_serialize_deserialize_version() {
                        KekwWithSerializeDeserialize::Configuration {
                            configuration_box_dyn_error,
                            code_occurence,
                        } => Self::Configuration {
                            configuration_box_dyn_error,
                            code_occurence,
                        },
                    }
                }
            }
        }
    }
    fn impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream(&self) -> proc_macro2::TokenStream {
        quote::quote!{
            impl std::convert::From<&KekwResponseVariants> for http::StatusCode {
                fn from(value: &KekwResponseVariants) -> Self {
                    match value {
                        KekwResponseVariants::Desirable(_) => http::StatusCode::CREATED,
                        KekwResponseVariants::Configuration {
                            configuration_box_dyn_error: _,
                            code_occurence: _,
                        } => http::StatusCode::INTERNAL_SERVER_ERROR,
                    }
                }
            }
        }
    }
    fn generated_status_code_enums_with_from_impls_logic_token_stream(&self) -> proc_macro2::TokenStream {
        quote::quote!{
            #[derive(Debug, serde :: Serialize, serde :: Deserialize)]
            enum KekwResponseVariantsTvfrr201Created {
                Desirable(std::vec::Vec<crate::server::postgres::uuid_wrapper::PossibleUuidWrapper>),
            }
            impl std::convert::From<KekwResponseVariantsTvfrr201Created> for KekwResponseVariants {
                fn from(value: KekwResponseVariantsTvfrr201Created) -> Self {
                    match value {
                        KekwResponseVariantsTvfrr201Created::Desirable(i) => Self::Desirable(i),
                    }
                }   
            }
            #[derive(Debug, serde :: Serialize, serde :: Deserialize)]
            enum KekwResponseVariantsTvfrr500InternalServerError {
                Configuration {
                    configuration_box_dyn_error: std::string::String,
                    code_occurence: crate::common::code_occurence::CodeOccurence,
                },
            }
            impl std::convert::From<KekwResponseVariantsTvfrr500InternalServerError> for KekwResponseVariants {
                fn from(value: KekwResponseVariantsTvfrr500InternalServerError) -> Self {
                    match value {
                        KekwResponseVariantsTvfrr500InternalServerError::Configuration {
                            configuration_box_dyn_error,
                            code_occurence,
                        } => Self::Configuration {
                            configuration_box_dyn_error,
                            code_occurence,
                        },
                    }
                }
            }
        }
    }
    fn try_from_response_logic_token_stream(&self) -> proc_macro2::TokenStream {
        quote::quote!{
            async fn try_from_response_kekw(
                response: reqwest::Response,
            ) -> Result<
                KekwResponseVariants,
                crate::common::api_request_unexpected_error::ApiRequestUnexpectedError,
            > {
                let status_code = response.status();
                let headers = response.headers().clone();
                if status_code == http::StatusCode::CREATED {
                    match response.text().await {
                        Ok(response_text) => match serde_json :: from_str :: <
                        KekwResponseVariantsTvfrr201Created > (& response_text)
                        {
                            Ok(value) => Ok(KekwResponseVariants :: from(value)), Err(e)
                            =>
                            Err(crate :: common :: api_request_unexpected_error ::
                            ApiRequestUnexpectedError :: DeserializeBody
                            { serde : e, status_code, headers, response_text }),
                        }, Err(e) =>
                        Err(crate :: common :: api_request_unexpected_error ::
                        ApiRequestUnexpectedError :: FailedToGetResponseText
                        { reqwest : e, status_code, headers, }),
                    }
                } else {
                    match response.text().await {
                        Ok(response_text) =>
                        Err(crate :: common :: api_request_unexpected_error ::
                        ApiRequestUnexpectedError :: StatusCode
                        {
                            status_code, headers, response_text_result : crate :: common
                            :: api_request_unexpected_error :: ResponseTextResult ::
                            ResponseText(response_text)
                        },), Err(e) =>
                        Err(crate :: common :: api_request_unexpected_error ::
                        ApiRequestUnexpectedError :: StatusCode
                        {
                            status_code, headers, response_text_result : crate :: common
                            :: api_request_unexpected_error :: ResponseTextResult ::
                            ReqwestError(e),
                        },),
                    }
                }
            }
        }
    }
    fn impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream(&self) -> proc_macro2::TokenStream {
        quote::quote!{
            impl TryFrom<KekwResponseVariants> for std::vec::Vec<crate::server::postgres::uuid_wrapper::PossibleUuidWrapper> {
                type Error = KekwWithSerializeDeserialize;
                fn try_from(value: KekwResponseVariants) -> Result<Self, Self::Error> {
                    match value {
                        KekwResponseVariants::Desirable(i) => Ok(i),
                        KekwResponseVariants::Configuration {
                            configuration_box_dyn_error,
                            code_occurence,
                        } => Err(KekwWithSerializeDeserialize::Configuration {
                            configuration_box_dyn_error,
                            code_occurence,
                        }),
                    }
                }
            }
        }
    }
    fn enum_status_codes_checker_name_logic_token_stream(&self) -> proc_macro2::TokenStream {
        quote::quote!{
            pub enum KekwStatusCodesChecker {
                ConfigurationTvfrr500InternalServerError,
            }
        }
    }
    fn axum_response_into_response_logic_token_stream(&self) -> proc_macro2::TokenStream {
        quote::quote!{
            // impl axum::response::IntoResponse for KekwResponseVariants {
            //     fn into_response(self) -> axum::response::Response {
            //         match &self {
            //             KekwResponseVariants::Desirable(_) => {
            //                 let mut res = axum::Json(self).into_response();
            //                 *res.status_mut() = http::StatusCode::CREATED;
            //                 res
            //             }

            //         }
            //     }
            // }
            KekwResponseVariants::Configuration {
                configuration_box_dyn_error: _,
                code_occurence: _,
            } => {
                let mut res = axum::Json(self).into_response();
                *res.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
                res
            }
        }
    }
}

fn type_variants_from_request_response(
    attribute: proc_macro_helpers::attribute::Attribute
) -> (
    proc_macro_helpers::attribute::Attribute,//attribute
    std::vec::Vec::<proc_macro2::TokenStream>,//enum_with_serialize_deserialize_logic_token_stream
    std::vec::Vec::<proc_macro2::TokenStream>,//from_logic_token_stream
    std::vec::Vec::<proc_macro2::TokenStream>,//impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream
    std::vec::Vec::<proc_macro2::TokenStream>,//generated_status_code_enums_with_from_impls_logic_token_stream
    std::vec::Vec::<proc_macro2::TokenStream>,//try_from_response_logic_token_stream
    std::vec::Vec::<proc_macro2::TokenStream>,//impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream
    std::vec::Vec::<proc_macro2::TokenStream>,//enum_status_codes_checker_name_logic_token_stream
    std::vec::Vec::<proc_macro2::TokenStream>,//axum_response_into_response_logic_token_stream
) {
    // fn attribute(&self) -> proc_macro_helpers::attribute::Attribute;
    //
    let enum_with_serialize_deserialize_logic_token_stream = {
        vec![quote::quote!{
            Configuration {
                configuration_box_dyn_error: std::string::String,
                code_occurence: crate::common::code_occurence::CodeOccurence,
            }
        }]
    };
    let from_logic_token_stream = {
        vec![quote::quote!{
            KekwWithSerializeDeserialize::Configuration {
                configuration_box_dyn_error,
                code_occurence,
            } => Self::Configuration {
                configuration_box_dyn_error,
                code_occurence,
            }
        }]
    };
    let impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream = {
        vec![quote::quote!{
            KekwResponseVariants::Configuration {
                configuration_box_dyn_error: _,
                code_occurence: _,
            } => http::StatusCode::INTERNAL_SERVER_ERROR
        }]
    };
    let generated_status_code_enums_with_from_impls_logic_token_stream = {
        vec![quote::quote!{
            // #[derive(Debug, serde :: Serialize, serde :: Deserialize)]
            // enum KekwResponseVariantsTvfrr500InternalServerError {
            //     Configuration {
            //         configuration_box_dyn_error: std::string::String,
            //         code_occurence: crate::common::code_occurence::CodeOccurence,
            //     },
            // }
            // impl std::convert::From<KekwResponseVariantsTvfrr500InternalServerError> for KekwResponseVariants {
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
        }]
    };
    let try_from_response_logic_token_stream = {
        //todo else if for each different status code
        vec![quote::quote!{
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
        vec![quote::quote!{
                // #ident_response_variants_token_stream::Configuration {
                //     configuration_box_dyn_error,
                //     code_occurence,
                // } => Err(KekwWithSerializeDeserialize::Configuration {
                //     configuration_box_dyn_error,
                //     code_occurence,
                // })
        }]
    };
    let enum_status_codes_checker_name_logic_token_stream = {
        vec![quote::quote!{
            ConfigurationTvfrr500InternalServerError,
        }]
    };
    let axum_response_into_response_logic_token_stream = {
        vec![quote::quote!{
            // #ident_response_variants_token_stream::Configuration {
            //     configuration_box_dyn_error: _,
            //     code_occurence: _,
            // } => {
            //     let mut res = axum::Json(self).into_response();
            //     *res.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
            //     res
            // }
        }]
    };
    (
        attribute,
        //
        enum_with_serialize_deserialize_logic_token_stream,
        from_logic_token_stream,
        impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream,
        generated_status_code_enums_with_from_impls_logic_token_stream,
        try_from_response_logic_token_stream,
        impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream,
        enum_status_codes_checker_name_logic_token_stream,
        axum_response_into_response_logic_token_stream,
    )
}