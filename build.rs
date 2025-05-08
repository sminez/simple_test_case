fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=tests/test_data");
    println!("cargo::rerun-if-changed=tests/test_data_2");
}
