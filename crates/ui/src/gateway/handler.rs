use std::sync::{Arc, LazyLock, Mutex};
use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use gilvave_core::dto::{message::MessageView, ws::ServerRecieve};

#[derive(Clone, Debug)]
pub enum GatewayEvent {
    MessageNew(MessageView),
    ChannelHistoryBefore(Vec<MessageView>),
    ChannelHistoryAfter(Vec<MessageView>),
}

static SUBSCRIBERS: LazyLock<Arc<Mutex<Vec<UnboundedSender<GatewayEvent>>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

pub fn subscribe_gateway_events() -> UnboundedReceiver<GatewayEvent> {
    let (tx, rx) = unbounded();
    if let Ok(mut subs) = SUBSCRIBERS.lock() {
        subs.push(tx);
    }
    rx
}

pub fn dispatch_gateway_event(event: GatewayEvent) {
    if let Ok(mut subs) = SUBSCRIBERS.lock() {
        subs.retain(|tx| tx.unbounded_send(event.clone()).is_ok());
    }
}

pub async fn handle(text: String) {
    match serde_json::from_str::<ServerRecieve>(&text) {
        Ok(msg) => match msg {
            ServerRecieve::HeartbeatAck => {
                web_sys::console::log_1(&"[WS] HeartbeatAck".into());
            }
            ServerRecieve::Hello => {
                web_sys::console::log_1(&"[WS] Hello received".into());
            }
            ServerRecieve::Error { message } => {
                web_sys::console::error_1(&format!("[WS] Server error: {message}").into());
            }
            ServerRecieve::JoinSuccess => {
                web_sys::console::log_1(&"[WS] JoinSuccess received".into());
            }
            ServerRecieve::MessageNew(message_view) => {
                web_sys::console::log_1(&format!("[WS] MessageNew received: {:?}", message_view).into());
                dispatch_gateway_event(GatewayEvent::MessageNew(message_view));
            }
            ServerRecieve::ChannelHistoryBefore(messages) => {
                web_sys::console::log_1(&format!("[WS] ChannelHistoryBefore received {} messages", messages.len()).into());
                dispatch_gateway_event(GatewayEvent::ChannelHistoryBefore(messages));
            }
            ServerRecieve::ChannelHistoryAfter(messages) => {
                web_sys::console::log_1(&format!("[WS] ChannelHistoryAfter received {} messages", messages.len()).into());
                dispatch_gateway_event(GatewayEvent::ChannelHistoryAfter(messages));
            }
        },
        Err(e) => {
            web_sys::console::error_1(&format!("[WS] parse ServerRecieve error: {e}, text: {text}").into());
        }
    }
}
