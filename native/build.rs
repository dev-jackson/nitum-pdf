fn main() {
    let include_debug_info = std::env::var("PROFILE").is_ok_and(|profile| profile != "release");
    let config = slint_build::CompilerConfiguration::new().with_debug_info(include_debug_info);
    slint_build::compile_with_config("ui/app.slint", config).expect("failed to compile Slint UI");
    #[cfg(target_os = "windows")]
    winresource::WindowsResource::new()
        .set_icon("../packaging/native/nitum-pdf.ico")
        .set("ProductName", "Nitum PDF")
        .set("FileDescription", "Nitum PDF")
        .set("LegalCopyright", "Copyright © Nitum contributors")
        .compile()
        .expect("failed to compile Windows resources");
}
