// #[proc_macro_derive(GeneratePostgresqlCrud)]
// pub fn generate_postgresql_crud(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     proc_macro_helpers::panic_location::panic_location();
//     let proc_macro_name = "GeneratePostgresqlCrud";
//     let ast: syn::DeriveInput = syn::parse(input).unwrap_or_else(|_| {
//         panic!(
//             "{proc_macro_name} {}",
//             proc_macro_helpers::global_variables::hardcode::AST_PARSE_FAILED
//         )
//     });
//     let ident = &ast.ident;
//     let proc_macro_name_ident_stringified = format!("{proc_macro_name} {ident}");
//     let data_struct = if let syn::Data::Struct(data_struct) = ast.data {
//         // println!("{data_struct:#?}");
//         data_struct
//     } else {
//         panic!("does not work on structs!");
//     };
//     let fields_named = if let syn::Fields::Named(fields_named) = data_struct.fields {
//         fields_named.named
//     } else {
//         panic!("{proc_macro_name_ident_stringified} supports only syn::Fields::Named");
//     };
//     let struct_options_ident_stringified = format!("{ident}Options");
//     let struct_options_ident_token_stream =
//     struct_options_ident_stringified.parse::<proc_macro2::TokenStream>()
//         .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {struct_options_ident_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
//     let fields_options = fields_named.iter().map(|field| {
//         let field_vis = &field.vis;
//         let field_ident = &field.ident;
//         let field_type_path = &field.ty;
//         quote::quote! {
//             #[serde(skip_serializing_if = "Option::is_none")]
//             #field_vis #field_ident: Option<#field_type_path>
//         }
//     });
//     let struct_options_tokenstream = quote::quote! {
//         #[derive(serde_derive::Serialize, serde_derive::Deserialize)]
//         pub struct #struct_options_ident_token_stream {
//             #(#fields_options),*
//         }
//     };
//     // let s = {
//     //     let mut fields_named_enumerated_cloned_stringified = fields_named.clone().iter().enumerate().map(|(index, field)|{
//     //         let f = field.ident.clone().unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} field.ident is None")).to_string();
//     //         (index, f)
//     //     }).collect::<Vec<(usize, std::string::String)>>();
//     //     //
//     //     let mut fields_named_clone_stringified = fields_named.clone().iter().map(|field|{
//     //         let f = field.ident.clone().unwrap_or_else(|| panic!("{proc_macro_name_ident_stringified} field.ident is None")).to_string();
//     //         f
//     //     }).collect::<Vec<std::string::String>>();
//     //     let mut veced_vec = fields_named_clone_stringified.clone().iter().map(|v| vec![v.clone()]).collect();
//     //     let f = column_names_factorial(
//     //         fields_named_enumerated_cloned_stringified,
//     //         fields_named_clone_stringified.clone(),
//     //         &mut veced_vec
//     //     );
//     // println!("{f:#?}");
//     // println!("{}", f.len());
//     // };

//     // println!("{struct_options_tokenstream}");
//     let gen = quote::quote! {
//         // pub struct Cat {
//         //     pub id: i64, //todo - if using js JSON.parse() - must be two variants - for usage and deserialization - coz json number type capacity less than i64::MAX
//         //     pub name: String,
//         //     pub color: String,
//         // }
//         // #struct_options_tokenstream

//     // #[derive(serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
//     // pub struct CatId {
//     //     pub id: i64,
//     // }
//     // #[derive(serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
//     // pub struct CatName {
//     //     pub name: String,
//     // }
//     // #[derive(serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
//     // pub struct CatColor {
//     //     pub color: String,
//     // }
//     // #[derive(serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
//     // pub struct CatIdName {
//     //     pub id: i64,
//     //     pub name: String,
//     // }
//     // #[derive(serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
//     // pub struct CatIdColor {
//     //     pub id: i64,
//     //     pub color: String,
//     // }
//     // #[derive(serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
//     // pub struct CatNameColor {
//     //     pub name: String,
//     //     pub color: String,
//     // }
//     // #[derive(serde_derive::Serialize, serde_derive::Deserialize, sqlx::FromRow)]
//     // pub struct CatIdNameColor {
//     //     pub id: i64,
//     //     pub name: String,
//     //     pub color: String,
//     // }

//             };
//     // println!("{gen}");
//     gen.into()
// }

// fn column_names_factorial(
//     original_input: Vec<(usize, String)>,
//     input: Vec<String>,
//     output: &mut Vec<Vec<String>>,
// ) -> Vec<Vec<String>> {
//     let len = input.len();
//     match len {
//         0 => {
//             let mut end_out = {
//                 let output_len = output.len();
//                 output.iter_mut().fold(Vec::with_capacity(output_len), |mut acc, element| {
//                     element.sort_by(|a,b|{
//                         let (index_a, _) = original_input.iter().find(|(_, field)|{a == field}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
//                         let (index_b, _) = original_input.iter().find(|(_, field)|{b == field}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
//                         index_a.partial_cmp(index_b).unwrap_or_else(||panic!("GeneratePostgresqlCrud index_a.partial_cmp(index_b) is None"))
//                     });
//                     acc.push(element.to_vec());
//                     acc
//                 })
//             };
//             end_out.sort_by(|a,b|match a.len() == b.len() {
//                 true => {
//                     let mut order = std::cmp::Ordering::Equal;
//                     for a_elem in a {
//                         let mut is_order_found = false;
//                         for b_elem in a {
//                             if let Some(or) = a_elem.partial_cmp(b_elem) {
//                                 match or {
//                                     std::cmp::Ordering::Less => {
//                                         order = or;
//                                         is_order_found = true;
//                                         break;
//                                     },
//                                     std::cmp::Ordering::Equal => (),
//                                     std::cmp::Ordering::Greater => {
//                                         order = or;
//                                         is_order_found = true;
//                                         break;
//                                     },
//                                 }
//                             }
//                         }
//                         if let true = is_order_found {
//                             break;
//                         }
//                     }
//                     order

//                 },
//                 false => std::cmp::Ordering::Equal,
//             });
//             end_out.sort_by(|a,b|{
//                 a.len().partial_cmp(&b.len()).unwrap_or_else(||panic!("GeneratePostgresqlCrud index_a.partial_cmp(index_b) is None"))
//             });
//             end_out
//         }
//         // 1 => {
//         //     let mut output_handle = vec![];
//         //     original_input.iter().for_each(|(_, element)| {
//         //         output_handle.push(vec![element.clone()]);
//         //     });
//         //     let first_element = input.get(0).unwrap_or_else(||panic!("GeneratePostgresqlCrud input.get(0) is None"));
//         //     output.iter().for_each(
//         //         |o| {
//         //             if let false = o.contains(first_element) {
//         //                 let mut cl = o.clone();
//         //                 cl.push(format!("{}", input[0]));
//         //                 cl.sort_by(|a,b|{
//         //                     let (index_a, _) = original_input.iter().find(|(_, field)|{a == field}).unwrap();
//         //                     let (index_b, _) = original_input.iter().find(|(_, field)|{b == field}).unwrap();
//         //                     index_a.partial_cmp(index_b).unwrap()
//         //                 });
//         //                 output_handle.push(cl);
//         //             }
//         //         },
//         //     );
//         //     output_handle
//         // }
//         _ => {
//             let mut output_handle = {
//                 let first_element = input.get(0).unwrap_or_else(||panic!("GeneratePostgresqlCrud input.get(0) is None"));
//                 let output_len = output.len();
//                 output.iter_mut().fold(Vec::with_capacity(output_len * 2), |mut acc, out| {
//                     if !acc.contains(out) {
//                         out.sort_by(|a,b|{
//                             let (index_a, _) = original_input.iter().find(|(_, field)|{a == field}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
//                             let (index_b, _) = original_input.iter().find(|(_, field)|{b == field}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
//                             index_a.partial_cmp(index_b).unwrap_or_else(||panic!("GeneratePostgresqlCrud index_a.partial_cmp(index_b) is None"))
//                         });
//                         acc.push(out.clone());
//                     }
//                     if let false = out.contains(first_element) {
//                         let mut cl = out.clone();
//                         cl.push(first_element.to_string());
//                         cl.sort_by(|a,b|{
//                             let (index_a, _) = original_input.iter().find(|(_, field)|{a == field}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
//                             let (index_b, _) = original_input.iter().find(|(_, field)|{b == field}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
//                             index_a.partial_cmp(index_b).unwrap_or_else(||panic!("GeneratePostgresqlCrud index_a.partial_cmp(index_b) is None"))
//                         });
//                         if !acc.contains(&cl) {
//                             acc.push(cl);
//                         }
//                     }
//                     acc
//                 })
//             };
//             let new_input_vec = {
//                 let input_len = input.len();
//                 input.into_iter().enumerate().fold(Vec::with_capacity(input_len), |mut acc, (index, value)| {
//                     if let true = index != 0 {
//                         acc.push(value);
//                     }
//                     acc
//                 })
//             };
//             column_names_factorial(original_input, new_input_vec, &mut output_handle)
//         }
//     }
// }
