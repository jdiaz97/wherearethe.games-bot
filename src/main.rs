#[macro_use]
extern crate dotenv_codegen;
extern crate dotenv;

use poise::serenity_prelude as serenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Get the 5 next releases of a given country
#[poise::command(slash_command, prefix_command)]
async fn next5(ctx: Context<'_>, #[description = "Next 5 releases of the country:"] country: String) -> Result<(), Error>  {
    let response = format!("Next 5 releases of the country: {}", country);
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
