mod config;
mod connection;

use config::Config;

#[tokio::main]
async fn main() {
    let bot_config = match Config::from_config_folder() {
        Ok(config) => config,
        Err(err) => {
            println!("error parsing config: {:?}", err);
            std::process::exit(1);
        }
    };

    println!("{:?}", bot_config);
}
