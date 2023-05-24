use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "starting the bot.")]
    Start,
    #[command(description = "displays available commands.")]
    Help,
    #[command(description = "gets first page.")]
    First,
    #[command(description = "gets last page.")]
    Last,
    #[command(description = "shows jump-to menu.")]
    Jump,
}
