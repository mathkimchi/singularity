use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(ComposeComponents, attributes(component, tree_component, focused_component))]
pub fn compose_components_derive(input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let ast = syn::parse_macro_input!(tokens as DeriveInput);

    let struct_identitifier = ast.ident;
    let struct_: syn::DataStruct = match ast.data {
        syn::Data::Struct(data) => data,
        _ => panic!(),
    };

    let (focused_component, focused_component_type) = {
        let mut focused_component = None;

        for attr in &ast.attrs {
            if attr.path.is_ident("focused_component") {
                assert!(focused_component.is_none());
                focused_component = match attr.tokens.clone().into_iter().next().unwrap() {
                    proc_macro2::TokenTree::Group(group) => match &group.stream().into_iter().collect::<Vec<proc_macro2::TokenTree>>().as_slice() {
                        &[proc_macro2::TokenTree::Group(area_generator), proc_macro2::TokenTree::Punct(seperator), proc_macro2::TokenTree::Group(node_renderer)] => {
                            assert_eq!(seperator.as_char(), ',');
                            Some((area_generator.stream(), node_renderer.stream()))
                        }
                        _ => {
                            Some((group.stream(), quote! { usize }))
                        }
                    }
                    _ => panic!(),
                };
            }
        }

        focused_component.unwrap_or((quote! { self.focused_component }, quote! { usize }))
    };
    // [(component ident, container size, focus id)]
    let components = {
        let mut components = vec![];
        for field in struct_.fields.iter() {
            for attr in field.attrs.iter() {
                if attr.path.is_ident("component") {
                    // just get the first thing from attr.tokens
                    let (container_size, focus_id) = match attr.tokens.clone().into_iter().next().unwrap() {
                        proc_macro2::TokenTree::Group(group) => match &group.stream().into_iter().collect::<Vec<proc_macro2::TokenTree>>().as_slice() {
                            &[proc_macro2::TokenTree::Group(container_size), proc_macro2::TokenTree::Punct(seperator), proc_macro2::TokenTree::Group(focus_id)] => {
                                assert_eq!(seperator.as_char(), ',');
                                (container_size.stream(), focus_id.stream())
                            },
                            _ => panic!("component attribute unparsable"),
                        }
                        _ => panic!("component attribute unparsable (not a group)"),
                    };
                    components.push((field.ident.clone().unwrap(), container_size, focus_id));
                }

                if attr.path.is_ident("tree_component") {
                    // // just get the first thing from attr.tokens
                    // let container_size = match attr.tokens.clone().into_iter().next().unwrap() {
                    //     proc_macro2::TokenTree::Group(group) => {
                    //         // just get the inner of the group without the delimiters
                    //         group.stream()
                    //     }
                    //     _ => panic!(),
                    // };
                    // components.push((field.ident.clone().unwrap(), container_size));
                }
            }
        }
        components
    };
    // [(component ident, individual area generator, individual node renderer, focus_id)]
    let tree_components = {
        let mut tree_components = vec![];
        for field in struct_.fields.iter() {
            for attr in field.attrs.iter() {
                if attr.path.is_ident("tree_component") {
                    let (area_generator, seperator, focus_id) = 
                    match attr.tokens.clone().into_iter().next().unwrap() {
                        proc_macro2::TokenTree::Group(group) => match &group.stream().into_iter().collect::<Vec<proc_macro2::TokenTree>>().as_slice() {
                            &[proc_macro2::TokenTree::Group(area_generator), proc_macro2::TokenTree::Punct(seperator0), proc_macro2::TokenTree::Group(node_renderer), proc_macro2::TokenTree::Punct(seperator1), proc_macro2::TokenTree::Group(focus_id)] => {
                                assert_eq!(seperator0.as_char(), ',', "Delimiter between area_generator and node_renderer of tree_component should be ','");
                                assert_eq!(seperator1.as_char(), ',', "Delimiter between node_renderer and focus id of tree_component should be ','");
                                (area_generator.stream(), node_renderer.stream(), focus_id.stream())
                            },
                            _ => panic!("tree_component attributes could not be parsed (hint: group: {})", group),
                        }
                        _ => panic!("expected group as attribute for tree_component"),
                    };
                    tree_components.push((field.ident.clone().unwrap(), area_generator, seperator, focus_id));
                }
            }
        }
        tree_components
    };
    let render_components = {
        let mut render_components = quote! {};
        for (component_ident, container_size, _) in &components {
            render_components.extend(quote! {
                self.#component_ident.render().contain(#container_size),
            });
        }
        for (component_ident, area_generator, node_renderer, _) in &tree_components {
            render_components.extend(quote! {
                singularity_ui::ui_element::UIElement::Container(
                    __singularity_common::utils::tree::tree_node_path::TraversableTree::collect_paths_dfs(&self.#component_ident)
                        .iter()
                        .enumerate()
                        .map(|(__index, __path)| {
                            #node_renderer.contain(#area_generator)
                        })
                        .collect(),
                )
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
        let mut search_clicked = quote! {};
        for (component_ident, component_size, focus_id) in components.iter() {
            match_cases.extend(quote! { 
                #focus_id => if let Some(remapped_event) = singularity_common::components::remap_event(#component_size, event.clone()) {
                    self.#component_ident.handle_event(remapped_event);
                    return Ok(());
                }
            });

            search_clicked.extend(quote! {
                if singularity_common::components::remap_event(#component_size, event.clone()).is_some() {
                    Err(Some(#focus_id))
                } else 
            });
        }
        quote! {
            // try to forward to the focused component
            match #focused_component {
                #match_cases
                _ => {},
            }

            // if not returned, then it means it was a mouseclick not on the focused component
            // look if there was a component clicked (in order of first to last in struct def)
            #search_clicked

            { Err(None) }
        }
    };

    quote! {
        const _: () = {
            extern crate singularity_common as __singularity_common;
            #[automatically_derived]
            impl #struct_identitifier {
                pub fn render_components(&mut self) -> singularity_ui::ui_element::UIElement {
                    singularity_ui::ui_element::UIElement::Container(vec![
                        #render_components
                    ])
                }

                /// If there is a mouse click outside the focused component,
                /// returns the index of the first component that contains the mouse click
                /// without passing the mouse click to it.
                /// If there is a mouse click on no designated  area, then returns Err(None)
                /// If passing it is desired behavior, then set the focused index to that and then rerun this.
                pub fn forward_events_to_focused(&mut self, event: singularity_common::tab::packets::Event) -> Result<(), Option<#focused_component_type>> {
                    #forward_events_impl
                }
            }
        };
    }
    .into()
}
