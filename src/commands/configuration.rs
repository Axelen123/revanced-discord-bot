use tracing::debug;

use crate::utils::bot::load_configuration;
use crate::utils::role_embed::update_role_embed;
use crate::{Context, Error};

/// Reload the Discord bot.
#[poise::command(slash_command)]
pub async fn reload(ctx: Context<'_>) -> Result<(), Error> {
    // Update the configuration
    let configuration = load_configuration();
    // Use the embed color from the updated configuration
    let embed_color = configuration.general.embed_color;
    // Lock user data
    debug!("Locking data...");
    let mut data = ctx.data().write().await;
    // Also save the new configuration to the user data
    data.configuration = configuration;

    debug!("{} reloaded the configuration.", ctx.author().name);

    ctx.send(|f| {
        f.ephemeral(true).embed(|f| {
            f.description("Successfully reloaded configuration.")
                .color(embed_color)
        })
    })
    .await?;

    // Update the role embed.
    update_role_embed(ctx.discord(), &mut data).await?;

    Ok(())
}

/// Stop the Discord bot.
#[poise::command(slash_command)]
pub async fn stop(ctx: Context<'_>) -> Result<(), Error> {
    debug!("{} stopped the bot.", ctx.author().name);

    let color = ctx.data().read().await.configuration.general.embed_color;
    ctx.send(|f| {
        f.ephemeral(true)
            .embed(|f| f.description("Stopped the bot.").color(color))
    })
    .await?;

    ctx.framework()
        .shard_manager()
        .lock()
        .await
        .shutdown_all()
        .await;

    Ok(())
}

/// Register slash commands.
#[poise::command(prefix_command, slash_command, ephemeral = true)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}
