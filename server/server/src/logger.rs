use env_logger::{Builder, Target};

pub fn init_logger() {
    let mut builder = Builder::from_default_env();
    builder.target(Target::Stdout);
    builder.init();
}
