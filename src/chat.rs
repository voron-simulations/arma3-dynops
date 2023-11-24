use crate::{CONTEXT, RUNTIME};
use arma_rs::{Context, Group};
use chatgpt::{config::ModelConfigurationBuilder, converse::Conversation, prelude::ChatGPT};
use std::{collections::HashMap, env, sync::Arc, time::Duration};
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;

lazy_static::lazy_static! {
    static ref CONVERSATIONS: Mutex<HashMap<String, Arc<Mutex<Conversation>>>> = Mutex::new(HashMap::new());
    static ref CLIENT: ChatGPT = init_chat_gpt();
}

pub fn group() -> Group {
    Group::new()
        .command("init", init)
        .command("message", message)
}

fn init_chat_gpt() -> ChatGPT {
    let key = env::var("CHATGPT_KEY").unwrap();
    let config = ModelConfigurationBuilder::default()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    ChatGPT::new_with_config(key, config).unwrap()
}

fn init(context: Context, uid: String, prompt: String) {
    let mut conversations = CONVERSATIONS.blocking_lock();
    if conversations.contains_key(&uid) {
        return;
    }

    let conversation = CLIENT.new_conversation_directed(prompt);
    conversations.insert(uid, Arc::new(Mutex::new(conversation)));
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
            "systemChat",
            "The person heard you and is thinking of response",
        );

        let conversation_store = {
            let mut conversations = CONVERSATIONS.lock().await;
            Arc::clone(conversations.get_mut(&uid_copy).unwrap())
        };
        let mut conversation = conversation_store.lock().await;
        let response = conversation.send_message(message).await.unwrap(); // Send request to ChatGPT
        let reply: String = response.message().content.clone();
        _ = context.callback_data("dynops", "systemChat", reply);
    });
}
