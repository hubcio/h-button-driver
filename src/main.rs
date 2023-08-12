mod app; // glue code between bluetooth, sound and tray

mod ble; // bluetooth related code
mod sound; // sound related code
mod tray; // tray related code

use app::App;
use std::error::Error;

extern crate pretty_env_logger;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    pretty_env_logger::init();
    let do_something = async {
        println!("Initialized tokio runtime");
    };
    do_something.await;
    tauri::async_runtime::set(tokio::runtime::Handle::current());

    let mut core = App::new();
    core.run().await?;
    Ok(())
}
