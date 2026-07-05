use crate::nodular_applet::{NodularApplet, NodularRunnerHook};

pub trait CreatableNodularApplet<Args, Hook = Box<dyn NodularRunnerHook>>:
    NodularApplet + Sized
where
    // idk why this is needed, something something box I think
    Self: 'static,
{
    fn new(args: Args, hook: Hook) -> Self;

    fn get_initiator(args: Args) -> impl FnOnce(Hook) -> Self {
        |hook: Hook| Self::new(args, hook)
    }
    fn get_boxed_initiator(args: Args) -> impl FnOnce(Hook) -> Box<dyn NodularApplet> {
        |hook: Hook| Box::new(Self::new(args, hook))
    }
    // fn get_applet_spawner() -> AppletSpawner {
    //     struct EditorSpawner;
    //     impl AppletSpawnerTrait for EditorSpawner {
    //         fn create_initializer(&self, args: &[&str]) -> Option<NodularAppletInitializer> {
    //             let file_path = args.first()?;

    //             Some(RecursiveNodeApplet::boxed_get_boxed_initializer(
    //                 TextEditorApplet::get_boxed_initiator(file_path.to_string()),
    //             ))
    //         }

    //         fn duplicate(&self) -> AppletSpawner {
    //             Box::new(Self)
    //         }
    //     }
    //     Box::new(EditorSpawner)
    // }
}
