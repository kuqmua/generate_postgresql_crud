pub fn postgres_transaction_commit_match(
    postgres_transaction_token_stream: &proc_macro2::TokenStream,
    commit_token_stream: &proc_macro2::TokenStream,
    response_variants_token_stream: &proc_macro2::TokenStream,
    desirable_token_stream: &proc_macro2::TokenStream,
    prepare_and_execute_query_error_token_stream: &proc_macro2::TokenStream,
    commit_failed_token_stream: &proc_macro2::TokenStream,
    error_log_call_token_stream: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote::quote! {
        match #postgres_transaction_token_stream.#commit_token_stream().await {
            Ok(_) => #response_variants_token_stream::#desirable_token_stream(()),
            Err(e) => {
                let error = #prepare_and_execute_query_error_token_stream::#commit_failed_token_stream;
                #error_log_call_token_stream
                #response_variants_token_stream::from(error)
            }
        }
    }
}
