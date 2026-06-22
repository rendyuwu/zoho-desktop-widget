fn main() {
    // Compile-time build secrets are baked via env! in source. Tell cargo to
    // rebuild when any of them changes, otherwise a stale value sticks across
    // builds on the same machine.
    println!("cargo:rerun-if-env-changed=ZOHO_WS_URL");
    println!("cargo:rerun-if-env-changed=LDAP_SERVER_URI");
    println!("cargo:rerun-if-env-changed=LDAP_BIND_TEMPLATE");
    println!("cargo:rerun-if-env-changed=LDAP_ALLOW_INSECURE");
    tauri_build::build()
}
