#![allow(dead_code)]
#[macro_use]
extern crate dotenv_codegen;
extern crate dotenv;

use chrono::NaiveDate;
use poise::serenity_prelude as serenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

use polars::prelude::*;
use std::fs::File;

fn get_country_df(country: &str) -> Result<DataFrame, PolarsError> {
    // Specify the path to the CSV file
    let file_path = format!("../export/{}.csv", country);

    // Open the file
    let file = File::open(file_path)?;

    // Read the CSV file into a DataFrame
    let df = CsvReader::new(file)
        .infer_schema(None)
        .has_header(true)
        .with_delimiter(b';')
        .finish()?; // Propagate error instead of unwrapping

    Ok(df)
}

fn parse_date(date_str: &str) -> NaiveDate {
    let trimmed = date_str.trim().trim_matches('"');

    // Try multiple format patterns
    let formats = ["%d %b, %Y", "%-d %b, %Y", "%e %b, %Y"];

    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(trimmed, format) {
            return date;
        }
    }

    NaiveDate::from_ymd_opt(1980, 1, 1).unwrap()
}

fn series_to_naive(data: &Series) -> Vec<NaiveDate> {
    (0..data.len())
        .map(|i| {
            let date_str = data.get(i).unwrap().to_string();
            parse_date(&date_str)
        })
        .collect()
}

fn df_to_markdown(df: &DataFrame, country: &str) -> String {
    let mut result = String::new();

    // Discord-friendly header
    result.push_str(&format!("## 🎮 Latest Games from: {}\n\n", country));

    for row in 0..5.min(df.height()) {
        let name = df.column("Name").unwrap().get(row).unwrap();
        let date = df.column("Release_Date").unwrap().get(row).unwrap();
        let link = df.column("Steam_Link").unwrap().get(row).unwrap();

        // Discord-friendly formatting with emojis and clear structure
        result.push_str(&format!("🎯 **{}**\n", name));
        result.push_str(&format!("📅 Released: {}\n", date));
        result.push_str(&format!("🔗 {}\n", link));
        result.push_str("\n"); // Extra spacing between games
    }

    result
}

/// Get the next releases of a given country
#[poise::command(slash_command, prefix_command)]
async fn latest5(
    ctx: Context<'_>,
    #[description = "Latest 5 releases of the country:"] country: String,
) -> Result<(), Error> {
    match get_country_df(&country) {
        Ok(mut df) => {
            let b = series_to_naive(df.column("Release_Date")?);
            let df = df.with_column(Series::new("Release_Date", b))?;

            let df = df.sort(["Release_Date"], true, false)?;
            let md = df_to_markdown(&df, &country);
            ctx.say(md).await?;
        }
        Err(_) => {
            ctx.say("Error, country not found").await?;
        }
    }

    Ok(())
}

/// say something
#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let response = format!("# MODS WEKOS");
    ctx.say(response).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = dotenv!("DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![latest5(), ping()],
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
