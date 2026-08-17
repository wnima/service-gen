extern crate embed_resource;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(target_os = "windows") {
        println!("cargo:info=Embedding resources (icon)...");
        embed_resource::compile("resource.rc");

        let manifest_path = std::path::Path::new(
            &std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        )
        .join("app.manifest");

        println!(
            "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
            manifest_path.display()
        );
        // 禁止链接器添加默认 UAC 片段，否则与清单中的 requireAdministrator 冲突
        println!("cargo:rustc-link-arg=/MANIFESTUAC:NO");
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:info=Manifest configured.");
    }
    Ok(())
}
