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
        #[derive(serde_derive::Serialize, serde_derive::Deserialize)]
        pub struct #struct_options_ident_token_stream {
            #(#fields_options),*
        }
    };
    let column_variants = {
        let fields_named_enumerated_cloned_stringified = fields_named
            .iter()
            .enumerate()
            .map(|(index, field)| (index, field))
            .collect::<Vec<(usize, &syn::Field)>>();
        let fields_named_clone_stringified = fields_named
            .iter()
            .map(|field| field)
            .collect::<Vec<&syn::Field>>();
        let mut veced_vec = fields_named_clone_stringified
            .iter()
            .map(|field| {
                vec![{
                    field
                        .ident
                        .clone()
                        .unwrap_or_else(|| {
                            panic!("{proc_macro_name_ident_stringified} field.ident is None")
                        })
                        .to_string()
                }]
            })
            .collect();
        let column_variants = column_names_factorial(
            fields_named_enumerated_cloned_stringified,
            fields_named_clone_stringified,
            &mut veced_vec,
        );
        println!("{column_variants:#?}\n{}", column_variants.len());
        column_variants
    };
    let structs_variants = {
        column_variants
            .iter()
            .map(|variant_columns| {
                let mut struct_name_stringified = format!("{ident}");
                variant_columns.iter().for_each(|variant_column| {
                    let column_title_cased = variant_column.to_case(convert_case::Case::Title);
                    struct_name_stringified.push_str(&column_title_cased);
                });
                let struct_name_token_stream = struct_name_stringified.parse::<proc_macro2::TokenStream>()
                .unwrap_or_else(|_| panic!("{proc_macro_name_ident_stringified} {struct_name_stringified} {}", proc_macro_helpers::global_variables::hardcode::PARSE_PROC_MACRO2_TOKEN_STREAM_FAILED_MESSAGE));
                quote::quote! {
                    pub struct #struct_name_token_stream {
                    //    pub id: i64,
                    //    pub name: String,
                    }
                }
            })
            .collect::<proc_macro2::TokenStream>()
    };
    // println!("{struct_options_tokenstream}");
    let gen = quote::quote! {
        // pub struct Cat {
        //     pub id: i64, //todo - if using js JSON.parse() - must be two variants - for usage and deserialization - coz json number type capacity less than i64::MAX
        //     pub name: String,
        //     pub color: String,
        // }
        #struct_options_tokenstream


    // pub struct CatId {
    //     pub id: i64,
    // }

    // pub struct CatName {
    //     pub name: String,
    // }

    // pub struct CatColor {
    //     pub color: String,
    // }

    // pub struct CatIdName {
    //     pub id: i64,
    //     pub name: String,
    // }

    // pub struct CatIdColor {
    //     pub id: i64,
    //     pub color: String,
    // }

    // pub struct CatNameColor {
    //     pub name: String,
    //     pub color: String,
    // }

    // pub struct CatIdNameColor {
    //     pub id: i64,
    //     pub name: String,
    //     pub color: String,
    // }

            };
    // println!("{gen}");
    gen.into()
}

fn column_names_factorial(
    original_input: Vec<(usize, &syn::Field)>,
    input: Vec<&syn::Field>,
    output: &mut Vec<Vec<String>>,
) -> Vec<Vec<String>> {
    let len = input.len();
    match len {
        0 => {
            let mut end_out = {
                let output_len = output.len();
                output
                    .iter_mut()
                    .fold(Vec::with_capacity(output_len), |mut acc, element| {
                        element.sort_by(|a, b| {
                            let (index_a, _) = original_input
                                .iter()
                                .find(|(_, field)| {
                                    a == &field
                                        .ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })
                                        .to_string()
                                })
                                .unwrap_or_else(|| {
                                    panic!(
                                        "GeneratePostgresqlCrud cannot find original input index"
                                    )
                                });
                            let (index_b, _) = original_input
                                .iter()
                                .find(|(_, field)| {
                                    b == &field
                                        .ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })
                                        .to_string()
                                })
                                .unwrap_or_else(|| {
                                    panic!(
                                        "GeneratePostgresqlCrud cannot find original input index"
                                    )
                                });
                            index_a.partial_cmp(index_b).unwrap_or_else(|| {
                                panic!(
                                    "GeneratePostgresqlCrud index_a.partial_cmp(index_b) is None"
                                )
                            })
                        });
                        acc.push(element.to_vec());
                        acc
                    })
            };
            end_out.sort_by(|a, b| match a.len() == b.len() {
                true => {
                    let mut order = std::cmp::Ordering::Equal;
                    for a_elem in a {
                        let mut is_order_found = false;
                        for b_elem in a {
                            if let Some(or) = a_elem.partial_cmp(b_elem) {
                                match or {
                                    std::cmp::Ordering::Less => {
                                        order = or;
                                        is_order_found = true;
                                        break;
                                    }
                                    std::cmp::Ordering::Equal => (),
                                    std::cmp::Ordering::Greater => {
                                        order = or;
                                        is_order_found = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if let true = is_order_found {
                            break;
                        }
                    }
                    order
                }
                false => std::cmp::Ordering::Equal,
            });
            end_out.sort_by(|a, b| {
                a.len().partial_cmp(&b.len()).unwrap_or_else(|| {
                    panic!("GeneratePostgresqlCrud index_a.partial_cmp(index_b) is None")
                })
            });
            end_out
        }
        // 1 => {
        //     let mut output_handle = vec![];
        //     original_input.iter().for_each(|(_, element)| {
        //         output_handle.push(vec![element.clone()]);
        //     });
        //     let first_element = input.get(0).unwrap_or_else(||panic!("GeneratePostgresqlCrud input.get(0) is None"));
        //     output.iter().for_each(
        //         |o| {
        //             if let false = o.contains(first_element) {
        //                 let mut cl = o.clone();
        //                 cl.push(format!("{}", input[0]));
        //                 cl.sort_by(|a,b|{
        //                     let (index_a, _) = original_input.iter().find(|(_, field)|{a == field}).unwrap();
        //                     let (index_b, _) = original_input.iter().find(|(_, field)|{b == field}).unwrap();
        //                     index_a.partial_cmp(index_b).unwrap()
        //                 });
        //                 output_handle.push(cl);
        //             }
        //         },
        //     );
        //     output_handle
        // }
        _ => {
            let mut output_handle = {
                let first_element = input
                    .get(0)
                    .unwrap_or_else(|| panic!("GeneratePostgresqlCrud input.get(0) is None"));
                let output_len = output.len();
                output.iter_mut().fold(Vec::with_capacity(output_len * 2), |mut acc, out| {
                    if !acc.contains(out) {
                        out.sort_by(|a,b|{
                            let (index_a, _) = original_input.iter().find(|(_, field)|{a == &field
                                        .ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })
                                        .to_string()}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
                            let (index_b, _) = original_input.iter().find(|(_, field)|{b == &field
                                        .ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })
                                        .to_string()}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
                            index_a.partial_cmp(index_b).unwrap_or_else(||panic!("GeneratePostgresqlCrud index_a.partial_cmp(index_b) is None"))
                        });
                        acc.push(out.clone());
                    }
                    if let false = out.contains(&first_element.ident.clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })
                                        .to_string()) {
                        let mut cl = out.clone();
                        cl.push(first_element.ident.clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })
                                        .to_string());
                        cl.sort_by(|a,b|{
                            let (index_a, _) = original_input.iter().find(|(_, field)|{a == &field
                                        .ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })
                                        .to_string()}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
                            let (index_b, _) = original_input.iter().find(|(_, field)|{b == &field
                                        .ident
                                        .clone()
                                        .unwrap_or_else(|| {
                                            panic!("GeneratePostgresqlCrud field.ident is None")
                                        })
                                        .to_string()}).unwrap_or_else(||panic!("GeneratePostgresqlCrud cannot find original input index"));
                            index_a.partial_cmp(index_b).unwrap_or_else(||panic!("GeneratePostgresqlCrud index_a.partial_cmp(index_b) is None"))
                        });
                        if !acc.contains(&cl) {
                            acc.push(cl);
                        }
                    }
                    acc
                })
            };
            let new_input_vec = {
                let input_len = input.len();
                input.into_iter().enumerate().fold(
                    Vec::with_capacity(input_len),
                    |mut acc, (index, value)| {
                        if let true = index != 0 {
                            acc.push(value);
                        }
                        acc
                    },
                )
            };
            column_names_factorial(original_input, new_input_vec, &mut output_handle)
        }
    }
}
