pub fn postgres_transaction_commit_match(
    primary_key_try_from_sqlx_row_name_token_stream: &proc_macro2::TokenStream,
    from_log_and_return_error_token_stream: &proc_macro2::TokenStream,
    rollback_error_name_token_stream: &proc_macro2::TokenStream,
    primary_key_from_row_and_failed_rollback_token_stream: &proc_macro2::TokenStream,
    //
    non_existing_primary_keys_name_token_stream: &proc_macro2::TokenStream,
    expected_updated_primary_keys_name_token_stream: &proc_macro2::TokenStream,
    primary_key_vec_name_token_stream: &proc_macro2::TokenStream,
    rollback_token_stream: &proc_macro2::TokenStream,
    non_existing_primary_keys_token_stream: &proc_macro2::TokenStream,
    non_existing_primary_keys_and_failed_rollback_token_stream: &proc_macro2::TokenStream,
    postgres_transaction_token_stream: &proc_macro2::TokenStream,
    commit_token_stream: &proc_macro2::TokenStream,
    response_variants_token_stream: &proc_macro2::TokenStream,
    desirable_token_stream: &proc_macro2::TokenStream,
    prepare_and_execute_query_error_token_stream: &proc_macro2::TokenStream,
    commit_failed_token_stream: &proc_macro2::TokenStream,
    error_log_call_token_stream: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote::quote! {
        let #primary_key_vec_name_token_stream = {
            let mut #primary_key_vec_name_token_stream = Vec::with_capacity(#expected_updated_primary_keys_name_token_stream.len());
            for element in results_vec {
                match #primary_key_try_from_sqlx_row_name_token_stream(&element) {
                    Ok(primary_key) => {
                        #primary_key_vec_name_token_stream.push(primary_key);
                    }
                    Err(e) => match #postgres_transaction_token_stream.#rollback_token_stream().await {
                        Ok(_) => {
                            #from_log_and_return_error_token_stream;
                        }
                        Err(#rollback_error_name_token_stream) => {
                            let error = #prepare_and_execute_query_error_token_stream::#primary_key_from_row_and_failed_rollback_token_stream;
                            #error_log_call_token_stream
                            return #response_variants_token_stream::from(error);
                        }
                    },
                }
            }
            #primary_key_vec_name_token_stream
        };
        {
            let #non_existing_primary_keys_name_token_stream = {
                let len = #expected_updated_primary_keys_name_token_stream.len();
                #expected_updated_primary_keys_name_token_stream.into_iter().fold(Vec::with_capacity(len), |mut acc, element| {
                    if let false = #primary_key_vec_name_token_stream.contains(&element) {
                        acc.push(element);
                    }
                    acc
                })
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
