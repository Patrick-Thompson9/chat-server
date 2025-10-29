use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;
use std::{collections::HashMap, sync::Arc};

type Sender = mpsc::UnboundedSender<Message>;

struct ChannelManager {
    channels: Arc<Mutex<HashMap<String,Vec<Sender>>>>,
}

impl ChannelManager {
    fn new() -> Self {
        ChannelManager { channels: Arc::new(Mutex::new(HashMap::new())) }
    }

    async fn get_or_create_channel(&self, channel_name: String) {
        let mut channels = self.channels.lock().await;
        if !channels.contains_key(&channel_name) {
            channels.insert(channel_name.clone(), Vec::new());
        }
    }

    async fn remove_sender_from_channel(&self, channel_name: String, sender: Sender) {
        let mut channels = self.channels.lock().await;
        if let Some(sender_vec) = channels.get_mut(&channel_name) {
            sender_vec.retain(|s| s as *const _ != &sender as *const _);
        }
    }
}