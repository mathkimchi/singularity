use crate::applet::Applet;

/// The Singularity Applet Runner (SAR) is kind of just a wrapper around the Singularity UI.
/// The reason I want to abstract the UI is to make the recursive applet runners easier.
///
/// Using Singularity UI requires the user to be active,
/// but using Applet Runner is passive.
pub struct AppletRunner<A: Applet> {
    applet: A,
}
impl<A: Applet> AppletRunner<A> {
    pub fn new(applet: A) -> Self {
        Self { applet }
    }

    /// Returns after the applet is closed.
    pub fn run(self) {
        todo!()
    }
}
