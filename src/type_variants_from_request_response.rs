pub trait TypeVariantsFromRequestResponse {
    fn attribute(&self) -> proc_macro_helpers::attribute::Attribute;
    //
    fn enum_with_serialize_deserialize_logic_token_stream(&self) -> proc_macro2::TokenStream;
    fn from_logic_token_stream(&self) -> proc_macro2::TokenStream;
    fn impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream(
        &self,
    ) -> proc_macro2::TokenStream;
    fn try_from_response_logic_token_stream(&self) -> proc_macro2::TokenStream;
    fn impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream(
        &self,
    ) -> proc_macro2::TokenStream;
    fn enum_status_codes_checker_name_logic_token_stream(&self) -> proc_macro2::TokenStream;
    fn axum_response_into_response_logic_token_stream(&self) -> proc_macro2::TokenStream;
}

struct Configuration {}

impl TypeVariantsFromRequestResponse for Configuration {
    fn attribute(&self) -> proc_macro_helpers::attribute::Attribute {
        proc_macro_helpers::attribute::Attribute::Tvfrr500InternalServerError
    }
    //
    fn enum_with_serialize_deserialize_logic_token_stream(&self) -> proc_macro2::TokenStream {
        quote::quote! {
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
        quote::quote! {
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
    fn impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream(
        &self,
    ) -> proc_macro2::TokenStream {
        quote::quote! {
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
    fn try_from_response_logic_token_stream(&self) -> proc_macro2::TokenStream {
        quote::quote! {
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
    fn impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream(
        &self,
    ) -> proc_macro2::TokenStream {
        quote::quote! {
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
        quote::quote! {
            pub enum KekwStatusCodesChecker {
                ConfigurationTvfrr500InternalServerError,
            }
        }
    }
    fn axum_response_into_response_logic_token_stream(&self) -> proc_macro2::TokenStream {
        quote::quote! {
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
    error_variant: ErrorVariant
}

pub struct ErrorVariant {
    error_variant_ident: proc_macro2::TokenStream,
    error_variant_fields: std::vec::Vec<ErrorVariantField>,
}

fn generate_status_code_enums_with_from_impls_logic_token_stream(
    derive_debug_serialize_deserialize_token_stream: &proc_macro2::TokenStream,//#[derive(Debug, serde::Serialize, serde::Deserialize)]
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
