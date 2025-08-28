use embed_resource;

fn main() {
    slint_build::compile("ui/manager.slint").unwrap();
    let _ = embed_resource::compile("tray.rc", embed_resource::NONE);

}