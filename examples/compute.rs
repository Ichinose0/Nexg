use gear::InstanceBuilder;

fn main() {
    let instance = InstanceBuilder::new().build();
    let connecter = instance.default_connector();
    let device = connecter.create_device();
}
