trait TypeVariantsFromRequestResponse {
    fn enum_with_serialize_deserialize_logic_token_stream() -> proc_macro2::TokenStream;
    fn from_logic_token_stream() -> proc_macro2::TokenStream;
    fn impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream() -> proc_macro2::TokenStream;
    fn generated_status_code_enums_with_from_impls_logic_token_stream() -> proc_macro2::TokenStream;
    fn try_from_response_logic_token_stream() -> proc_macro2::TokenStream;
    fn impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream() -> proc_macro2::TokenStream;
    fn ident_request_error_logic_token_stream() -> proc_macro2::TokenStream;
    fn extraction_logic_token_stream() -> proc_macro2::TokenStream;
    fn enum_status_codes_checker_name_logic_token_stream() -> proc_macro2::TokenStream;
    fn axum_response_into_response_logic_token_stream() -> proc_macro2::TokenStream;
}

struct Configuration{}

impl TypeVariantsFromRequestResponse for Configuration {
    fn enum_with_serialize_deserialize_logic_token_stream() -> proc_macro2::TokenStream {
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
    fn from_logic_token_stream() -> proc_macro2::TokenStream {
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
    fn impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream() -> proc_macro2::TokenStream {
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
    fn generated_status_code_enums_with_from_impls_logic_token_stream() -> proc_macro2::TokenStream {
        quote::quote!{
//
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
//
        }
    }
    fn try_from_response_logic_token_stream() -> proc_macro2::TokenStream {
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
    fn impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream() -> proc_macro2::TokenStream {
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
    fn ident_request_error_logic_token_stream() -> proc_macro2::TokenStream {
        quote::quote!{
            #[derive(Debug, thiserror :: Error, error_occurence :: ErrorOccurence)]
            pub enum KekwRequestError {
                ExpectedType {
                    #[eo_display_with_serialize_deserialize]
                    expected_type: KekwWithSerializeDeserialize,
                    code_occurence: crate::common::code_occurence::CodeOccurence,
                },
                UnexpectedStatusCode {
                    #[eo_display]
                    status_code: http::StatusCode,
                    #[eo_display_foreign_type]
                    headers: reqwest::header::HeaderMap,
                    #[eo_display_foreign_type]
                    response_text_result: crate::common::api_request_unexpected_error::ResponseTextResult,
                    code_occurence: crate::common::code_occurence::CodeOccurence,
                },
                FailedToGetResponseText {
                    #[eo_display_foreign_type]
                    reqwest: reqwest::Error,
                    #[eo_display]
                    status_code: http::StatusCode,
                    #[eo_display_foreign_type]
                    headers: reqwest::header::HeaderMap,
                    code_occurence: crate::common::code_occurence::CodeOccurence,
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
                    code_occurence: crate::common::code_occurence::CodeOccurence,
                },
                Reqwest {
                    #[eo_display_foreign_type]
                    reqwest: reqwest::Error,
                    code_occurence: crate::common::code_occurence::CodeOccurence,
                },
            }
        }
    }
    fn extraction_logic_token_stream() -> proc_macro2::TokenStream {
        quote::quote!{
            async fn tvfrr_extraction_logic_kekw<'a>(
                future: impl std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
            ) -> Result<
                std::vec::Vec<crate::server::postgres::uuid_wrapper::PossibleUuidWrapper>,
                KekwRequestError,
            > {
                match future.await
                {
                    Ok(response) => match try_from_response_kekw(response).await
                    {
                        Ok(variants) => match std :: vec :: Vec :: < crate :: server ::
                        postgres :: uuid_wrapper :: PossibleUuidWrapper > ::
                        try_from(variants)
                        {
                            Ok(value) => Ok(value), Err(e) =>
                            Err(KekwRequestError :: ExpectedType
                            {
                                expected_type : e, code_occurence : crate ::
                                code_occurence_tufa_common! (),
                            }),
                        }, Err(e) => match e
                        {
                            crate :: common :: api_request_unexpected_error ::
                            ApiRequestUnexpectedError :: StatusCode
                            { status_code, headers, response_text_result, } =>
                            Err(KekwRequestError :: UnexpectedStatusCode
                            {
                                status_code, headers, response_text_result, code_occurence :
                                crate :: code_occurence_tufa_common! ()
                            }), crate :: common :: api_request_unexpected_error ::
                            ApiRequestUnexpectedError :: FailedToGetResponseText
                            { reqwest, status_code, headers } =>
                            Err(KekwRequestError :: FailedToGetResponseText
                            {
                                reqwest, status_code, headers, code_occurence : crate ::
                                code_occurence_tufa_common! ()
                            }), crate :: common :: api_request_unexpected_error ::
                            ApiRequestUnexpectedError :: DeserializeBody
                            { serde, status_code, headers, response_text, } =>
                            Err(KekwRequestError :: DeserializeResponse
                            {
                                serde, status_code, headers, response_text, code_occurence :
                                crate :: code_occurence_tufa_common! ()
                            }),
                        },
                    }, Err(e) =>
                    Err(KekwRequestError :: Reqwest
                    {
                        reqwest : e, code_occurence : crate :: code_occurence_tufa_common!
                        (),
                    }),
                }
            }
        }
    }
    fn enum_status_codes_checker_name_logic_token_stream() -> proc_macro2::TokenStream {
        quote::quote!{
            pub enum KekwStatusCodesChecker {
                ConfigurationTvfrr500InternalServerError,
            }
        }
    }
    fn axum_response_into_response_logic_token_stream() -> proc_macro2::TokenStream {
        quote::quote!{
            impl axum::response::IntoResponse for KekwResponseVariants {
                fn into_response(self) -> axum::response::Response {
                    match &self {
                        KekwResponseVariants::Desirable(_) => {
                            let mut res = axum::Json(self).into_response();
                            *res.status_mut() = http::StatusCode::CREATED;
                            res
                        }
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
        }
    }
}