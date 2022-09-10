use std::error::Error;

use aws_sdk_dynamodb::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = config::load_from_env().await?;
    let client = Client::new(&config.aws_sdk_config);
    let resp = client
        .scan()
        .table_name(config.dynamodb_table)
        .send()
        .await?;

    if let Some(item) = resp.items {
        dbg!(item);
    }
    Ok(())
}

mod config {
    use std::env;
    use std::error::Error;

    use aws_config::SdkConfig;

    pub struct Config {
        pub dynamodb_table: String,
        pub aws_sdk_config: SdkConfig,
    }

    pub async fn load_from_env() -> Result<Config, Box<dyn Error>> {
        let dynamodb_table =
            env::var("DYNAMODB_TABLE").expect("Missing 'DYNAMODB_TABLE' environment variable.");
        let aws_sdk_config = aws_config::load_from_env().await;
        Ok(Config {
            dynamodb_table,
            aws_sdk_config,
        })
    }
}
