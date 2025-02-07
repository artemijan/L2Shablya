fn main() {
    // Check if we're running tests
    let profile = std::env::var("PROFILE").unwrap_or_default();
    if profile == "debug" {
        println!("cargo:rustc-cfg=feature=\"test-factories\"");
    }
}
