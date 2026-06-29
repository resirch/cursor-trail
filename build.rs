fn main() {
    #[cfg(target_os = "windows")]
    {
        generate_windows_icon();
        if let Err(error) = winres::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .compile()
        {
            eprintln!("winres error: {error}");
        }
    }
}

#[cfg(target_os = "windows")]
fn generate_windows_icon() {
    use std::fs::File;
    use std::io::BufWriter;

    let png_bytes = std::fs::read("assets/icon.png").expect("assets/icon.png is missing");
    let image = image::load_from_memory(&png_bytes)
        .expect("failed to decode assets/icon.png")
        .to_rgba8();

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for size in [16u32, 32, 48, 256] {
        let resized = image::imageops::resize(
            &image,
            size,
            size,
            image::imageops::FilterType::Lanczos3,
        );
        let icon_image = ico::IconImage::from_rgba_data(size, size, resized.into_raw());
        let entry = ico::IconDirEntry::encode(&icon_image).expect("failed to encode icon entry");
        icon_dir.add_entry(entry);
    }

    let mut file = File::create("assets/icon.ico").expect("failed to create assets/icon.ico");
    icon_dir
        .write(BufWriter::new(&mut file))
        .expect("failed to write assets/icon.ico");

    println!("cargo:rerun-if-changed=assets/icon.png");
}
