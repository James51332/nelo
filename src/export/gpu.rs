//! Helper to retrieve GPU device and queue.

use wgpu::{Device, DeviceDescriptor, Instance, Queue, RequestAdapterOptions};

pub fn create() -> (Device, Queue) {
    // Create our wgpu instance.
    let instance = Instance::default();

    // Get our adapter to retrieve device and queue.
    let adapter_opts = RequestAdapterOptions::default();
    let adapter = pollster::block_on(instance.request_adapter(&adapter_opts))
        .expect("Failed to get GPU adapter");

    // Get and return the device and queue.
    let device_opts = DeviceDescriptor::default();
    pollster::block_on(adapter.request_device(&device_opts)).expect("Failed to get GPU device")
}
