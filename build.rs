use std::{env, fs, path::Path};

// Obfuscation key - values are XOR'd before embedding so they don't appear
// as plaintext in `strings` output. Rotated at the byte level, not encryption.
const KEY: u8 = 0x5A;

fn xor(s: &str) -> Vec<u8> {
    s.bytes().map(|b| b ^ KEY).collect()
}

fn byte_array(name: &str, bytes: &[u8]) -> String {
    let lit = bytes
        .iter()
        .map(|b| format!("0x{b:02x}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("const {name}: &[u8] = &[{lit}];\n")
}

fn main() {
    for var in &[
        "CLISTRAP_TENANT_ID",
        "CLISTRAP_CLIENT_ID",
        "CLISTRAP_DOMAIN",
        "CLISTRAP_COMPANY",
    ] {
        println!("cargo:rerun-if-env-changed={var}");
    }

    let tenant_id = env::var("CLISTRAP_TENANT_ID").unwrap_or_default();
    let client_id = env::var("CLISTRAP_CLIENT_ID").unwrap_or_default();
    let domain = env::var("CLISTRAP_DOMAIN").unwrap_or_default();
    let company = env::var("CLISTRAP_COMPANY").unwrap_or_default();

    let baked = !tenant_id.is_empty() && !client_id.is_empty();

    let mut out = String::new();
    out.push_str(&format!("const BAKED_CONFIG: bool = {baked};\n"));
    out.push_str(&byte_array("BAKED_TENANT_ID", &xor(&tenant_id)));
    out.push_str(&byte_array("BAKED_CLIENT_ID", &xor(&client_id)));
    out.push_str(&byte_array("BAKED_DOMAIN", &xor(&domain)));
    out.push_str(&byte_array("BAKED_COMPANY", &xor(&company)));

    let out_dir = env::var("OUT_DIR").unwrap();
    fs::write(Path::new(&out_dir).join("baked_config.rs"), out).unwrap();
}
