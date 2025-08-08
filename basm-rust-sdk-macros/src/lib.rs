use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use quote::{quote};
use syn::{parse_macro_input, ItemFn, TypePath};

#[derive(Debug, FromMeta)]
struct EntrypointArgs {
    secret_type: TypePath,
    input_type: TypePath,
}


#[proc_macro_attribute]
pub fn bky_entrypoint(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };
    let parsed_args = match EntrypointArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let input_type = &parsed_args.input_type;
    let secret_type = &parsed_args.secret_type;

    let input = parse_macro_input!(input as ItemFn);

    let sig = &input.sig;

    let fn_name = &sig.ident.clone();

    let output = &sig.output;
    
    let body = &input.block;


    // ensure that the function has the correct signature, with Context and returning Serializable O
    // function should be generic accross I, S and O

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn #fn_name(
            input_ptr: basm_rust_sdk::memory::FatPointer,
            secret_ptr: basm_rust_sdk::memory::FatPointer
        ) -> basm_rust_sdk::memory::FatPointer {
            #input

            let input_data = input_ptr.copy_data();
            let secret_data = secret_ptr.copy_data();

            let input_str = str::from_utf8(&input_data).unwrap();
            let input: #input_type = serde_json::from_str(input_str)
                .expect("Failed to deserialize input JSON");

            let secrets_str = str::from_utf8(&secret_data).unwrap();
            let secrets: #secret_type = serde_json::from_str(secrets_str)
                .expect("Failed to deserialize secrets JSON");

            let ctx = Context {
                input: input,
                secrets: secrets
            };

            let output = #fn_name(ctx);

            let x = serde_json::to_string(&output)
                .expect("Failed to serialize output into JSON");
            basm_rust_sdk::memory::leak_to_shared_memory(&x.as_bytes())
        }
    }.into()
}
