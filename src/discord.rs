use anyhow::Result;
use std::sync::Arc;

use serenity::{all::{Context, EventHandler, GatewayIntents, Message, Ready}, async_trait, Client};

#[async_trait]
pub trait DiscordHandler: Send + Sync {
    async fn message(&self, ctx: &Context, msg: &Message) -> Result<()>;
}

pub struct Discord {
    token: String,
    handler: Arc<dyn DiscordHandler>,
}

impl Discord {
    pub fn new(token: &str, handler: impl DiscordHandler + 'static) -> Self {
        Self {
            token: token.to_string(),
            handler: Arc::new(handler),
        }
    }

    pub async fn start(self) {
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;
        let mut client = Client::builder(&self.token, intents)
            .event_handler(self)
            .await
            .expect("Err creating client");

        if let Err(why) = client.start().await {
            println!("Client error: {why:?}");
        }
    }
}

#[async_trait]
impl EventHandler for Discord {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.name == "ChatPete" {
            return;
        }
        if let Err(e) = self.handler.message(&ctx, &msg).await {
            println!("{}", e);
            if let Err(e) = msg.channel_id.say(&ctx.http, e.to_string()).await {
                println!("{}", e);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
