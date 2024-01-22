use gear::InstanceBuilder;

fn main() {
    let instance = InstanceBuilder::new().build();
    let devices = instance.enumerate_physical_device();
}
