//! Procedural macros used by contrib plugins such as `summer-pubsub` and `summer-sa-token`.
//!
//! Application code typically imports these through the plugin crate
//! (`summer_pubsub::pubsub_listener`, `summer_sa_token::sa_check_login`, …).

mod pubsub;
mod sa_token;

use proc_macro::TokenStream;

fn input_and_compile_error(mut item: TokenStream, err: syn::Error) -> TokenStream {
    let compile_err = TokenStream::from(err.to_compile_error());
    item.extend(compile_err);
    item
}

/// Google Cloud Pub/Sub listener (subscription id or full resource name).
#[proc_macro_attribute]
pub fn pubsub_listener(args: TokenStream, input: TokenStream) -> TokenStream {
    pubsub::listener(args, input)
}

/// Check login status
#[proc_macro_attribute]
pub fn sa_check_login(attr: TokenStream, input: TokenStream) -> TokenStream {
    sa_token::sa_check_login_impl(attr, input)
}

/// Check user role
#[proc_macro_attribute]
pub fn sa_check_role(attr: TokenStream, input: TokenStream) -> TokenStream {
    sa_token::sa_check_role_impl(attr, input)
}

/// Check user permission
#[proc_macro_attribute]
pub fn sa_check_permission(attr: TokenStream, input: TokenStream) -> TokenStream {
    sa_token::sa_check_permission_impl(attr, input)
}

/// Check multiple roles with AND logic
#[proc_macro_attribute]
pub fn sa_check_roles_and(attr: TokenStream, input: TokenStream) -> TokenStream {
    sa_token::sa_check_roles_and_impl(attr, input)
}

/// Check multiple roles with OR logic
#[proc_macro_attribute]
pub fn sa_check_roles_or(attr: TokenStream, input: TokenStream) -> TokenStream {
    sa_token::sa_check_roles_or_impl(attr, input)
}

/// Check multiple permissions with AND logic
#[proc_macro_attribute]
pub fn sa_check_permissions_and(attr: TokenStream, input: TokenStream) -> TokenStream {
    sa_token::sa_check_permissions_and_impl(attr, input)
}

/// Check multiple permissions with OR logic
#[proc_macro_attribute]
pub fn sa_check_permissions_or(attr: TokenStream, input: TokenStream) -> TokenStream {
    sa_token::sa_check_permissions_or_impl(attr, input)
}

/// Ignore authentication for this endpoint
#[proc_macro_attribute]
pub fn sa_ignore(attr: TokenStream, input: TokenStream) -> TokenStream {
    sa_token::sa_ignore_impl(attr, input)
}
