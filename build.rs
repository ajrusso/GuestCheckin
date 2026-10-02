fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=assets/icon.ico");

        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile()
            .expect("failed to compile Windows resources (icon)");

        // winres emits cargo:rustc-link-lib, which Cargo only applies to the *library*
        // target when a package has both lib + bin. Pass the .res to binaries so the
        // desktop/exe icon actually appears on guest-checkin.exe.
        let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
        let res_file = std::path::Path::new(&out_dir).join("resource.res");
        if res_file.exists() {
            println!("cargo:rustc-link-arg-bins={}", res_file.display());
        } else {
            // Some winres/toolchain combos leave the .res under a sibling build hash.
            println!(
                "cargo:warning=resource.res not found at {}; skipping binary icon link",
                res_file.display()
            );
        }
    }
}
