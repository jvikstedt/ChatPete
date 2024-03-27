use std::env;
use std::sync::Mutex;

use clap::Parser;
use discord::{Discord, DiscordHandler};
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4;
use openai_api_rs::v1::image::ImageGenerationRequest;
use serenity::all::{Context, CreateAttachment, CreateMessage, Message};
use serenity::async_trait;
use anyhow::Result;
use unidecode::unidecode;

mod discord;

#[derive(Parser)]
enum Commands {
    Chat {
        message: String,
    },
    Init {
        description: String,
    },
    Image {
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

        let author: Option<String> = msg.author.global_name.clone().map_or_else(|| None, |v| Some(unidecode(&v).chars().filter(|c| c.is_alphabetic()).collect()));

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
                            name: author,
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
            },
            Commands::Image { description } => {
                let req = ImageGenerationRequest::new(description.clone()).model("dall-e-3".into());

                let result = self.client.image_generation(req)?;

                let image_url = result.data[0].url.clone();

                let image_bytes = reqwest::get(image_url).await?.bytes().await?;

                let attachment = CreateAttachment::bytes(image_bytes, "generated_image.png");
                let builder = CreateMessage::new().content("Generated file:");

                msg.channel_id.send_files(&ctx.http, [attachment], builder).await?;
            }
        }

        Ok(())
    }
}
