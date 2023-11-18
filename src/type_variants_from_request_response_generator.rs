fn type_variants_from_request_response_generator(
    desirable_attribute: proc_macro_helpers::attribute::Attribute,
    ident: &syn::Ident,
    ident_response_variants_token_stream: &proc_macro2::TokenStream,//KekwResponseVariants
    desirable_token_stream: &proc_macro2::TokenStream,
    desirable_type_token_stream: &proc_macro2::TokenStream,//std::vec::Vec<crate::server::postgres::uuid_wrapper::PossibleUuidWrapper>
    //
    type_variants_from_request_response: std::vec::Vec<impl crate::type_variants_from_request_response::TypeVariantsFromRequestResponse>,
    // attribute: proc_macro_helpers::attribute::Attribute,
    // enum_with_serialize_deserialize_logic_token_stream: std::vec::Vec<proc_macro2::TokenStream>,
    // from_logic_token_stream: std::vec::Vec<proc_macro2::TokenStream>,
    // impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream: std::vec::Vec<proc_macro2::TokenStream>,
    // generated_status_code_enums_with_from_impls_logic_token_stream: proc_macro2::TokenStream,
    // try_from_response_logic_token_stream: proc_macro2::TokenStream,
    // impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream: proc_macro2::TokenStream,
    // ident_request_error_logic_token_stream: proc_macro2::TokenStream,
    // extraction_logic_token_stream: proc_macro2::TokenStream,
    // enum_status_codes_checker_name_logic_token_stream: proc_macro2::TokenStream,
    // axum_response_into_response_logic_token_stream: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let http_status_code_quote = desirable_attribute.to_http_status_code_quote();
    use crate::type_variants_from_request_response::TypeVariantsFromRequestResponse;
    let type_variants_from_request_response_len = type_variants_from_request_response.len();
    let (
        attributes,
        enum_with_serialize_deserialize_logic_token_stream,
        from_logic_token_stream,
        impl_std_convert_from_ident_response_variants_token_stream_for_http_status_code_logic_token_stream,
        generated_status_code_enums_with_from_impls_logic_token_stream,
        try_from_response_logic_token_stream,
        impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream,
        ident_request_error_logic_token_stream,
        extraction_logic_token_stream,
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
        acc.7.push(element.ident_request_error_logic_token_stream());
        acc.8.push(element.extraction_logic_token_stream());
        acc.9.push(element.enum_status_codes_checker_name_logic_token_stream());
        acc.10.push(element.axum_response_into_response_logic_token_stream());
        acc
    });
    let enum_with_serialize_deserialize_logic_token_stream_handle_token_stream = {
        quote::quote!{
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            pub enum #ident_response_variants_token_stream {
                #desirable_token_stream(#desirable_type_token_stream),
                // Configuration {
                //     configuration_box_dyn_error: std::string::String,
                //     code_occurence: crate::common::code_occurence::CodeOccurence,
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
                        #ident_response_variants_token_stream::#desirable_token_stream(_) => #http_status_code_quote,//http::StatusCode::CREATED
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
            // #[derive(Debug, serde::Serialize, serde::Deserialize)]
            // enum KekwResponseVariantsTvfrr201Created {
            //     #desirable_token_stream(#desirable_type_token_stream),
            // }
            // impl std::convert::From<KekwResponseVariantsTvfrr201Created> for KekwResponseVariants {
            //     fn from(value: KekwResponseVariantsTvfrr201Created) -> Self {
            //         match value {
            //             KekwResponseVariantsTvfrr201Created::#desirable_token_stream(i) => Self::#desirable_token_stream(i),
            //         }
            //     }   
            // }
            // #[derive(Debug, serde::Serialize, serde::Deserialize)]
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
        }
    };
    let try_from_response_logic_token_stream_handle_token_stream = {
        quote::quote!{

        }
    };
    let impl_try_from_ident_response_variants_token_stream_for_desirable_logic_token_stream_handle_token_stream = {
        quote::quote!{

        }
    };
    let ident_request_error_logic_token_stream_handle_token_stream = {
        quote::quote!{

        }
    };
    let extraction_logic_token_stream_handle_token_stream = {
        quote::quote!{

        }
    };
    let enum_status_codes_checker_name_logic_token_stream_handle_token_stream = {
        quote::quote!{

        }
    };
    let axum_response_into_response_logic_token_stream_handle_token_stream = {
        quote::quote!{

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