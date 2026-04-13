extern crate winres;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(target_os = "windows") {
        println!("cargo:info=Setting icon using embed-resource...");

        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("app.manifest");
        res.set_icon("../../assets/icon.ico");
        res.compile().unwrap();

        println!("cargo:info=Icon embedded successfully.");
    }

    Ok(())
}
