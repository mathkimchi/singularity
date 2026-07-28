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
}
