extern crate embed_resource;
extern crate winres;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(target_os = "windows") {
        println!("cargo:info=Setting icon using embed-resource...");
        embed_resource::compile("../../assets/icon.rc");
        println!("cargo:info=Icon embedded successfully.");
    }

    Ok(())
}
