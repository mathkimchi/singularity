use singularity_common::subapp::{dynamic_loader::CreateSubappInterface, SubappInterface};

pub struct DemoSubappInterface;

impl SubappInterface for DemoSubappInterface {
    fn inform_event(&mut self, event: singularity_common::subapp::Event) {
        println!("Wowsers, an event!");
    }

    fn dump_requests(&mut self) -> Vec<singularity_common::subapp::Request> {
        vec![]
    }
}
impl Drop for DemoSubappInterface {
    fn drop(&mut self) {}
}

#[no_mangle]
pub extern "C" fn create_subapp_interface() -> *mut dyn SubappInterface {
    let subapp_interface = Box::new(DemoSubappInterface);

    Box::into_raw(subapp_interface)
}

// #[no_mangle]
// pub static create_subapp_interface: CreateSubappInterface = create_subapp_interface_fn;
