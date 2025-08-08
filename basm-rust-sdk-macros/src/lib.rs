use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use quote::{quote};
use syn::{parse_macro_input, ItemFn, Signature, Type};

// secret and input type should be optional and derived from ctx type if possible.
#[derive(Debug, FromMeta)]
struct EntrypointArgs {
    #[darling(default="default_json_deserializer_fn")]
    secret_deserializer: syn::TypePath,
    #[darling(default="default_json_deserializer_fn")]
    input_deserializer: syn::TypePath,
    #[darling(default="default_json_serializer_fn")]
    output_serializer: syn::TypePath,
}

fn default_json_serializer_fn() -> syn::TypePath {
    syn::parse_str("serde_json::to_string").unwrap()
}
fn default_json_deserializer_fn() -> syn::TypePath {
    syn::parse_str("serde_json::from_str").unwrap()
}

// Extracts the context types from the function signature.
fn extract_context_types(sig: &Signature) -> Result<(Type, Type), syn::Error> {
    // 1. Get the first function argument.
    let first_arg = sig.inputs.first().ok_or_else(|| {
        syn::Error::new_spanned(sig, "Function must have at least one argument")
    })?;

    // 2. Check if the first argument is a typed argument.
    let pat_type = if let syn::FnArg::Typed(pt) = first_arg {
        pt
    } else {
        return Err(syn::Error::new_spanned(
            first_arg,
            "Expected a typed argument",
        ));
    };

    // 3. Ensure the argument's type is a path type, e.g., `my_crate::Context<A, B>`.
    let type_path = if let syn::Type::Path(tp) = &*pat_type.ty {
        tp
    } else {
        return Err(syn::Error::new_spanned(
            &pat_type.ty,
            "Argument type must be a path like `Context<A, B>`",
        ));
    };

    // 4. Get the last part of the path (the type name and its generics).
    let last_segment = type_path.path.segments.last().ok_or_else(|| {
        syn::Error::new_spanned(&type_path.path, "Type path must have at least one segment")
    })?;

    // 5. Check that the type is `Context`.
    if last_segment.ident != "Context" {
        return Err(syn::Error::new_spanned(
            &last_segment.ident,
            "Expected the argument type to be `Context`",
        ));
    }

    // 6. Extract the angle-bracketed generic arguments, e.g., `<A, B>`.
    let angle_bracketed_args =
        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
            args
        } else {
            return Err(syn::Error::new_spanned(
                last_segment,
                "Expected angle-bracketed generic arguments like `<A, B>`",
            ));
        };

    // 7. Collect all the *type* arguments from the generics.
    let generic_types: Vec<&Type> = angle_bracketed_args
        .args
        .iter()
        .filter_map(|arg| if let syn::GenericArgument::Type(ty) = arg { Some(ty) } else { None })
        .collect();

    // 8. There should be either 1 or 2 type arguments.
    if generic_types.len() < 1 || generic_types.len() > 2 {
        return Err(syn::Error::new_spanned(
            &angle_bracketed_args.args,
            "Expected 1 or 2 type arguments in `Context<A, B>`",
        ));
    }

    if generic_types.len() == 1 {
        return Ok((
            generic_types[0].clone(),
            syn::parse_quote!(()),
        ));
    } else {
        return Ok((
            generic_types[0].clone(),
            generic_types[1].clone(),
        ));
    }
}

#[proc_macro_attribute]
pub fn bky_entrypoint(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => return e.into_compile_error().into(),
    };
    let EntrypointArgs {
        input_deserializer,
        output_serializer,
        secret_deserializer
    } = match EntrypointArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let input = parse_macro_input!(input as ItemFn);
    let sig = &input.sig;

    let (input_type, secret_type) = match extract_context_types(sig) {
        Ok((input_type, secret_type)) => (input_type, secret_type),
        Err(e) => return e.into_compile_error().into(),
    };

    let fn_name = &sig.ident.clone();

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn #fn_name(
            input_ptr: basm_rust_sdk::memory::FatPointer,
            secret_ptr: basm_rust_sdk::memory::FatPointer
        ) -> basm_rust_sdk::memory::FatPointer {
            #input

            std::panic::set_hook(Box::new(|p| {
                basm_rust_sdk::host_log!("{}", p);
                std::process::abort();
            }));

            let input_data = input_ptr.copy_data();
            let secret_data = secret_ptr.copy_data();

            let input_str = str::from_utf8(&input_data).unwrap();
            let input: #input_type = #input_deserializer(input_str)
                .expect("Failed to deserialize input JSON");

            let secrets_str = str::from_utf8(&secret_data).unwrap();
            let secrets: #secret_type = #secret_deserializer(secrets_str)
                .expect("Failed to deserialize secrets JSON");

            let ctx = Context {
                input: input,
                secrets: secrets
            };

            let output = #fn_name(ctx);

            let x = #output_serializer(&output)
                .expect("Failed to serialize output into JSON");
            basm_rust_sdk::memory::leak_to_shared_memory(&x.as_bytes())
        }
    }.into()
}
