use image::{ImageReader, RgbaImage};
use singularity_common::sap::packets::StandardEvent;
use singularity_sttk::nodular_applet::{AppletContext, StandardApplet};
use singularity_sttk::nodular_applet::{
    AppletSpawner, AppletSpawnerTrait, NodularAppletInitializer,
};
use singularity_sttk::standard_keybinds::handle_standard_keybinds;
use sonamu_ui::display_units::DisplayContainerSize;
use sonamu_ui::ui_element::UITree;
use std::path::PathBuf;

pub struct ImageViewerApplet {
    image_path: PathBuf,
    _image: RgbaImage,

    focused: bool,

    hook: AppletContext,
}
impl ImageViewerApplet {
    pub fn new<P>(image_path: P, hook: AppletContext) -> Self
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
            _image: image,
            focused: true,
            hook,
        }
    }
    pub fn get_initiator<P>(file_path: P) -> impl FnOnce(AppletContext) -> Self
    where
        P: AsRef<std::path::Path>,
        PathBuf: From<P>,
    {
        |hook: AppletContext| Self::new(file_path, hook)
    }
    pub fn get_boxed_initiator<P>(image_path: P) -> NodularAppletInitializer
    where
        P: AsRef<std::path::Path> + Send + 'static,
        PathBuf: From<P>,
    {
        Box::new(|hook: AppletContext| Box::new(Self::new(image_path, hook)))
    }
    #[must_use]
    pub fn get_applet_spawner() -> AppletSpawner {
        struct ImageViewerSpawner;
        impl AppletSpawnerTrait for ImageViewerSpawner {
            fn create_initializer(&self, args: &[&str]) -> Option<NodularAppletInitializer> {
                let file_path = args.first()?;

                Some(ImageViewerApplet::get_boxed_initiator(
                    file_path.to_string(),
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
impl StandardApplet for ImageViewerApplet {
    fn handle_standard_event(&mut self, standard_event: StandardEvent) {
        match standard_event {
            StandardEvent::UIEvent(ui_event) => {
                if handle_standard_keybinds(&ui_event, &self.hook) {
                    return;
                }

                self.hook.damage_window();
                // self.hook.damage_treeview();
            }
            StandardEvent::FocusChanged(focus) => {
                self.focused = focus;
                println!("Yay focus {focus}!");

                self.hook.damage_window();
            }
            _ => {}
        }
    }

    fn get_window_standard_applet(&self, _container_size: DisplayContainerSize) -> UITree {
        // UIElement::Image(self.image.clone())
        todo!("Didn't think abt giving wgpu to applets, ImageViewer isn't used anyways")
    }

    fn get_treeview(&self) -> singularity_common::utils::tree::world_tree::WorldTree<String> {
        singularity_common::utils::tree::world_tree::WorldTree::Base(self.get_title())
    }
}
