use std::env;
use std::sync::Mutex;

use clap::Parser;
use discord::{Discord, DiscordHandler};
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4;
use serenity::all::{Context, Message};
use serenity::async_trait;
use anyhow::Result;
mod discord;

#[derive(Parser)]
enum Commands {
    Chat {
        message: String,
    },
    Init {
        description: String,
    }
}

#[tokio::main]
async fn main() {
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap().to_string();
    let handler = Handler::new(&openai_api_key);
    let discord_token = env::var("DISCORD_API_KEY").unwrap().to_string();
    let discord = Discord::new(&discord_token, handler);

    discord.start().await;
}

struct Handler {
    client: Client,
    description: Mutex<String>,
}

impl Handler {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::new(api_key.to_string()),
            description: Mutex::new("You are a funny bot in discord channel".to_string()),
        }
    }
}

#[async_trait]
impl DiscordHandler for Handler {
    async fn message(&self, ctx: &Context, msg: &Message) -> Result<()> {
        if !msg.mentions_me(&ctx.http).await? {
            return Ok(());
        }
        let args = shlex::split(&msg.content).unwrap();
        println!("{:?}", args);

        let command = Commands::try_parse_from(args)?;

        match command {
            Commands::Chat{ message } => {
                let req = ChatCompletionRequest::new(
                    GPT4.to_string(),
                    vec![
                        chat_completion::ChatCompletionMessage {
                            role: chat_completion::MessageRole::system,
                            content: chat_completion::Content::Text(self.description.lock().unwrap().clone()),
                            name: Some("ChatPete".into()),
                        },
                            chat_completion::ChatCompletionMessage {
                            role: chat_completion::MessageRole::user,
                            content: chat_completion::Content::Text(message.clone()),
                            name: msg.author.global_name.clone(),
                        }
                    ],
                );

                let result = self.client.chat_completion(req)?;
                let content = result.choices[0].message.content.clone();
                msg.channel_id.say(&ctx.http, content.unwrap()).await?;
            },
            Commands::Init { description } => {
                *self.description.lock().unwrap() = description;
                msg.channel_id.say(&ctx.http, "Ok.").await?;
            }
        }

        Ok(())
    }
}
