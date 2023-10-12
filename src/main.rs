use std::borrow::BorrowMut;
use std::env;

use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::prelude::ChannelId;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, _new_message: Message) {
        async fn send_discord_message(_ctx: Context, channel_id: ChannelId, message: String) {
            match _ctx.cache.guild_channel(channel_id) {
                Some(guild_channel) => {
                    let result = guild_channel.send_message(&_ctx, |m| m.content(message)).await;

                    match result {
                        Ok(_) => {},
                        Err(err) => {
                            eprintln!("failed to send a message to the channel, error: {}", err.to_string());
                        }
                    }
                },
                None => {
                    eprintln!("failed to get guild channel to send a message");
                }
            }
        }

        if _new_message.content == ".color-reset" {
            let guild = _ctx.cache.guild(_new_message.guild_id.unwrap()).unwrap();

            match guild.member(&_ctx.http, _new_message.author.id).await {
                Ok(member) => {
                    let color_role_name_prefix = format!("color_bot_{}_", member.user.name);
                    
                    // remove past colors added
                    for member_role_id in &member.roles {
                        for guild_role in &guild.roles {
                            if guild_role.0.0 == member_role_id.clone().0 {
                                if guild_role.1.name.starts_with(&color_role_name_prefix.as_str()) {
                                    match &guild.delete_role(&_ctx, guild_role.0).await {
                                        Ok(_) => { },
                                        Err(err) => {
                                            eprintln!("failed to remove member color, error: {}", err.to_string());
                                            send_discord_message(_ctx.clone(), _new_message.channel_id, "Failed to remove member color".to_string()).await;
                                        }
                                    }
                                }
        
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    eprintln!("failed to get member, error: {}", err.to_string());
                    send_discord_message(_ctx.clone(), _new_message.channel_id, "Failed to get member".to_string()).await;
                }
            }
        } else if _new_message.content.starts_with(".color-set ") {
            let mut color_string = &_new_message.content[11..];

            if color_string.starts_with("#") {
                color_string = &color_string[1..];
            }

            if color_string.len() == 0 {
                send_discord_message(_ctx.clone(), _new_message.channel_id, "Empty color".to_string()).await;

                return;
            }

            let mut color = 0xFFFFFF;

            match u64::from_str_radix(color_string, 16) {
                Ok(color_integer) => {
                    color = color_integer;
                },
                Err(err) => {
                    eprintln!("Invalid HEX value, error: {}", err.to_string());
                    send_discord_message(_ctx.clone(), _new_message.channel_id, "Invalid hex number, please use hex colors".to_string()).await;

                    return;
                }
            }

            if color == 0 {
                send_discord_message(_ctx.clone(), _new_message.channel_id, "Discord doesn't work with 0x000000 color, using the fallback color 0x0000001".to_string()).await;
                color = 0x1;
            }

            let guild = _ctx.cache.guild(_new_message.guild_id.unwrap()).unwrap();

            match guild.member(&_ctx.http, _new_message.author.id).await {
                Ok(member) => {
                    let color_role_name_prefix = format!("color_bot_{}_", member.user.name);

                    // remove past colors added
                    for member_role_id in &member.roles {
                        for guild_role in &guild.roles {
                            if guild_role.0.0 == member_role_id.clone().0 {
                                if guild_role.1.name.starts_with(&color_role_name_prefix.as_str()) {
                                    match &guild.delete_role(&_ctx, guild_role.0).await {
                                        Ok(_) => { },
                                        Err(err) => {
                                            eprintln!("failed to remove member color, error: {}", err.to_string());
                                            send_discord_message(_ctx.clone(), _new_message.channel_id, "Failed to remove member color".to_string()).await;
                                        }
                                    }
                                }
        
                                break;
                            }
                        }
                    }

                    // create the new color role
                    match guild.create_role(&_ctx, |r| r.name(format!("{}{}", color_role_name_prefix, color_string)).colour(color).position(0)).await {
                        Ok(new_role) => {
                            // add the role
                            match member.clone().add_role(&_ctx, new_role.id).await {
                                Ok(_) => { },
                                Err(err) => {
                                    eprintln!("failed to add a new role to member, error: {}", err.to_string());
                                    send_discord_message(_ctx.clone(), _new_message.channel_id, "Failed to add a new role to member".to_string()).await;
                                }
                            }
                        },
                        Err(err) => {
                            eprintln!("failed to create a color role, error: {}", err.to_string());
                            send_discord_message(_ctx.clone(), _new_message.channel_id, "Failed to create a color role".to_string()).await;
                        }
                    }
                },
                Err(err) => {
                    eprintln!("failed to get member, error: {}", err.to_string());
                    send_discord_message(_ctx.clone(), _new_message.channel_id, "Failed to get member".to_string()).await;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new();

    let args = env::args().collect::<Vec<String>>();
    let token = args.get(1).unwrap_or(&"--".to_string()).to_owned();

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILDS;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
