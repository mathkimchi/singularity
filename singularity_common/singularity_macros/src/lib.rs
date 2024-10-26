use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(ComposeComponents, attributes(component))]
pub fn compose_components_derive(input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let ast = syn::parse_macro_input!(tokens as DeriveInput);

    let struct_identitifier = ast.ident;
    let struct_: syn::DataStruct = match ast.data {
        syn::Data::Struct(data) => data,
        _ => panic!(),
    };

    // [(component ident, container size)]
    let components = {
        let mut components = vec![];
        for field in struct_.fields.iter() {
            for attr in field.attrs.iter() {
                if attr.path.is_ident("component") {
                    // just get the first thing from attr.tokens
                    let container_size = match attr.tokens.clone().into_iter().next().unwrap() {
                        proc_macro2::TokenTree::Group(group) => {
                            // just get the inner of the group without the delimiters
                            group.stream()
                        }
                        _ => panic!(),
                    };
                    components.push((field.ident.clone().unwrap(), container_size));
                }
            }
        }
        components
    };
    let render_components = {
        let mut render_components = quote! {};
        for (component_ident, container_size) in &components {
            render_components.extend(quote! {
                self.#component_ident.render().contain(#container_size),
            });
        }
        render_components
    };
    // let components_tuple = {
    //     let mut components_tuple = quote! {};
    //     for (component_ident, _) in &components {
    //         components_tuple.extend(quote! { #component_ident, });
    //     }
    //     quote! { ( #components_tuple ) }
    // };
    let forward_events_impl = {
        let mut match_cases = quote! {};
        for (index, (component_ident, component_size)) in components.iter().enumerate() {
            match_cases.extend(quote! { 
                #index => singularity_common::components::EnclosedComponent::forward_event(&mut self.#component_ident, #component_size, event), 
            });
        }
        quote! {
            match self.focused_component {
                #match_cases
                _ => panic!(),
            }
        }
    };

    quote! {
        #[automatically_derived]
        impl #struct_identitifier {
            pub fn render_components(&mut self) -> singularity_ui::ui_element::UIElement {
                singularity_ui::ui_element::UIElement::Container(vec![
                    #render_components
                ])
            }

            /// TODO: currently doesn't map mouse
            pub fn forward_events_to_focused(&mut self, event: singularity_common::tab::packets::Event) {
                #forward_events_impl
            }
        }
    }
    .into()
}
