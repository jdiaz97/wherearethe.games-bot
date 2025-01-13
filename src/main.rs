#[macro_use]
extern crate dotenv_codegen;
extern crate dotenv;

use poise::serenity_prelude as serenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

use polars::prelude::*;
use std::fs::File;

fn get_country_df(country: String) -> DataFrame {
    // Specify the path to the CSV file
    let file_path = format!("../export/{}.csv",country);

    // Open the file
    let file = File::open(file_path).unwrap();

    // Read the CSV file into a DataFrame
    let df = CsvReader::new(file)
        .infer_schema(None) // Automatically infer the schema
        .has_header(true)   // Assume the first row is a header
        .with_delimiter(b';') // Specify the delimiter as `;`
        .finish()         // Build the DataFrame
        .unwrap();

    return df
}

/// Get the 5 next releases of a given country
#[poise::command(slash_command, prefix_command)]
async fn next5(ctx: Context<'_>, #[description = "Next 5 releases of the country:"] country: String) -> Result<(), Error>  {
    let response = format!("Next 5 releases of the country: {}", country);
    ctx.say(response).await?;
    Ok(())
}

/// Get the next releases of a given country
#[poise::command(slash_command, prefix_command)]
async fn next(ctx: Context<'_>, #[description = "Next release of the country:"] country: String) -> Result<(), Error>  {
    let response = format!("Next release of the country: {}", country);
    ctx.say(response).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = dotenv!("DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![next5()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
