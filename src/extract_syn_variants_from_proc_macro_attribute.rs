pub fn extract_syn_variants_from_proc_macro_attribute(
    ast: &syn::DeriveInput,
    proc_macro_attribute_name: &str,
    proc_macro_name_lower_case: &str,
    proc_macro_name_camel_case_ident_stringified: &std::string::String
) -> std::vec::Vec<syn::Variant> {
    let additional_http_status_codes_error_variant_path = format!("{proc_macro_name_lower_case}::{proc_macro_attribute_name}");
    let additional_http_status_codes_error_variants_attribute = proc_macro_helpers::get_macro_attribute::get_macro_attribute(
        &ast.attrs,
        &additional_http_status_codes_error_variant_path,
        proc_macro_name_camel_case_ident_stringified
    );
    match additional_http_status_codes_error_variants_attribute.path.segments.len() == 2 {
        true => {
            let first_ident = &additional_http_status_codes_error_variants_attribute.path.segments.first().unwrap_or_else(|| {
                panic!("{proc_macro_name_camel_case_ident_stringified} {additional_http_status_codes_error_variant_path} additional_http_status_codes_error_variants_attribute.path.segments.get(0) is None")
            }).ident;
            let second_ident = &additional_http_status_codes_error_variants_attribute.path.segments.last().unwrap_or_else(|| {
                panic!("{proc_macro_name_camel_case_ident_stringified} {additional_http_status_codes_error_variant_path} additional_http_status_codes_error_variants_attribute.path.segments.get(0) is None")
            }).ident;
            let possible_additional_http_status_codes_error_variants_attribute_path = format!("{first_ident}::{second_ident}");
            if let false = additional_http_status_codes_error_variant_path == possible_additional_http_status_codes_error_variants_attribute_path {
                panic!("{proc_macro_name_camel_case_ident_stringified} {additional_http_status_codes_error_variant_path} {possible_additional_http_status_codes_error_variants_attribute_path} is not {additional_http_status_codes_error_variant_path}")
            }
        },
        false => panic!("{proc_macro_name_camel_case_ident_stringified} {additional_http_status_codes_error_variant_path} no {additional_http_status_codes_error_variant_path} path")
    }
    let mut additional_http_status_codes_error_variants_attribute_tokens_stringified = additional_http_status_codes_error_variants_attribute.tokens.to_string();
    let additional_http_status_codes_error_variants_attribute_tokens_stringified_len = additional_http_status_codes_error_variants_attribute_tokens_stringified.len();
    let additional_http_status_codes_error_variants_attribute_tokens_without_brackets_stringified = &additional_http_status_codes_error_variants_attribute_tokens_stringified[1..(additional_http_status_codes_error_variants_attribute_tokens_stringified_len - 1)];//todo maybe check
    additional_http_status_codes_error_variants_attribute_tokens_without_brackets_stringified.split(";").collect::<Vec<&str>>()
        .iter().fold(std::vec::Vec::<syn::Variant>::new(), |mut acc, element| {
            let element_derive_input: syn::DeriveInput = syn::parse(
                element.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_camel_case_ident_stringified} {additional_http_status_codes_error_variant_path} {element} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE))
                .into()
            ).unwrap_or_else(|e| {
                panic!("{proc_macro_name_camel_case_ident_stringified} {additional_http_status_codes_error_variant_path} parse additional_http_status_codes_error_variants_attribute_tokens failed {e}");
            });
            // let element_ident = element_derive_input.ident;//todo check if error type even exists (with empty functions)
            let data_enum = if let syn::Data::Enum(data_enum) = element_derive_input.data {
        data_enum
            } else {
                panic!("{proc_macro_name_camel_case_ident_stringified} {additional_http_status_codes_error_variant_path} does not work on enums!");
            };
            for element in data_enum.variants {
                acc.push(element);
            }
            acc
        })
}