fn main() {
    pkg_config::Config::new()
        .probe("opus")
        .expect("libopus not found via pkg-config; install libopus development files");
}
