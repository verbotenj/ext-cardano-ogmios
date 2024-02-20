use operator::controller;
use kube::CustomResourceExt;

fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&controller::OgmiosPort::crd()).unwrap()
    )
}
