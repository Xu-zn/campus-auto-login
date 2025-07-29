use embed_resource;
use embed_manifest::embed_manifest_file;

fn main() {
    let _ = embed_resource::compile("tray.rc", embed_resource::NONE);
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest_file("app.manifest")
            .expect("unable to embed manifest file");
    }
}