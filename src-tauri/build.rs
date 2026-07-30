fn main() {
    let _ = std::fs::copy("../.env", ".env");
    println!("cargo:rerun-if-changed=../.env");
    tauri_build::build();
}
