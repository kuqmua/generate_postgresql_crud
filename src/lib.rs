use std::collections::HashMap;

use convert_case::Casing;

#[proc_macro_derive(GeneratePostgresqlCrud)]
pub fn generate_postgresql_crud(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro_helpers::panic_location::panic_location();
    let proc_macro_name = "GeneratePostgresqlCrud";
    let ast: syn::DeriveInput = syn::parse(input).unwrap_or_else(|_| {
        panic!(
            "{proc_macro_name} {}",
            proc_macro_helpers::global_variables::hardcode::AST_PARSE_FAILED
        )
    });
    let ident = &ast.ident;
    let proc_macro_name_ident_stringified = format!("{proc_macro_name} {ident}");
    let data_struct = if let syn::Data::Struct(data_struct) = ast.data {
        // println!("{data_struct:#?}");
        data_struct
    } else {
        panic!("does not work on structs!");
    };
    let fields_named = if let syn::Fields::Named(fields_named) = data_struct.fields {
        fields_named.named
    } else {
        panic!("{proc_macro_name_ident_stringified} supports only syn::Fields::Named");
    };
    let struct_options_ident_stringified = format!("{ident}Options");
    let struct_options_ident_token_stream = 
    struct_options_ident_stringified.parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {struct_options_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
    let fields_options = fields_named.iter().map(|field| {
        let field_vis = &field.vis;
        let field_ident = &field.ident;
        let field_type_path = &field.ty;
        quote::quote! {
            #[serde(skip_serializing_if = "Option::is_none")]
            #field_vis #field_ident: Option<#field_type_path>
        }
    });
    let struct_options_tokenstream = quote::quote! {
        #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize)]
        pub struct #struct_options_ident_token_stream {
            #(#fields_options),*
        }
    };
    let s = {
        let fields_named_len =  fields_named.len();
        let mut hashmap_prep_invariants: std::collections::HashMap<std::string::String, Vec<&std::string::String>> = HashMap::with_capacity(fields_named_len);
        let fields_named_stringified = fields_named.iter().map(|field|{
            field.ident.clone().unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} field.ident is None")).to_string()
        }).collect::<Vec<std::string::String>>();
        fields_named.iter().for_each(|field|{
            let field_ident_stringified = field.ident.clone().unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} field.ident is None")).to_string();
            // let filtered_vec = Vec::with_capacity(fields_named_len);
            // let filtered_vec = fields_named_stringified.c
//
            let filtered_vec = fields_named_stringified.iter().fold(Vec::with_capacity(fields_named_len), |mut acc, elem| {
                if let false = &field_ident_stringified == elem {
                    acc.push(elem);
                }
                acc
            });
//
            hashmap_prep_invariants.insert(field_ident_stringified, filtered_vec);
        });
        println!("{hashmap_prep_invariants:#?}");
        let mut vec_variants = Vec::with_capacity(fields_named_len * fields_named_len);

        hashmap_prep_invariants.iter().for_each(|(key, value)|{
            let key_first_upper = key.to_case(convert_case::Case::Camel);
            if let false = vec_variants.contains(&key_first_upper) {
                vec_variants.push(key_first_upper);
            }
            // value.iter().for_each(|v|{
            //     let 
            // });
        });
    };

    // println!("{struct_options_tokenstream}");
    let gen = quote::quote! {
        // pub struct Cat {
        //     pub id: i64, //todo - if using js JSON.parse() - must be two variants - for usage and deserialization - coz json number type capacity less than i64::MAX
        //     pub name: String,
        //     pub color: String,
        // }
        #struct_options_tokenstream

    // #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
    // pub struct CatId {
    //     pub id: i64,
    // }
    // #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
    // pub struct CatName {
    //     pub name: String,
    // }
    // #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
    // pub struct CatColor {
    //     pub color: String,
    // }
    // #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
    // pub struct CatIdName {
    //     pub id: i64,
    //     pub name: String,
    // }
    // #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
    // pub struct CatIdColor {
    //     pub id: i64,
    //     pub color: String,
    // }
    // #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
    // pub struct CatNameColor {
    //     pub name: String,
    //     pub color: String,
    // }
    // #[derive(Debug, serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
    // pub struct CatIdNameColor {
    //     pub id: i64,
    //     pub name: String,
    //     pub color: String,
    // }

            };
    // println!("{gen}");
    gen.into()
}
