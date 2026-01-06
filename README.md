# ChatPete

ChatPete is a Discord bot powered by OpenAI's GPT-4 and DALL-E models. It provides intelligent chat responses, image generation, and vision capabilities directly in your Discord server.

## Features

- **Chat**: Interactive conversations with GPT-4o in dedicated threads
- **Image Generation**: Create images using DALL-E 2 or DALL-E 3
- **Vision**: Ask questions about images by providing image URLs
- **Thread Management**: Automatically creates threads for organized conversations
- **Custom Bot Personality**: Initialize the bot with custom system prompts

## Prerequisites

- Rust (2021 edition or later)
- Discord Bot Token
- OpenAI API Key

## Installation

1. Clone the repository:
```bash
git clone https://github.com/jvikstedt/ChatPete.git
cd ChatPete
```

2. Build the project:
```bash
cargo build --release
```

## Configuration

Set the required environment variables:

```bash
export DISCORD_API_KEY="your_discord_bot_token"
export OPENAI_API_KEY="your_openai_api_key"
```

### Getting API Keys

- **Discord Bot Token**: Create a bot at [Discord Developer Portal](https://discord.com/developers/applications)
  - Enable "Message Content Intent" in the Bot settings
  - Grant the bot necessary permissions: Send Messages, Create Public Threads, Read Message History
- **OpenAI API Key**: Get your API key from [OpenAI Platform](https://platform.openai.com/api-keys)

## Usage

Run the bot:
```bash
cargo run --release
```

### Commands

Mention the bot (`@ChatPete`) in a Discord channel to interact with it. The bot will automatically create a thread for the conversation.

#### Chat
Simply mention the bot with your message:
```
@ChatPete Hello! How are you today?
```

Or use the explicit chat command:
```
@ChatPete chat What is the meaning of life?
```

#### Initialize Bot Personality
Customize the bot's behavior with a system prompt:
```
@ChatPete init You are a pirate who speaks in pirate language
```

#### Image Generation
Generate images using DALL-E:
```
@ChatPete image A sunset over the mountains
```

Specify the model (default is DALL-E 2):
```
@ChatPete image A futuristic city --model dalle3
```

Available models:
- `dalle2` - DALL-E 2 (default)
- `dalle3` - DALL-E 3

#### Vision
Ask questions about images:
```
@ChatPete vision https://example.com/image.jpg What do you see in this image?
```

## Dependencies

- [serenity](https://github.com/serenity-rs/serenity) - Discord API library
- [openai-api-rs](https://github.com/openai-rs/openai-api-rs) - OpenAI API client
- [tokio](https://tokio.rs/) - Async runtime
- [clap](https://github.com/clap-rs/clap) - Command-line argument parser

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Janne Vikstedt

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
