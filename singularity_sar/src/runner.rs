use crate::applet::BasicApplet;

/// The Singularity Applet Runner (SAR) is kind of just a wrapper around the Singularity UI.
/// The reason I want to abstract the UI is to make the recursive applet runners easier.
///
/// Using Singularity UI requires the user to be active,
/// but using Applet Runner is passive.
pub struct AppletRunner<A: BasicApplet> {
    applet: A,
}
impl<A: BasicApplet> AppletRunner<A> {
    pub fn new(applet_initializing_data: A::InitializingData) -> Self {
        Self {
            applet: A::initialize(applet_initializing_data, todo!()),
        }
    }

    /// Returns after the applet is closed.
    pub fn run(self) {
        todo!()
    }
}
