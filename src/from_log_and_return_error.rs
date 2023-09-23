pub fn from_log_and_return_error(
    prepare_and_execute_query_error_token_stream: &proc_macro2::TokenStream,
    error_log_call_token_stream: &proc_macro2::TokenStream,
    prepare_and_execute_query_response_variants_token_stream: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote::quote! {
        let error = #prepare_and_execute_query_error_token_stream::from(e);
        #error_log_call_token_stream
        return #prepare_and_execute_query_response_variants_token_stream::from(error);
    }
}
