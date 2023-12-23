#[allow(clippy::too_many_arguments)]
pub fn generate_postgres_execute_query(
    query_string_name_token_stream: &proc_macro2::TokenStream,
    query_string_token_stream: &proc_macro2::TokenStream,
    binded_query_name_token_stream: &proc_macro2::TokenStream,
    binded_query_token_stream: &proc_macro2::TokenStream,
    acquire_pool_and_connection_token_stream: &proc_macro2::TokenStream,
    pg_connection_token_stream: &proc_macro2::TokenStream,
    response_variants_token_stream: &proc_macro2::TokenStream,
    desirable_token_stream: &proc_macro2::TokenStream,
    from_log_and_return_error_token_stream: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote::quote! {
        let #query_string_name_token_stream = {
            #query_string_token_stream
        };
        println!("{}", #query_string_name_token_stream);
        let #binded_query_name_token_stream = {
            #binded_query_token_stream
        };
        #acquire_pool_and_connection_token_stream
        match #binded_query_name_token_stream.execute(#pg_connection_token_stream.as_mut()).await {
            //todo - is need to return rows affected?
            Ok(_) => #response_variants_token_stream::#desirable_token_stream(vec![]),//todo desirable value
            Err(e) => {
                 #from_log_and_return_error_token_stream
            }
        }
    }
}
