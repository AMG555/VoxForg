use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(tag = "event")]
pub enum TwilioInboundMessage {
    #[serde(rename = "connected")]
    Connected { protocol: Option<String>, version: Option<String> },
    #[serde(rename = "start")]
    Start {
        #[serde(rename = "streamSid")]
        stream_sid: String,
        #[serde(rename = "accountSid")]
        account_sid: Option<String>,
        #[serde(rename = "callSid")]
        call_sid: Option<String>,
    },
    #[serde(rename = "media")]
    Media {
        #[serde(rename = "streamSid")]
        stream_sid: String,
        media: TwilioMediaPayload,
    },
    #[serde(rename = "stop")]
    Stop {
        #[serde(rename = "streamSid")]
        stream_sid: Option<String>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct TwilioMediaPayload {
    pub payload: String, // Base64 encoded 8kHz mu-law audio
    #[serde(default)]
    pub track: Option<String>,
    #[serde(default)]
    pub chunk: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TwilioOutboundMedia {
    pub event: String,
    #[serde(rename = "streamSid")]
    pub stream_sid: String,
    pub media: TwilioOutboundPayload,
}

#[derive(Debug, Serialize)]
pub struct TwilioOutboundPayload {
    pub payload: String, // Base64 encoded 8kHz mu-law audio
}

/// `GET /v1/telephony/twilio/stream` — bidirectional WebSocket gateway for Twilio Media Streams
pub async fn twilio_media_stream_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_twilio_socket(socket, state))
}

async fn handle_twilio_socket(mut socket: WebSocket, _state: AppState) {
    info!("Twilio media stream WebSocket connected");
    let mut current_stream_sid = String::new();

    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                warn!("Twilio WebSocket read error: {e}");
                break;
            }
        };

        if let Message::Text(text) = msg {
            if let Ok(inbound) = serde_json::from_str::<TwilioInboundMessage>(&text) {
                match inbound {
                    TwilioInboundMessage::Connected { protocol, version } => {
                        debug!("Twilio connected: protocol={protocol:?}, version={version:?}");
                    }
                    TwilioInboundMessage::Start { stream_sid, call_sid, .. } => {
                        info!("Twilio stream started: streamSid={stream_sid}, callSid={call_sid:?}");
                        current_stream_sid = stream_sid;
                    }
                    TwilioInboundMessage::Media { stream_sid, media } => {
                        // Received 8kHz mu-law audio packet from phone line caller
                        debug!(
                            "Twilio media packet: streamSid={}, payload_len={}",
                            stream_sid,
                            media.payload.len()
                        );
                        // Echo or route through ASR -> TTS pipeline
                        // Send audio back when assistant responds
                    }
                    TwilioInboundMessage::Stop { stream_sid } => {
                        info!("Twilio stream stopped: {stream_sid:?}");
                        break;
                    }
                    TwilioInboundMessage::Unknown => {}
                }
            }
        } else if let Message::Close(_) = msg {
            info!("Twilio WebSocket closed by client");
            break;
        }
    }

    info!("Twilio media stream session ended (sid={})", current_stream_sid);
}
