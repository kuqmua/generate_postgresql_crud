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
        let fields_named_clone = fields_named.clone();
        let fields_named_clone_len =  fields_named_clone.len();
        // let mut hashmap_prep_invariants: std::collections::HashMap<std::string::String, Vec<&std::string::String>> = std::collections::HashMap::with_capacity(fields_named_clone_len);
        let mut vec_prep_invariants: Vec<(std::string::String, Vec<&std::string::String>)> = Vec::with_capacity(fields_named_clone_len);
        let mut fields_named_clone_stringified = fields_named_clone.iter().map(|field|{
            let f = field.ident.clone().unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} field.ident is None")).to_string();
            println!("111{f}");
            f
        }).collect::<Vec<std::string::String>>();
        println!("{fields_named_clone_stringified:#?}");
        // let mut hashmap_enumeration: std::collections::HashMap<&std::string::String, usize> = std::collections::HashMap::with_capacity(fields_named_clone_len);
        let mut vec_enumeration = Vec::new();
        fields_named_clone_stringified.iter().enumerate().for_each(|(index, field)|{
            // hashmap_enumeration.insert(field, index);
             vec_enumeration.push((field, index));
        });
        // println!("{hashmap_enumeration:#?}");
        println!("{vec_enumeration:#?}");
        
        // fields_named_clone_stringified.sort();
        fields_named_clone.iter().for_each(|field|{
            let field_ident = &field.ident.clone().unwrap();
            println!("222{field_ident}");
            let field_ident_stringified = field.ident.clone().unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} field.ident is None")).to_string();
            let mut filtered_vec = fields_named_clone_stringified.iter().fold(Vec::with_capacity(fields_named_clone_len), |mut acc, elem| {
                if let false = &field_ident_stringified == elem {
                    acc.push(elem);
                }
                acc
            });
            // filtered_vec.sort();

            // hashmap_prep_invariants.insert(field_ident_stringified, filtered_vec);
            vec_prep_invariants.push((field_ident_stringified, filtered_vec));
        });
        // println!("before reverse {hashmap_prep_invariants:#?}");
        println!("before reverse {vec_prep_invariants:#?}");
        // hashmap_prep_invariants.reserve(0);
        // println!("after reverse {hashmap_prep_invariants:#?}");
        let mut vec_variants = Vec::with_capacity(fields_named_clone_len * fields_named_clone_len);

        println!("stage 1 {vec_variants:#?}");
        vec_prep_invariants.iter().for_each(|(key, value)|{
            vec_variants.push(format!("{key}"));
        });
        vec_prep_invariants.iter().for_each(|(key, value)|{

            println!("333{key}");
            
            let mut g = Vec::new();
            g.push(key.clone());
            value.iter().for_each(|v|{
                g.push(v.to_string());
            });
            g.sort();
            g.iter().for_each(|v|{
                if let true = key != v {
                    let (_, key_index) = vec_enumeration.iter().find(|(keyy, index)|{
                        *keyy == key
                    }).unwrap();
                    let (_, v_index) = vec_enumeration.iter().find(|(keyy, index)|{
                        *keyy == v
                    }).unwrap();
                    // let key_index = hashmap_enumeration.get(key).unwrap_or_else(||panic!("{proc_macro_name_ident_stringified} hashmap_enumeration.get(key) is None"));
                    // let v_index = hashmap_enumeration.get(v).unwrap_or_else(||panic!("{proc_macro_name_ident_stringified} hashmap_enumeration.get(v) is None"));
                    if key_index < v_index {//todo maybe use .enumerate for compare index and not alphabet
                        let r = format!("{key}{v}");
                        if !vec_variants.contains(&r) {
                            vec_variants.push(r);
                        }
                    }
                    
                }
            });
        });
        let keys_merged = fields_named_clone.iter().fold(std::string::String::from(""), |mut acc, field| {
                acc.push_str(&field.ident.clone().unwrap().to_string());
                acc
            });
        vec_variants.push(keys_merged);
        println!("{vec_variants:#?}");

//
// [
//     "idnamecolor",
//     "name",
//     "namecolor",
//     "color",
//     "id",
//     "idcolor",
//     "idname",
// ]
// [
//     "idnamecolor",
//     "name",
//     "namecolor",
//     "color",
//     "id",
//     "idcolor",
//     "idname",
// ]
// [
//     "idnamecolor",
//     "name",
//     "namecolor",
//     "color",
//     "id",
//     "idcolor",
//     "idname",
// ]
// [
//     "idnamecolor",
//     "color",
//     "id",
//     "idcolor",
//     "idname",
//     "name",
//     "namecolor",
// ]
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