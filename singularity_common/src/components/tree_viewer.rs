// use super::Component;
// use crate::utils::tree::tree_node_path::{TraversableTree, TreeNodePath};
// use singularity_ui::{
//     display_units::{DisplayArea, DisplayCoord, DisplaySize},
//     ui_element::UIElement,
// };

// /// FIXME
// pub fn basic_tree_area_generator(index: usize, path: &TreeNodePath) -> DisplayArea {
//     DisplayArea::from_corner_size(
//         DisplayCoord::new(
//             (path.depth() as f32 * 0.2).into(),
//             (index as f32 * 0.2).into(),
//         ),
//         DisplaySize::new(0.5.into(), 0.2.into()),
//     )
// }

// /// TODO: inputs
// pub struct TreeViewer<'a, Tree: TraversableTree> {
//     tree: Tree,
//     area_generator: &'a mut dyn FnMut(usize, &TreeNodePath) -> DisplayArea,
//     individual_renderer: &'a mut dyn FnMut(usize, &TreeNodePath) -> UIElement,
// }
// impl<'a, Tree: TraversableTree> Component for TreeViewer<'a, Tree>
// where
//     Self: Send,
// {
//     fn render(&mut self) -> singularity_ui::ui_element::UIElement {
//         UIElement::Container(
//             self.tree
//                 .iter_paths_dfs()
//                 .enumerate()
//                 .map(|(index, path)| {
//                     (self.individual_renderer)(index, &path)
//                         .contain((self.area_generator)(index, &path))
//                 })
//                 .collect(),
//         )
//     }

//     fn handle_event(&mut self, _event: crate::tab::packets::Event) {}
// }
