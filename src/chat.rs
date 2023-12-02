use crate::{CONTEXT, RUNTIME};
use arma_rs::Group;
use chatgpt::functions::{GptFunction, gpt_function};
use chatgpt::{config::ModelConfigurationBuilder, converse::Conversation, prelude::ChatGPT};
use std::{collections::HashMap, env, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tokio::time::sleep;

lazy_static::lazy_static! {
    static ref CONVERSATIONS: Mutex<HashMap<String, Arc<Mutex<Conversation>>>> = Mutex::new(HashMap::new());
    static ref CLIENT: ChatGPT = init_chat_gpt();
}

pub fn group() -> Group {
    Group::new()
        .command("init", init)
        .command("destroy", destroy)
        .command("message", message)
}

/// Makes character flee from confrontation
/// 
#[gpt_function]
async fn flee() {
    let context_store = CONTEXT.read().await;
    if let Some(context) = context_store.as_ref() {
        _ = context.callback_null("dynops", "DynOps_fnc_agentFlee");
    }
}

/// Makes character attack nearest player, if the character feels too threatened
#[gpt_function]
async fn attack() {
    let context_store = CONTEXT.read().await;
    if let Some(context) = context_store.as_ref() {
        _ = context.callback_null("dynops", "systemChat");
    }
}


fn init_chat_gpt() -> ChatGPT {
    let key = env::var("CHATGPT_KEY").unwrap();
    let config = ModelConfigurationBuilder::default()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();
    ChatGPT::new_with_config(key, config).unwrap()
}

fn init(uid: String, prompt: String) {
    let mut conversations = CONVERSATIONS.blocking_lock();
    if conversations.contains_key(&uid) {
        return;
    }

    let mut conversation = CLIENT.new_conversation_directed(prompt);
    _ = conversation.add_function(flee());
    _ = conversation.add_function(attack());
    conversations.insert(uid, Arc::new(Mutex::new(conversation)));
}

fn destroy(uid: String) {
    let mut conversations = CONVERSATIONS.blocking_lock();
    conversations.remove(&uid);
}

fn message(uid: String, message: String) {
    RUNTIME.spawn(async {
        let uid_copy = uid;
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            // TODO: Error handling
            return;
        };
        sleep(Duration::from_millis(5)).await;
        _ = context.callback_data(
            "dynops",
            "DynOps_fnc_agentReply",
            "This person heard you and is thinking of response",
        );

        let conversation_store = {
            let mut conversations = CONVERSATIONS.lock().await;
            Arc::clone(conversations.get_mut(&uid_copy).unwrap())
        };
        let mut conversation = conversation_store.lock().await;
        let response = conversation.send_message(message).await.unwrap(); // Send request to ChatGPT
        let reply: String = response.message().content.clone();
        _ = context.callback_data("dynops", "DynOps_fnc_agentReply", reply);
    });
}
