#![allow(dead_code)]
#[macro_use]
extern crate dotenv_codegen;
extern crate dotenv;

use chrono::{NaiveDate, Utc};
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

fn df_to_markdown(df: &DataFrame, country: &str, n: usize) -> String {
    let mut result = String::new();

    // Discord-friendly header
    result.push_str(&format!("## 🎮 Latest Games from: {}\n\n", country));

    for row in 0..n.min(df.height()) {
        let name = df.column("Name").unwrap().get(row).unwrap();
        let date = df.column("Release_Date").unwrap().get(row).unwrap();
        let link = df.column("Steam_Link").unwrap().get(row).unwrap();

        // Discord-friendly formatting with emojis and clear structure
        result.push_str(&format!("🎯 **{}**\n", name.to_string().trim_matches('"')));
        result.push_str(&format!("📅 Released: {}\n", date));
        result.push_str(&format!("🔗 <{}>\n", link.to_string().trim_matches('"')));
        result.push_str("\n"); // Extra spacing between games
    }

    result
}

fn sort_df(df: &mut DataFrame) -> DataFrame {
    // and delete unnanounced

    let newdf = df
        .with_column(Series::new(
            "Release_Date",
            series_to_naive(df.column("Release_Date").unwrap()),
        ))
        .unwrap();
    let df = newdf.sort(["Release_Date"], true, false).unwrap();

    let df = &df
        .lazy()
        .filter(col("Release_Date").neq(lit(NaiveDate::from_ymd_opt(1980, 1, 1).unwrap())))
        .collect()
        .unwrap();

    return df.clone();
}

/// Latest 5 game releases of a given country
#[poise::command(slash_command, prefix_command)]
async fn latest5(
    ctx: Context<'_>,
    #[description = "Latest 5 releases of the country:"] country: String,
) -> Result<(), Error> {
    match get_country_df(&country) {
        Ok(mut df) => {
            if df.height() == 0 {
                ctx.say("Sorry, we don't have enough data :(").await?;
            } else {
                let df = sort_df(&mut df);

                let now = Utc::now().naive_utc().date(); // UTC version

                let df = &df
                    .lazy()
                    .filter(col("Release_Date").lt(lit(now)))
                    .collect()
                    .unwrap();

                let md = df_to_markdown(&df, &country, 5);
                ctx.say(md).await?;
            }
        }
        Err(_) => {
            ctx.say("Error, country not found").await?;
        }
    }

    Ok(())
}

/// Latest 3 game releases of a given country
#[poise::command(slash_command, prefix_command)]
async fn latest3(
    ctx: Context<'_>,
    #[description = "Latest 3 releases of the country:"] country: String,
) -> Result<(), Error> {
    match get_country_df(&country) {
        Ok(mut df) => {
            if df.height() == 0 {
                ctx.say("Sorry, we didn't find any games :(").await?;
            } else {
                let df = sort_df(&mut df);

                let now = Utc::now().naive_utc().date(); // UTC version

                let df = &df
                    .lazy()
                    .filter(col("Release_Date").lt(lit(now)))
                    .collect()
                    .unwrap();

                let md = df_to_markdown(&df, &country, 3);
                ctx.say(md).await?;
            }
        }
        Err(_) => {
            ctx.say("Error, country not found").await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let response = format!("# MODS WEKOS");
    ctx.say(response).await?;
    Ok(())
}

/// The next 5 game releases of a given country
#[poise::command(slash_command, prefix_command)]
async fn next3(
    ctx: Context<'_>,
    #[description = "Next 3 releases of the country:"] country: String,
) -> Result<(), Error> {
    let response = format!("# MODS WEKOS, NOT IMPLEMENTED YET");
    ctx.say(response).await?;
    Ok(())
}

/// The next 5 game releases of a given country
#[poise::command(slash_command, prefix_command)]
async fn next5(
    ctx: Context<'_>,
    #[description = "Next 5 releases of the country:"] country: String,
) -> Result<(), Error> {
    let response = format!("# MODS WEKOS, NOT IMPLEMENTED YET");
    ctx.say(response).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let token = dotenv!("DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![latest5(), latest3(), next5(),next3(),ping()],
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
