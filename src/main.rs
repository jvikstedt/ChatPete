use std::env;
use std::sync::Mutex;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use discord::{Discord, DiscordHandler};
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4_O;
use openai_api_rs::v1::image::ImageGenerationRequest;
use serenity::all::{Context, CreateAttachment, CreateMessage, Message};
use serenity::async_trait;
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

        #[clap(long = "model")]
        model: Option<ImageModel>,
        // #[clap(long = "quality")]
        // quality: Option<ImageQuality>,

        // #[clap(long = "style")]
        // style: Option<ImageStyle>,
    },
    Vision {
        link: String,
        question: String,
    },
}

#[derive(ValueEnum, Debug, Clone)]
enum ImageModel {
    Dalle3,
    Dalle2,
}

impl ImageModel {
    fn api_value(&self) -> String {
        match self {
            ImageModel::Dalle3 => String::from("dall-e-3"),
            ImageModel::Dalle2 => String::from("dall-e-2"),
        }
    }
}

// TODO:
// openai-api-rs library not yet supporting quality or style changes

// #[derive(ValueEnum, Debug, Clone)]
// enum ImageQuality {
//     Standard,
//     HD,
// }
//
// impl ImageQuality {
//     fn api_value(&self) -> String {
//         match self {
//             ImageQuality::Standard => {
//                 String::from("standard")
//             },
//             ImageQuality::HD => {
//                 String::from("hd")
//             }
//         }
//     }
// }
//
// #[derive(ValueEnum, Debug, Clone)]
// enum ImageStyle {
//     Vivid,
//     Natural,
// }
//
// impl ImageStyle {
//     fn api_value(&self) -> String {
//         match self {
//             ImageStyle::Vivid => {
//                 String::from("vivid")
//             },
//             ImageStyle::Natural => {
//                 String::from("natural")
//             }
//         }
//     }
// }

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

    async fn handle_chat_command(
        &self,
        ctx: &Context,
        msg: &Message,
        message: &str,
        author: Option<String>,
    ) -> Result<()> {
        let req = ChatCompletionRequest::new(
            GPT4_O.to_string(),
            vec![
                chat_completion::ChatCompletionMessage {
                    role: chat_completion::MessageRole::system,
                    content: chat_completion::Content::Text(
                        self.description.lock().unwrap().clone(),
                    ),
                    name: Some("ChatPete".into()),
                },
                chat_completion::ChatCompletionMessage {
                    role: chat_completion::MessageRole::user,
                    content: chat_completion::Content::Text(message.to_string()),
                    name: author,
                },
            ],
        );

        let result = self.client.chat_completion(req)?;
        let content = result.choices[0].message.content.clone();
        msg.channel_id.say(&ctx.http, content.unwrap()).await?;
        Ok(())
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

        let command = Commands::try_parse_from(&args);

        let author: Option<String> = msg.author.global_name.clone().map_or_else(
            || None,
            |v| {
                Some(
                    unidecode(&v)
                        .chars()
                        .filter(|c| c.is_alphabetic())
                        .collect(),
                )
            },
        );

        match command {
            Err(e) => match e.kind() {
                clap::error::ErrorKind::DisplayHelp => {
                    return Err(e.into());
                }
                _ => {
                    self.handle_chat_command(ctx, msg, args[1..].join("").as_str(), author)
                        .await?;
                }
            },
            Ok(Commands::Chat { message }) => {
                self.handle_chat_command(ctx, msg, &message, author).await?;
            }
            Ok(Commands::Init { description }) => {
                *self.description.lock().unwrap() = description;
                msg.channel_id.say(&ctx.http, "Ok.").await?;
            }
            Ok(Commands::Image { description, model }) => {
                let model_val = model.unwrap_or(ImageModel::Dalle2).api_value();
                // let quality_val = quality.unwrap_or(ImageQuality::Standard).api_value();
                // let style_val = style.unwrap_or(ImageStyle::Vivid).api_value();

                let req = ImageGenerationRequest::new(description.clone()).model(model_val.clone());

                let result = self.client.image_generation(req)?;

                let image_url = result.data[0].url.clone();

                let image_bytes = reqwest::get(image_url).await?.bytes().await?;

                let attachment = CreateAttachment::bytes(image_bytes, "generated_image.png");
                let builder =
                    CreateMessage::new().content(format!("Generated image ({})", &model_val));

                msg.channel_id
                    .send_files(&ctx.http, [attachment], builder)
                    .await?;
            }
            Ok(Commands::Vision { link, question }) => {
                let req = ChatCompletionRequest::new(
                    GPT4_O.to_string(),
                    vec![chat_completion::ChatCompletionMessage {
                        role: chat_completion::MessageRole::user,
                        content: chat_completion::Content::ImageUrl(vec![
                            chat_completion::ImageUrl {
                                r#type: chat_completion::ContentType::text,
                                text: Some(question.clone()),
                                image_url: None,
                            },
                            chat_completion::ImageUrl {
                                r#type: chat_completion::ContentType::image_url,
                                text: None,
                                image_url: Some(chat_completion::ImageUrlType {
                                    url: link.clone(),
                                }),
                            },
                        ]),
                        name: author,
                    }],
                );

                let result = self.client.chat_completion(req)?;
                let content = result.choices[0].message.content.clone();
                msg.channel_id.say(&ctx.http, content.unwrap()).await?;
            }
        }

        Ok(())
    }
}
