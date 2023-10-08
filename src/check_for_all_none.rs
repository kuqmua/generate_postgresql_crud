pub enum QueryPart {
    Payload,
    QueryParameters
} 

impl std::fmt::Display for QueryPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Payload => write!(f, "payload"),
            Self::QueryParameters => write!(f, "query")
        }
    }
}

impl QueryPart {
    pub fn get_response_variant(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Payload => quote::quote!{
                NoPayloadFields {
                    no_payload_fields: std::string::String::from("no payload fields"), 
                    code_occurence: crate::code_occurence_tufa_common!()
                }
            },
            Self::QueryParameters => quote::quote!{
                NoQueryParameters { 
                    no_query_parameters: std::string::String::from("no query parameters"), 
                    code_occurence: crate::code_occurence_tufa_common!(),
                }
            }
        }
    }
}

pub fn check_for_all_none(
    fields_named: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    id_field: &syn::Field,
    proc_macro_name_ident_stringified: &std::string::String,
    dot_space: &str,
    prepare_and_execute_query_response_variants_token_stream: &proc_macro2::TokenStream,
    query_part: QueryPart
) -> proc_macro2::TokenStream {
    let (none_elements, match_elements) = {
        let fields_named_filtered = fields_named.iter().filter(|field|*field != id_field).collect::<Vec<&syn::Field>>();
        let fields_named_len = fields_named_filtered.len();
        fields_named_filtered.iter().enumerate().fold(
            (
                std::string::String::default(),
                std::string::String::default()
            ), |mut acc, (index, field)| {
                let field_ident = field.ident.clone()
                    .unwrap_or_else(|| {
                        panic!("{proc_macro_name_ident_stringified} field.ident is None")
                    });
                let possible_dot_space = match (index + 1) == fields_named_len {
                    true => "",
                    false => dot_space,
                };
                acc.0.push_str(&format!("None{possible_dot_space}"));
                acc.1.push_str(&format!("&self.{query_part}.{field_ident}{possible_dot_space}"));
                acc
            })
    };
    let none_elements_token_stream = none_elements.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {none_elements} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let match_elements_token_stream = match_elements.parse::<proc_macro2::TokenStream>()
    .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {match_elements} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let response_variant_token_stream = query_part.get_response_variant();
    quote::quote!{
        if let (#none_elements_token_stream) = (#match_elements_token_stream) {
            return #prepare_and_execute_query_response_variants_token_stream::#response_variant_token_stream;
        }
    }
}