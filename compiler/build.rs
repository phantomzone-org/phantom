// build.rs
fn main() {
    // Tells Cargo to rerun the build script if `default.x` changes.
    println!("cargo:rerun-if-changed=compiler/linker-script/default.x");
}
