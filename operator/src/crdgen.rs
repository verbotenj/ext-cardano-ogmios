use kube::CustomResourceExt;
use operator::controller;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "json" {
        print!(
            "{}",
            serde_json::to_string_pretty(&controller::OgmiosPort::crd()).unwrap()
        );
        return;
    }

    print!(
        "{}",
        serde_yaml::to_string(&controller::OgmiosPort::crd()).unwrap()
    )
}
