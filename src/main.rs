use std::{env, sync::Arc};

use async_openai::{Client, types::{CreateChatRequestArgs, MessageArgs, self}};
use serenity::{async_trait, prelude::{EventHandler, Context, GatewayIntents, TypeMapKey, RwLock}, model::prelude::{Message, Ready}, Client as SerenityClient};

struct History;

impl TypeMapKey for History {
    type Value = Arc<RwLock<Vec<types::Message>>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot { return };

        let client = Client::new();

        let message = MessageArgs::default()
            .content(msg.content.to_string())
            .role(async_openai::types::MessageRole::Assistant)
            .build()
            .expect("Erreur de build de Message");

        let history_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<History>().expect("Erreur lors de la récupération de l'historique").clone()
        };

        let mut history = history_lock.write().await;

        history.push(message);

        let request = CreateChatRequestArgs::default()
            .model("gpt-3.5-turbo-0301")
            .messages(history.to_vec())
            .build()
            .expect("Erreur de build de CreateChatRequest");

        let response = client.chat().create(request).await.expect("Erreur pour récupérer la réponse");

        println!("\nResponse (single):\n");
        for choice in response.choices {
            msg.channel_id.say(&ctx.http, choice.message.content).await.expect("Erreur lors de l'envoie du message");
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client =
        SerenityClient::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<History>(Arc::new(RwLock::new(vec![])));
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
