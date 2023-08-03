mod core; // glue code between bluetooth, sound and tray

mod ble; // bluetooth related code
mod sound; // sound related code
mod tray; // tray related code

use std::error::Error;

extern crate pretty_env_logger;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    pretty_env_logger::init();
    core::run().await?;
    Ok(())
}
