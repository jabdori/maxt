fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        return;
    }

    for name in [
        "DEBUG_GENERATED_CODE",
        "TYPE_DEF_TMP_PATH",
        "CARGO_CFG_NAPI_RS_CLI_VERSION",
        "NAPI_DEBUG_GENERATED_CODE",
        "NAPI_TYPE_DEF_TMP_FOLDER",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    println!(
        "cargo:rerun-if-env-changed=NAPI_FORCE_BUILD_{}",
        std::env::var("CARGO_PKG_NAME")
            .expect("package name")
            .to_uppercase()
            .replace('-', "_")
    );

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("target OS");
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").expect("target environment");
    if target_os == "macos" {
        println!("cargo:rustc-cdylib-link-arg=-Wl");
        println!("cargo:rustc-cdylib-link-arg=-undefined");
        println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
    }
    if (target_env == "gnu" && target_os != "windows")
        || matches!(target_os.as_str(), "freebsd" | "openbsd")
    {
        println!("cargo:rustc-link-arg=-Wl,-z,nodelete");
    }
}
