//! `ToTokens` implementations for the types in [`json_tests::case`].

use diplomacy::{judge::OrderState, ShortName};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};

use crate::case::{build, main, retreat, Cases, RawTestCase, TestCase, TestCaseBody, TestCaseTodo};

fn order_state_to_ident(state: OrderState) -> proc_macro2::Ident {
    match state {
        OrderState::Succeeds => format_ident!("Succeeds"),
        OrderState::Fails => format_ident!("Fails"),
    }
}

impl<T: ToTokens> ToTokens for Cases<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let cases = &self.cases;
        tokens.append_all(quote!(
            //! This module was automatically generated. Do not edit it directly.
            #![cfg(test)]

            #[path = "./util.rs"]
            mod util;
            #[path = "./world.rs"]
            mod world;

            use diplomacy::judge::OrderState::{Fails, Succeeds};
            use util::*;
            use world::TestWorld;

            #(#cases)*
        ));
    }
}

impl ToTokens for main::TestCase {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let start_state = self.starting_state.as_ref().map(|starting_state| {
            let starting_state = starting_state.iter().map(std::string::ToString::to_string);
            tokens.append_all(quote! {
                let starting_state = vec![#(unit_pos(#starting_state)),*];
            });
            quote! {
                @start &starting_state;
            }
        });

        let orders = self.orders.iter().map(|(order, expectation)| {
            let order = order.to_string();
            let expectation = expectation.map(order_state_to_ident);
            match expectation {
                Some(expectation) => quote!(#order: #expectation),
                None => quote!(#order),
            }
        });

        tokens.append_all(quote! {
            judge! {
                #start_state
                #(#orders),*
            };
        });
    }
}

impl ToTokens for retreat::TestCase {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let main_phase = self
            .preceding_main_phase
            .orders
            .iter()
            .map(|(order, expectation)| {
                let order = order.to_string();
                let expectation = expectation.map(order_state_to_ident);
                match expectation {
                    Some(expectation) => quote!(#order: #expectation),
                    None => quote!(#order),
                }
            });

        let orders = self.orders.iter().map(|(order, expectation)| {
            let order = order.to_string();
            let expectation = expectation.map(order_state_to_ident);
            match expectation {
                Some(expectation) => quote!(#order: #expectation),
                None => quote!(#order),
            }
        });

        tokens.append_all(quote! {
            let (submission, expected) = submit_main_phase! {
                #(#main_phase,)*
            };

            let outcome = resolve_main!(submission, expected);

            judge_retreat! {
                outcome,
                #(#orders),*
            };
        });
    }
}

impl ToTokens for build::TestCase {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let occupiers = self.occupiers.iter().flatten().map(|(province, nation)| {
            let province = province.short_name();
            let nation = nation.to_string();
            quote!(.with_occupier(#province, #nation))
        });

        let units = self.starting_state.iter().flatten().map(|unit| {
            let unit = unit.to_string();
            quote!(.with_unit(#unit))
        });

        let orders = self.orders.iter().map(|(order, expectation)| {
            let order = order.to_string();
            let expectation = expectation.map(order_state_to_ident);
            match expectation {
                Some(expectation) => quote!(#order: #expectation),
                None => quote!(#order),
            }
        });

        // Keep this out of the judge_build! macro call so it's formatted properly.
        tokens.append_all(quote! {
            let world = TestWorld::empty()
                #(#occupiers)*
                #(#units)*;
        });

        let judge_call = quote! {
            judge_build! {
                world,
                #(#orders),*
            };
        };

        if let Some(civil_disorder) = &self.civil_disorder {
            let civil_disorder = civil_disorder.iter().map(|pos| {
                let pos = pos.to_string();
                quote!(unit_pos(#pos))
            });

            let inner_assertion = quote! {
                assert!(civil_disorder.contains(&disbanded), "{disbanded} should have disbanded");
            };

            tokens.append_all(quote! {
                let (_, civil_disorder) = #judge_call;
            });

            let disband_check = if civil_disorder.len() == 1 {
                quote! {
                    let disbanded = #(#civil_disorder)*;
                    #inner_assertion
                }
            } else {
                quote! {
                    for disbanded in [#(#civil_disorder),*] {
                        #inner_assertion
                    }
                }
            };

            tokens.append_all(disband_check);
        } else {
            tokens.append_all(judge_call);
        }
    }
}

impl ToTokens for TestCaseBody {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            TestCaseBody::Main(main) => main.to_tokens(tokens),
            TestCaseBody::Retreat(retreat) => retreat.to_tokens(tokens),
            TestCaseBody::Build(build) => build.to_tokens(tokens),
        }
    }
}

impl ToTokens for TestCase {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { info, body } = self;
        let name = format_ident!("{}", info.name.as_deref().unwrap_or("unnamed"));
        let url = info.url.as_ref().map(|u| quote!(#[doc = #u]));
        tokens.append_all(quote! {
            #url
            #[test]
            fn #name() {
                #body
            }
        });
    }
}

impl ToTokens for TestCaseTodo {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { info, todo } = self;
        let name = format_ident!("{}", info.name.as_deref().unwrap_or("unnamed"));
        let url = info.url.as_ref().map(|u| quote!(#[doc = #u]));
        tokens.append_all(quote! {
            #url
            #[test]
            #[ignore = #todo]
            fn #name() {
                todo!(#todo);
            }
        });
    }
}

impl ToTokens for RawTestCase {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            RawTestCase::Todo(case) => case.to_tokens(tokens),
            RawTestCase::Case(case) => case.to_tokens(tokens),
        }
    }
}
