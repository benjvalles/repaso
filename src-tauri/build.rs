fn main() {
    let _ = std::fs::copy("../.env", ".env");
    println!("cargo:rerun-if-changed=../.env");

    // Leer variables del .env y exportarlas como variables de entorno para env!()
    let vars = ["SHARED_SECRETS", "PROXY_BASEROW_URL", "PROXY_BREVO_URL"];
    if let Ok(content) = std::fs::read_to_string(".env") {
        for line in content.lines() {
            let line = line.trim();
            for var in &vars {
                if let Some(val) = line.strip_prefix(&format!("{var}=")) {
                    let val = val.trim();
                    if !val.is_empty() {
                        println!("cargo:rustc-env={var}={val}");
                    }
                }
            }
        }
    }

    tauri_build::build();
}
