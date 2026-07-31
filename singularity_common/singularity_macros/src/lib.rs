use std::{
    hash::{Hash, Hasher},
    str::FromStr,
};

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{DeriveInput, Fields};

/// REVIEW: I realize that this is actually very arbitrary and unflexible for most use-cases
/// I mean, I am starting to feel that I don't even need a macro
/// I could also make an attribute macro that can be applied to each component, which would increase redundancy but make it suitable for a wider variety of usage
#[deprecated = "look at DEVLOG 2024/11/30"]
#[proc_macro_derive(
    ComposeComponents,
    attributes(component, tree_component, focused_component)
)]
pub fn compose_components_derive(input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let ast = syn::parse_macro_input!(tokens as DeriveInput);

    let struct_identifier = ast.ident;
    let struct_: syn::DataStruct = match ast.data {
        syn::Data::Struct(data) => data,
        _ => panic!(),
    };

    let (focused_component, focused_component_type) = {
        let mut focused_component = None;

        for attr in &ast.attrs {
            if attr.path().is_ident("focused_component") {
                assert!(focused_component.is_none());
                focused_component = match attr.to_token_stream().clone().into_iter().next().unwrap()
                {
                    proc_macro2::TokenTree::Group(group) => match &group
                        .stream()
                        .into_iter()
                        .collect::<Vec<proc_macro2::TokenTree>>()
                        .as_slice()
                    {
                        &[
                            proc_macro2::TokenTree::Group(area_generator),
                            proc_macro2::TokenTree::Punct(seperator),
                            proc_macro2::TokenTree::Group(node_renderer),
                        ] => {
                            assert_eq!(seperator.as_char(), ',');
                            Some((area_generator.stream(), node_renderer.stream()))
                        }
                        _ => Some((group.stream(), quote! { usize })),
                    },
                    _ => panic!(),
                };
            }
        }

        focused_component.unwrap_or((quote! { self.focused_component }, quote! { usize }))
    };
    // TODO: better argument parsing

    // [(component ident, container size, focus id)]
    let components = {
        let mut components = vec![];
        for field in &struct_.fields {
            for attr in &field.attrs {
                if attr.path().is_ident("component") {
                    // just get the first thing from attr.tokens
                    let (container_size, focus_id) =
                        match attr.to_token_stream().clone().into_iter().next().unwrap() {
                            proc_macro2::TokenTree::Group(group) => match &group
                                .stream()
                                .into_iter()
                                .collect::<Vec<proc_macro2::TokenTree>>()
                                .as_slice()
                            {
                                &[
                                    proc_macro2::TokenTree::Group(container_size),
                                    proc_macro2::TokenTree::Punct(seperator),
                                    proc_macro2::TokenTree::Group(focus_id),
                                ] => {
                                    assert_eq!(seperator.as_char(), ',');
                                    (container_size.stream(), focus_id.stream())
                                }
                                _ => panic!("component attribute unparsable"),
                            },
                            _ => panic!("component attribute unparsable (not a group)"),
                        };
                    components.push((field.ident.clone().unwrap(), container_size, focus_id));
                }

                if attr.path().is_ident("tree_component") {
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
    // [(component ident, individual area generator, individual node renderer, individual event handler, focus_id)]
    let tree_components = {
        let mut tree_components = vec![];
        for field in &struct_.fields {
            for attr in &field.attrs {
                if attr.path().is_ident("tree_component") {
                    let (area_generator, renderer, event_handler, focus_id) = match attr
                        .to_token_stream()
                        .clone()
                        .into_iter()
                        .next()
                        .unwrap()
                    {
                        proc_macro2::TokenTree::Group(group) => match &group
                            .stream()
                            .into_iter()
                            .collect::<Vec<proc_macro2::TokenTree>>()
                            .as_slice()
                        {
                            &[
                                proc_macro2::TokenTree::Group(area_generator),
                                proc_macro2::TokenTree::Punct(seperator0),
                                proc_macro2::TokenTree::Group(node_renderer),
                                proc_macro2::TokenTree::Punct(seperator1),
                                proc_macro2::TokenTree::Group(event_handler),
                                proc_macro2::TokenTree::Punct(seperator2),
                                proc_macro2::TokenTree::Group(focus_id),
                            ] => {
                                assert_eq!(
                                    seperator0.as_char(),
                                    ',',
                                    "Delimiter between area_generator and node_renderer of tree_component should be ','"
                                );
                                assert_eq!(
                                    seperator1.as_char(),
                                    ',',
                                    "Delimiter between node_renderer and event handler of tree_component should be ','"
                                );
                                assert_eq!(
                                    seperator2.as_char(),
                                    ',',
                                    "Delimiter between event handler and focus id of tree_component should be ','"
                                );
                                (
                                    area_generator.stream(),
                                    node_renderer.stream(),
                                    event_handler.stream(),
                                    focus_id.stream(),
                                )
                            }
                            e => panic!(
                                "tree_component attributes could not be parsed (hint: group: `{}` and e.len(): `{}`)",
                                group,
                                e.len()
                            ),
                        },
                        _ => panic!("expected group as attribute for tree_component"),
                    };
                    tree_components.push((
                        field.ident.clone().unwrap(),
                        area_generator,
                        renderer,
                        event_handler,
                        focus_id,
                    ));
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
        for (component_ident, area_generator, node_renderer, _, _) in &tree_components {
            render_components.extend(quote! {
                sonamu_ui::ui_element::UIElement::Container(
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
    let forward_events_impl = {
        let mut match_cases = quote! {};
        let mut search_clicked = quote! {};
        for (component_ident, component_size, focus_id) in &components {
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
        for (component_ident, area_generator, _, event_handler, focus_id) in &tree_components {
            match_cases.extend(quote! {
                #focus_id => {
                    for (__index, __path) in __singularity_common::utils::tree::tree_node_path::TraversableTree::collect_paths_dfs(&self.#component_ident)
                        .iter()
                        .enumerate() {
                        // FIXME: only works for clicks, for other events, it just forwards to the first element
                        if let Some(remapped_event) = singularity_common::components::remap_event(#area_generator, event.clone()) {
                            let __event = remapped_event;
                            #event_handler;
                            return Ok(());
                        }
                    }
                }
            });

            search_clicked.extend(quote! {
                if __singularity_common::utils::tree::tree_node_path::TraversableTree::collect_paths_dfs(&self.#component_ident)
                        .iter()
                        .enumerate()
                        .any(|(__index, __path)| {
                            singularity_common::components::remap_event(#area_generator, event.clone()).is_some()
                        }) {
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
            impl #struct_identifier {
                pub fn render_components(&mut self) -> sonamu_ui::ui_element::UIElement {
                    sonamu_ui::ui_element::UIElement::Container(vec![
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

/// (to_data_impl, try_from_data_impl)
fn packet_union_impls(
    data_enum: syn::DataEnum,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    // [(variant name, type), ...]
    let (packets, packet_unions) = {
        let mut packets = Vec::new();
        let mut packet_unions = Vec::new();

        for variant in &data_enum.variants {
            let inner_packet_type = match &variant.fields {
                Fields::Unnamed(fields_unnamed) => {
                    assert_eq!(
                        fields_unnamed.unnamed.len(),
                        1,
                        "variants should have exactly 1 unnamed field"
                    );
                    fields_unnamed.unnamed.first().unwrap().clone()
                }
                _ => panic!("Expected unnamed fields for all variants"),
            };

            if variant
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("sub_union"))
            {
                packet_unions.push((variant.ident.clone(), inner_packet_type));
            } else {
                packets.push((variant.ident.clone(), inner_packet_type));
            }
        }

        (packets, packet_unions)
    };

    let to_data_impl = {
        let to_data_match_packets_cases: proc_macro2::TokenStream = packets.iter().map(|(ident, inner_type)|
            quote! {
                Self::#ident(inner_packet) => (<#inner_type as PacketTrait>::PACKET_TYPE_ID, inner_packet.to_data()),
            }
        ).collect();
        let to_data_match_packet_unions_cases: proc_macro2::TokenStream = packet_unions
            .iter()
            .map(|(ident, _)| {
                quote! {
                    Self::#ident(inner_packet_union) => inner_packet_union.packet_to_data(),
                }
            })
            .collect();

        quote! {
            match self {
                // $(Self::$subevent(subevent) => ($subevent::PACKET_TYPE_ID, subevent.to_data()),)*
                #to_data_match_packets_cases
                #to_data_match_packet_unions_cases
            }
        }
    };

    let try_from_data_impl = {
        let try_from_data_match_packets_cases: proc_macro2::TokenStream = packets.iter().map(|(ident, inner_type)|
            quote! {
                <#inner_type as PacketTrait>::PACKET_TYPE_ID => Some(Self::#ident(#inner_type::try_from_data(packet_inner_data)?)),
            }
        ).collect();
        let try_from_data_try_packet_unions: proc_macro2::TokenStream = packet_unions.iter().map(|(variant_ident, inner_packet_union_type)|
            // REVIEW: look for ways to metacommunicate between `PacketUnion` derives
            quote! {
                if let Some(inner_packet_union) = #inner_packet_union_type::packet_try_from_data(packet_id, packet_inner_data) {
                    return Some(Self::#variant_ident(inner_packet_union));
                }
            }
        ).collect();

        quote! {
            match packet_id {
                // $($subevent::PACKET_TYPE_ID => Some(Self::$subevent($subevent::try_from_data(data)?)),)*
                #try_from_data_match_packets_cases
                _ => {
                    #try_from_data_try_packet_unions
                    None
                }
            }
        }
    };

    (to_data_impl, try_from_data_impl)
}

/// (to_data_impl, try_from_data_impl)
fn enum_datable_derive(
    data_enum: syn::DataEnum,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    // [(name, type), ...]
    let variants: Vec<_> = data_enum
        .variants
        .iter()
        .map(|variant| {
            let inner_packet_type = match &variant.fields {
                Fields::Unnamed(fields_unnamed) => {
                    assert_eq!(
                        fields_unnamed.unnamed.len(),
                        1,
                        "variants with more than 1 unnamed field not implemented"
                    );
                    fields_unnamed.unnamed.first().unwrap().clone()
                }
                _ => todo!(),
            };

            (variant.ident.clone(), inner_packet_type)
        })
        .collect();

    let try_from_data_match_cases: proc_macro2::TokenStream = variants.iter().enumerate().map(|(variant_num, (ident, inner_type))|
        quote! {
            #variant_num => Some(Self::#ident(<#inner_type as TryFromData>::try_from_data(inner_data)?)),
        }
    ).collect();

    let to_data_match_cases: proc_macro2::TokenStream = variants
        .iter()
        .enumerate()
        .map(|(variant_num, (ident, _inner_type))| {
            quote! {
                Self::#ident(inner_packet) => (#variant_num, inner_packet.to_data()),
            }
        })
        .collect();

    let to_data_impl = quote! {
        let (id, inner_data) = match self {
            // $(Self::$subevent(subevent) => ($subevent::PACKET_TYPE_ID, subevent.to_data()),)*
            #to_data_match_cases
        };

        let id_bytes: &[u8] = &id.to_be_bytes();
        [id_bytes, &inner_data].concat()
    };

    let try_from_data_impl = quote! {
        let (id_bytes, inner_data) = data.split_at((usize::BITS / 8) as usize);
        let id = usize::from_be_bytes(id_bytes.try_into().unwrap());

        match id {
            // $($subevent::PACKET_TYPE_ID => Some(Self::$subevent($subevent::try_from_data(data)?)),)*
            #try_from_data_match_cases
            _ => None,
        }
    };

    (to_data_impl, try_from_data_impl)
}

/// (to_data_impl, try_from_data_impl)
///
/// The number of fields is known and constant, and everything can just be ordered.
///
/// Before each field data, we give the size of that field.
/// This is kind of inefficient though if we have a lot of constant size fields.
/// Figure that out later.
/// REVIEW: could have try from data return how much of the data was used.
fn struct_datable_derive(
    data_struct: syn::DataStruct,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut is_tuple_struct = false;

    // [(name, type), ...]
    let fields: Vec<_> = data_struct
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let identifier = match &field.ident {
                Some(ident) => quote! { #ident },
                None => {
                    is_tuple_struct = true;
                    proc_macro2::TokenStream::from_str(&index.to_string()).unwrap()
                }
            };

            (identifier, field.ty.clone())
        })
        .collect();

    if fields.is_empty() {
        let to_data_impl = quote! {
            Vec::new()
        };

        let try_from_data_impl = quote! {
            if data.len() != 0 {
                return None;
            }

            Some(Self)
        };

        return (to_data_impl, try_from_data_impl);
    }

    let define_field_data: proc_macro2::TokenStream = fields
        .iter()
        .map(|(field_ident, _ty)| {
            // REVIEW: figure out what call site actually is
            // Having "bytes_..." is better than "..._bytes" because it also works with "bytes_0" for tuple structs
            let data_bytes_ident = proc_macro2::Ident::new(
                &format!("bytes_{field_ident}"),
                proc_macro2::Span::call_site(),
            );
            let data_len_ident = proc_macro2::Ident::new(
                &format!("len_{field_ident}"),
                proc_macro2::Span::call_site(),
            );
            quote! {
                let #data_bytes_ident = self.#field_ident.to_data();
                // TODO: make this le bytes
                let #data_len_ident = #data_bytes_ident.len().to_be_bytes();
            }
        })
        .collect();

    let combine_field_data: proc_macro2::TokenStream = fields
        .iter()
        .map(|(field_ident, _ty)| {
            // REVIEW: figure out what call site actually is
            // Having "bytes_..." is better than "..._bytes" because it also works with "bytes_0" for tuple structs
            let data_bytes_ident = proc_macro2::Ident::new(
                &format!("bytes_{field_ident}"),
                proc_macro2::Span::call_site(),
            );
            let data_len_ident = proc_macro2::Ident::new(
                &format!("len_{field_ident}"),
                proc_macro2::Span::call_site(),
            );
            quote! {
                #data_len_ident.as_slice(),
                #data_bytes_ident.as_slice(),
            }
        })
        .collect();

    let to_data_impl = quote! {
        #define_field_data

        [
            #combine_field_data
        ].concat()
    };

    let self_constructor =
        if is_tuple_struct {
            let fields: proc_macro2::TokenStream = fields.iter().map(|(_ident, ty)| {
            quote! {
                {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;

                    let inner_data = &data[index..(index + len)];
                    index += len;

                    <#ty>::try_from_data(inner_data)?
                },
            }
        }).collect();
            quote! {
                Self(#fields)
            }
        } else {
            let fields: proc_macro2::TokenStream = fields.iter().map(|(ident, ty)| {
            quote! {
                #ident: {
                    let len = usize::from_be_bytes(data[index..(index + 8)].try_into().ok()?);
                    index += 8;

                    let inner_data = &data[index..(index + len)];
                    index += len;

                    <#ty>::try_from_data(inner_data)?
                },
            }
        }).collect();

            quote! {
                Self {
                    #fields
                }
            }
        };

    let try_from_data_impl = quote! {
        let mut index = 0;
        let constructed_self = #self_constructor;

        if index != data.len() {
            return None;
        }

        Some(constructed_self)
    };

    (to_data_impl, try_from_data_impl)
}

#[proc_macro_derive(Datable)]
pub fn datable_derive(input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let ast = syn::parse_macro_input!(tokens as DeriveInput);

    let identifier = ast.ident;
    let (to_data_impl, try_from_data_impl) = match ast.data {
        syn::Data::Enum(data_enum) => enum_datable_derive(data_enum),
        syn::Data::Struct(data_struct) => struct_datable_derive(data_struct),
        syn::Data::Union(_data_union) => unimplemented!(),
    };

    quote! {
        const _: () = {
            // extern crate singularity_common as __singularity_common;
            // extern crate singularity_sap as __singularity_sap;
            // use crate as __singularity_sap; // FIXME: this only works for singularity sap itself
            // FIXME: the imports

            #[automatically_derived]
            impl ToData for #identifier {
                fn to_data(&self) -> Vec<u8> {
                    #to_data_impl
                }
            }
            #[automatically_derived]
            impl TryFromData for #identifier {
                fn try_from_data(data: &[u8]) -> Option<Self> {
                    #try_from_data_impl
                }
            }
        };
    }
    .into()
}

#[proc_macro_derive(Packet)]
pub fn packet_derive(input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let ast = syn::parse_macro_input!(tokens as DeriveInput);

    let identifier = ast.ident;
    let packet_type_id = {
        let mut hasher = std::hash::DefaultHasher::new();
        // TODO: hash `module_path!()`
        identifier.hash(&mut hasher);
        input.to_string().hash(&mut hasher);
        // TODO: figure out circular imports or wait till rust allows proc macros in normal crates,
        // to do something like: `hash as singularity_common::sap::packet::PacketTypeId`.
        // right now, they are luckily the same
        hasher.finish()
    };

    quote! {
        const _: () = {
            // extern crate singularity_common as __singularity_common;
            // extern crate singularity_sap as __singularity_sap;
            // use crate as __singularity_sap; // FIXME: this only works for singularity sap itself
            // FIXME: the imports

            #[automatically_derived]
            impl PacketTrait for #identifier {
                const PACKET_TYPE_ID: PacketTypeId = #packet_type_id;
            }
        };
    }
    .into()
}

#[proc_macro_derive(Event)]
pub fn event_derive(input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let ast = syn::parse_macro_input!(tokens as DeriveInput);

    let identifier = ast.ident;

    quote! {
        const _: () = {
            // extern crate singularity_common as __singularity_common;
            // extern crate singularity_sap as __singularity_sap;
            // use crate as __singularity_sap; // FIXME: this only works for singularity sap itself
            // FIXME: the imports

            #[automatically_derived]
            impl EventPacketTrait for #identifier {}
        };
    }
    .into()
}

#[proc_macro_derive(Request)]
pub fn request_derive(input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let ast = syn::parse_macro_input!(tokens as DeriveInput);

    let identifier = ast.ident;

    quote! {
        const _: () = {
            // extern crate singularity_common as __singularity_common;
            // extern crate singularity_sap as __singularity_sap;
            // use crate as __singularity_sap; // FIXME: this only works for singularity sap itself
            // FIXME: the imports

            #[automatically_derived]
            impl RequestPacketTrait for #identifier {}
        };
    }
    .into()
}

#[proc_macro_derive(Query, attributes(ResponseType))]
pub fn query_derive(input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let ast = syn::parse_macro_input!(tokens as DeriveInput);

    let identifier = ast.ident;
    let response_type = ast.attrs.iter().find_map(|attr| {
        if attr.path().is_ident("ResponseType") {
            // dbg!(&attr.tokens);
            // dbg!(&attr.tokens.clone().into_iter().next().unwrap());
            match attr.to_token_stream().clone().into_iter().next().unwrap() {
                proc_macro2::TokenTree::Group(group) => match &group.stream().into_iter().collect::<Vec<proc_macro2::TokenTree>>().as_slice() {
                    &[proc_macro2::TokenTree::Ident(response_type_ident)] => {
                        Some(response_type_ident.clone())
                    },
                    e => panic!("tree_component attributes could not be parsed (hint: group: `{}` and e.len(): `{}`)", group, e.len()),
                }
                _ => panic!("expected group as attribute for ResponseType attribute"),
            }
        } else {
            None
        }
    }).expect("`Query` macro should have one `ResponseType` attribute.");

    quote! {
        const _: () = {
            // extern crate singularity_common as __singularity_common;
            // extern crate singularity_sap as __singularity_sap;
            // use crate as __singularity_sap; // FIXME: this only works for singularity sap itself
            // FIXME: the imports

            #[automatically_derived]
            impl UniversalQueryTrait for #identifier {
                type ResponseType = #response_type;
            }
        };
    }
    .into()
}

#[proc_macro_derive(PacketUnion, attributes(sub_union))]
pub fn packet_union_derive(input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let ast = syn::parse_macro_input!(tokens as DeriveInput);

    let identifier = ast.ident;
    let (to_data_impl, try_from_data_impl) = match ast.data {
        syn::Data::Enum(data_enum) => packet_union_impls(data_enum),
        syn::Data::Struct(_data_struct) => panic!("PacketUnion must be used on an enum"),
        syn::Data::Union(_data_union) => todo!(), // TODO: I actually do want to see how Union type works
    };

    quote! {
        const _: () = {
            // extern crate singularity_common as __singularity_common;
            // extern crate singularity_sap as __singularity_sap;
            // use crate as __singularity_sap; // FIXME: this only works for singularity sap itself
            // FIXME: the imports

            #[automatically_derived]
            impl PacketUnion for #identifier {
                fn packet_to_data(&self) -> (PacketTypeId, Vec<u8>) {
                    #to_data_impl
                }

                fn packet_try_from_data(packet_id: PacketTypeId, packet_inner_data: &[u8]) -> Option<Self> {
                    #try_from_data_impl
                }
            }
        };
    }
    .into()
}

/// TODO: test for `EventPacketTrait`. Could have the packet union impls take in a packet prefix, so it would use cast the variants into `EventPacketTrait` instead of `PacketTrait`
#[proc_macro_derive(EventPacketUnion, attributes(sub_union))]
pub fn event_packet_union_derive(input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let ast = syn::parse_macro_input!(tokens as DeriveInput);

    let identifier = ast.ident;

    let packet_union_derive = proc_macro2::TokenStream::from(packet_union_derive(input));

    quote! {
        const _: () = {
            // extern crate singularity_common as __singularity_common;
            // extern crate singularity_sap as __singularity_sap;
            // use crate as __singularity_sap; // FIXME: this only works for singularity sap itself
            // FIXME: the imports

            #packet_union_derive

            #[automatically_derived]
            impl EventPacketUnion for #identifier {}
        };
    }
    .into()
}

/// TODO: test for `RequestPacketUnion`
#[proc_macro_derive(RequestPacketUnion, attributes(sub_union))]
pub fn request_packet_union_derive(input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let ast = syn::parse_macro_input!(tokens as DeriveInput);

    let identifier = ast.ident;

    let packet_union_derive = proc_macro2::TokenStream::from(packet_union_derive(input));

    quote! {
        const _: () = {
            // extern crate singularity_common as __singularity_common;
            // extern crate singularity_sap as __singularity_sap;
            // use crate as __singularity_sap; // FIXME: this only works for singularity sap itself
            // FIXME: the imports

            #packet_union_derive

            #[automatically_derived]
            impl RequestPacketUnion for #identifier {}
        };
    }
    .into()
}
