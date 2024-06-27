use std::env;
use log::{error, info};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use nat2pub::config::Config;
use nat2pub::service::nat_service::NatService;


#[tokio::main]
pub async fn main() -> anyhow::Result<()> {

    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let mut config_file = File::open("config.toml").await?;
    let mut str = String::new();
    config_file.read_to_string(&mut str).await?;
    let config: Config = toml::from_str(str.as_str())?;
    for svc in config.services {
        let service = NatService::new(svc).unwrap();
        match service.start().await {
            Ok(_) => {
                info!("handler_stream ok");
            }
            Err(e) => {
                error!("handler_stream error: {:?}", e);
            }
        };
    }
    //创建本地端口


    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
