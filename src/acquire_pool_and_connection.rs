pub fn acquire_pool_and_connection(
    prepare_and_execute_query_error_token_stream: &proc_macro2::TokenStream,
    error_log_call_token_stream: &proc_macro2::TokenStream,
    prepare_and_execute_query_response_variants_token_stream: &proc_macro2::TokenStream,
    pg_connection_token_stream: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote::quote! {
        let mut pool_connection = match app_info_state.get_postgres_pool().acquire().await {//todo find out difference between acquire and try_acquire
            Ok(value) => value,
            Err(e) => {
                let error = #prepare_and_execute_query_error_token_stream::from(e);
                #error_log_call_token_stream
                return #prepare_and_execute_query_response_variants_token_stream::from(error);
            }
        };
        let #pg_connection_token_stream = match sqlx::Acquire::acquire(&mut pool_connection).await {
            Ok(value) => value,
            Err(e) => {
                let error = #prepare_and_execute_query_error_token_stream::from(e);
                #error_log_call_token_stream
                return #prepare_and_execute_query_response_variants_token_stream::from(error);
            }
        };
    }
}
