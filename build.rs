fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/icon.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon("assets/icon.ico")
        .set("ProductName", "XurPorisoy")
        .set("FileDescription", "XurPorisoy - audio metadata editor")
        .set("CompanyName", "XurPorisoy")
        .set("InternalName", "xur-porisoy")
        .set("LegalCopyright", "MIT licensed")
        .compile()
        .expect("failed to embed the Windows icon and version resource");
}
