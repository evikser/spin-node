use spin_runtime::SpinNode;

pub fn install_tracing() {
    use tracing_subscriber::{fmt, prelude::*, registry, EnvFilter};

    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        "warn,spin_runtime,spin_primitives=debug,erc20,example_token,root,fibonacci=trace"
            .to_owned()
    });
    println!("RUST_LOG={}", filter);

    let main_layer = fmt::layer()
        .event_format(fmt::format().with_ansi(true))
        .with_filter(EnvFilter::from(filter));

    let registry = registry().with(main_layer);

    registry.init();
}

pub fn init_temp_node() -> SpinNode {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let db_path = std::env::temp_dir().join("spin_node").join(ts.to_string());
    let mut node = spin_runtime::SpinNode::new(String::from(db_path.to_str().unwrap()));

    node.init_genesis();

    node
}
