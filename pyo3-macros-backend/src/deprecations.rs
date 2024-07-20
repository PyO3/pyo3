use crate::method::{FnArg, FnSpec};
use proc_macro2::TokenStream;
use quote::quote_spanned;

pub(crate) fn deprecate_trailing_option_default(spec: &FnSpec<'_>) -> TokenStream {
    if spec.signature.attribute.is_none()
        && spec.tp.signature_attribute_allowed()
        && spec.signature.arguments.iter().any(|arg| {
            if let FnArg::Regular(arg) = arg {
                arg.option_wrapped_type.is_some()
            } else {
                false
            }
        })
    {
        use std::fmt::Write;
        let mut deprecation_msg = String::from(
            "this function has implicit defaults for the trailing `Option<T>` arguments \n\
             = note: these implicit defaults are being phased out \n\
             = help: add `#[pyo3(signature = (",
        );
        spec.signature.arguments.iter().for_each(|arg| {
            match arg {
                FnArg::Regular(arg) => {
                    if arg.option_wrapped_type.is_some() {
                        write!(deprecation_msg, "{}=None, ", arg.name)
                    } else {
                        write!(deprecation_msg, "{}, ", arg.name)
                    }
                }
                FnArg::VarArgs(arg) => write!(deprecation_msg, "{}, ", arg.name),
                FnArg::KwArgs(arg) => write!(deprecation_msg, "{}, ", arg.name),
                FnArg::Py(_) | FnArg::CancelHandle(_) => Ok(()),
            }
            .expect("writing to `String` should not fail");
        });

        //remove trailing space and comma
        deprecation_msg.pop();
        deprecation_msg.pop();

        deprecation_msg.push_str(
            "))]` to this function to silence this warning and keep the current behavior",
        );
        quote_spanned! { spec.name.span() =>
            #[deprecated(note = #deprecation_msg)]
            #[allow(dead_code)]
            const SIGNATURE: () = ();
            const _: () = SIGNATURE;
        }
    } else {
        TokenStream::new()
    }
}
