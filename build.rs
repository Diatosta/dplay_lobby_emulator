use std::env;

use winres::WindowsResource;

fn main() {
    slint_build::compile("ui/appwindow.slint").unwrap();

    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let result = WindowsResource::new()
            .set_icon("ui/icon.ico")
            .compile();

        if let Err(e) = result {
            eprintln!("Failed to compile resources: {}", e);
        }
    }
}
