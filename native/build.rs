fn main() {
    slint_build::compile("ui/app.slint").expect("failed to compile Slint UI");
    #[cfg(target_os = "windows")]
    winresource::WindowsResource::new()
        .set_icon("../packaging/native/nitum-pdf.ico")
        .set("ProductName", "Nitum PDF")
        .set("FileDescription", "Nitum PDF")
        .set("LegalCopyright", "Copyright © Nitum contributors")
        .compile()
        .expect("failed to compile Windows resources");
}
