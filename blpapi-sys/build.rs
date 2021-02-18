use std::env;
use std::path::PathBuf;

const ENV_WARNING: &'static str = r#"Error while building blpapi-sys.

    Cannot find 'BLPAPI_ROOT' environment variable.

    You can download blpapi binaries from bloomberg at:
    https://www.bloomberg.com/professional/support/api-library/

    Once extracted, the BLPAPI_ROOT environment variable should point to the
    directory containing the extracted package.
"#;

fn main() {
    let blpapi_root_dir = PathBuf::from(env::var("BLPAPI_ROOT").expect(ENV_WARNING));

    let lib_dir = {
        let mut dir = blpapi_root_dir.clone();

        if cfg!(target_os = "windows") {
            dir.push("lib");
        } else if cfg!(target_os = "linux") {
            dir.push("Linux");
        } else if cfg!(target_os = "macos") {
            dir.push("Darwin");
        }

        dir.into_os_string().into_string().unwrap()
    };

    println!("cargo:rustc-link-search={}", lib_dir);
    println!("cargo:rustc-link-lib=blpapi3_64");

    let include_dir = {
        let mut dir = blpapi_root_dir.clone();
        dir.push("include");
        dir.into_os_string().into_string().unwrap()
    };

    // Dynamically build bindings.rs based on wrapper.h
    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_dir))
        .size_t_is_usize(true)
        .derive_default(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
