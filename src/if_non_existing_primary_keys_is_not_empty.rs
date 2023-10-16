pub fn if_non_existing_primary_keys_is_not_empty(
    non_existing_primary_keys_name_token_stream: &proc_macro2::TokenStream,
    expected_updated_primary_keys_name_token_stream: &proc_macro2::TokenStream,
    postgres_transaction_token_stream: &proc_macro2::TokenStream,
    rollback_token_stream: &proc_macro2::TokenStream,
    prepare_and_execute_query_error_token_stream: &proc_macro2::TokenStream,
    error_log_call_token_stream: &proc_macro2::TokenStream,
    response_variants_token_stream: &proc_macro2::TokenStream,
    non_existing_primary_keys_token_stream: &proc_macro2::TokenStream,
    non_existing_primary_keys_and_failed_rollback_token_stream: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote::quote! {
        let #non_existing_primary_keys_name_token_stream = {
            let mut #non_existing_primary_keys_name_token_stream =
                Vec::with_capacity(expected_updated_primary_keys.len());
            for element in #expected_updated_primary_keys_name_token_stream {
                if let false = primary_key_vec.contains(&element) {
                    #non_existing_primary_keys_name_token_stream.push(element);
                }
            }
            #non_existing_primary_keys_name_token_stream
        };
        if let false = #non_existing_primary_keys_name_token_stream.is_empty() {
            match #postgres_transaction_token_stream.#rollback_token_stream().await {
                Ok(_) => {
                    let error = #prepare_and_execute_query_error_token_stream::#non_existing_primary_keys_token_stream;
                    #error_log_call_token_stream
                    return #response_variants_token_stream::from(error);
                }
                Err(e) => {
                    let error = #prepare_and_execute_query_error_token_stream::#non_existing_primary_keys_and_failed_rollback_token_stream;
                    #error_log_call_token_stream
                    return #response_variants_token_stream::from(error);
                }
            }
        }
    }
}
