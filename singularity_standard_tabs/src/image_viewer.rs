use image::{ImageReader, RgbaImage};
use singularity_common::utils::tree::world_tree::WorldTreePath;
use singularity_sar::applet::{BasicApplet, BasicRunnerHook};
use singularity_sttk::nodular_applet::recursive_node_applet::RecursiveNodeApplet;
use singularity_sttk::nodular_applet::{
    AppletSpawner, AppletSpawnerTrait, NodularAppletInitializer,
};
use singularity_sttk::nodular_applet::{NodularApplet, NodularRunnerHook};
use singularity_sttk::standard_keybinds::handle_standard_keybinds;
use sonamu_ui::display_units::DisplayContainerSize;
use sonamu_ui::{ui_element::UIElement, ui_event::UIEvent};
use std::path::PathBuf;

pub struct ImageViewerApplet {
    image_path: PathBuf,
    image: RgbaImage,

    focused: bool,

    hook: Box<dyn NodularRunnerHook>,
}
impl ImageViewerApplet {
    pub fn new<P>(image_path: P, hook: Box<dyn NodularRunnerHook>) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: From<P>,
    {
        let image = ImageReader::open(&image_path)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();

        Self {
            image_path: image_path.into(),
            image,
            focused: true,
            hook,
        }
    }
    pub fn get_initiator<P>(file_path: P) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: From<P>,
    {
        |hook: Box<dyn NodularRunnerHook>| Self::new(file_path, hook)
    }
    pub fn get_boxed_initiator<P>(
        image_path: P,
    ) -> impl FnOnce(Box<dyn NodularRunnerHook>) -> Box<dyn NodularApplet>
    where
        P: AsRef<std::path::Path>,
        PathBuf: From<P>,
    {
        |hook: Box<dyn NodularRunnerHook>| Box::new(Self::new(image_path, hook))
    }
    #[must_use]
    pub fn get_applet_spawner() -> AppletSpawner {
        struct ImageViewerSpawner;
        impl AppletSpawnerTrait for ImageViewerSpawner {
            fn create_initializer(&self, args: &[&str]) -> Option<NodularAppletInitializer> {
                let file_path = args.first()?;

                Some(RecursiveNodeApplet::boxed_get_boxed_initializer(
                    ImageViewerApplet::get_boxed_initiator(file_path.to_string()),
                ))
            }

            fn duplicate(&self) -> AppletSpawner {
                Box::new(Self)
            }
        }
        Box::new(ImageViewerSpawner)
    }

    fn get_title(&self) -> String {
        self.image_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}
impl BasicApplet for ImageViewerApplet {
    fn handle_ui_event(&mut self, ui_event: UIEvent) {
        if handle_standard_keybinds(&ui_event, &self.hook) {
            return;
        }

        self.hook.damage_window();
        // self.hook.damage_treeview();
    }

    fn get_window(&self, _container_size: DisplayContainerSize) -> UIElement {
        UIElement::Image(self.image.clone())
    }
}
impl NodularApplet for ImageViewerApplet {
    fn handle_nodular_event(
        &mut self,
        nodular_event: singularity_sttk::nodular_applet::NodularEvent,
    ) {
        match nodular_event {
            singularity_sttk::nodular_applet::NodularEvent::Highlighted(_) => todo!(),
            singularity_sttk::nodular_applet::NodularEvent::Focused(focus) => {
                self.focused = focus;
                println!("Yay focus {focus}!");

                self.hook.damage_window();
            }
        }
    }

    fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
        singularity_common::utils::tree::world_tree::WorldTree::Base(self.get_title())
    }

    fn get_focus_path(&self) -> WorldTreePath {
        // WorldTreePath::new_empty()
        WorldTreePath::new_into()
    }
}
