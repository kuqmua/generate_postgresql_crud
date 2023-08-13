#[proc_macro_derive()]
pub fn (input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro_helpers::panic_location::panic_location();//panic_location function from https://github.com/kuqmua/proc_macro_helpers
    let ast: syn::DeriveInput =
        syn::parse(input).unwrap_or_else(|_| panic!("let ast: syn::DeriveInput = syn::parse(input) failed"));
    let ident = &ast.ident;
    match ast.data {
        syn::Data::Union(_) => panic!("does not work on union!"),
        syn::Data::Struct(_) => panic!("does not work on structs!"),
        syn::Data::Enum(_) => panic!("does not work on enums!"),
    }
    let gen = quote::quote! {};
    //println!("{gen}");
    gen.into()
}
