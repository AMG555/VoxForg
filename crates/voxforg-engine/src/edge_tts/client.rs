use std::io::Cursor;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use futures::{SinkExt, StreamExt};
use sha2::{Digest, Sha256};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, warn};
use uuid::Uuid;

use voxforg_core::error::{Result, VoxForgError};
use voxforg_core::models::AudioChunk;
use crate::traits::SynthesisRequest;

pub const TRUSTED_CLIENT_TOKEN: &str = "6A5AA1D4EAFF4E9FB37E23D68491D6F4";
pub const SEC_MS_GEC_VERSION: &str = "1-143.0.3650.75";
pub const EDGE_WSS_HOST: &str = "speech.platform.bing.com";
pub const SAMPLE_RATE: u32 = 24000;
const WIN_EPOCH: u64 = 11644473600;

/// Generate time-windowed Sec-MS-GEC token required by Microsoft Edge TTS WebSocket
pub fn generate_sec_ms_gec() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut ticks = now + WIN_EPOCH;
    ticks -= ticks % 300;
    ticks *= 10_000_000;

    let payload = format!("{}{}", ticks, TRUSTED_CLIENT_TOKEN);
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let hash = hasher.finalize();

    hex::encode(hash).to_uppercase()
}

pub fn generate_muid() -> String {
    let id = Uuid::new_v4().simple().to_string();
    id.to_uppercase()
}

pub struct EdgeTtsClient;

impl EdgeTtsClient {
    /// Build WebSocket connection URL with valid anti-abuse headers
    fn build_url(connection_id: &str) -> String {
        let sec_ms_gec = generate_sec_ms_gec();
        format!(
            "wss://{}/consumer/speech/synthesize/readaloud/edge/v1?TrustedClientToken={}&ConnectionId={}&Sec-MS-GEC={}&Sec-MS-GEC-Version={}",
            EDGE_WSS_HOST,
            TRUSTED_CLIENT_TOKEN,
            connection_id,
            sec_ms_gec,
            SEC_MS_GEC_VERSION
        )
    }

    /// Synthesize speech and stream AudioChunk instances through channel
    pub async fn stream_synthesis(
        request: &SynthesisRequest,
        ssml: &str,
        tx: mpsc::Sender<Result<AudioChunk>>,
    ) -> Result<()> {
        let chunk = Self::synthesize(request, ssml).await?;
        let chunk_size = 4800; // 200ms at 24kHz
        let mut offset = 0;
        let total = chunk.pcm_data.len();

        while offset < total {
            let end = (offset + chunk_size).min(total);
            let is_final = end >= total;
            let slice = chunk.pcm_data[offset..end].to_vec();

            let sub_chunk = AudioChunk {
                sample_rate: chunk.sample_rate,
                channels: chunk.channels,
                pcm_data: slice,
                is_final,
            };

            if tx.send(Ok(sub_chunk)).await.is_err() {
                break;
            }
            offset = end;
        }

        Ok(())
    }

    /// Direct synthesis returning single AudioChunk
    pub async fn synthesize(_request: &SynthesisRequest, ssml: &str) -> Result<AudioChunk> {
        let connection_id = Uuid::new_v4().simple().to_string();
        let request_id = Uuid::new_v4().simple().to_string();
        let url = Self::build_url(&connection_id);
        debug!("Connecting to Edge TTS WebSocket: {}", url);

        let mut req = url.into_client_request().map_err(|e| {
            VoxForgError::AudioProcessing(format!("Failed to build WebSocket request: {}", e))
        })?;

        let headers = req.headers_mut();
        headers.insert(
            "User-Agent",
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36 Edg/143.0.0.0",
            ),
        );
        headers.insert(
            "Origin",
            HeaderValue::from_static("chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold"),
        );
        headers.insert("Pragma", HeaderValue::from_static("no-cache"));
        headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));
        headers.insert("Accept-Encoding", HeaderValue::from_static("gzip, deflate, br, zstd"));
        headers.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.9"));

        let cookie_val = format!("muid={};", generate_muid());
        if let Ok(hv) = HeaderValue::from_str(&cookie_val) {
            headers.insert("Cookie", hv);
        }

        let (ws_stream, _) = tokio::time::timeout(Duration::from_secs(8), connect_async(req))
            .await
            .map_err(|_| VoxForgError::AudioProcessing("Edge TTS connection timed out".to_string()))?
            .map_err(|e| VoxForgError::AudioProcessing(format!("Edge TTS WebSocket error: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();

        // 1. Send speech.config message requesting 24kHz MP3
        let js_date = chrono::Utc::now()
            .format("%a %b %d %Y %H:%M:%S GMT+0000 (Coordinated Universal Time)")
            .to_string();

        let config_json = serde_json::json!({
            "context": {
                "synthesis": {
                    "audio": {
                        "metadataoptions": {
                            "sentenceBoundaryEnabled": "true",
                            "wordBoundaryEnabled": "false"
                        },
                        "outputFormat": "audio-24khz-48kbitrate-mono-mp3"
                    }
                }
            }
        });

        let config_message = format!(
            "X-Timestamp:{}\r\nContent-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n{}\r\n",
            js_date, config_json
        );

        debug!("Sending Edge TTS config message");
        write
            .send(Message::Text(config_message))
            .await
            .map_err(|e| VoxForgError::AudioProcessing(format!("Failed to send speech.config: {}", e)))?;

        // 2. Send SSML request (note trailing 'Z' as required by Microsoft Edge speech parser)
        let ssml_message = format!(
            "X-RequestId:{}\r\nContent-Type:application/ssml+xml\r\nX-Timestamp:{}Z\r\nPath:ssml\r\n\r\n{}",
            request_id, js_date, ssml
        );

        debug!("Sending Edge TTS SSML request");
        write
            .send(Message::Text(ssml_message))
            .await
            .map_err(|e| VoxForgError::AudioProcessing(format!("Failed to send SSML message: {}", e)))?;

        // 3. Collect incoming MP3 bytes
        let mut mp3_bytes = Vec::new();

        while let Some(msg_result) = read.next().await {
            let msg = match msg_result {
                Ok(m) => m,
                Err(e) => {
                    warn!("WebSocket read error: {}", e);
                    break;
                }
            };

            match &msg {
                Message::Text(text) => {
                    debug!("Received text message: {}", text);
                    if text.contains("Path:turn.end") {
                        break;
                    }
                }
                Message::Binary(bytes) => {
                    debug!("Received binary frame (len {})", bytes.len());
                    if let Some(audio_data) = parse_binary_audio_payload(bytes) {
                        mp3_bytes.extend_from_slice(audio_data);
                    }
                }
                Message::Close(c) => {
                    if let Some(ref cf) = c {
                        debug!("WebSocket closed with code: {} ({:?}), reason: {}", u16::from(cf.code), cf.code, cf.reason);
                    }
                    break;
                }
                _ => {}
            }
        }

        if mp3_bytes.is_empty() {
            return Err(VoxForgError::AudioProcessing(
                "Edge TTS returned empty audio stream".to_string(),
            ));
        }

        let (sample_rate, channels, pcm_data) = decode_mp3_to_pcm(&mp3_bytes)?;

        Ok(AudioChunk {
            sample_rate,
            channels,
            pcm_data,
            is_final: true,
        })
    }
}

/// Parse raw binary WebSocket frame from Edge TTS and extract audio payload
pub fn parse_binary_audio_payload(bytes: &[u8]) -> Option<&[u8]> {
    if bytes.len() < 2 {
        return None;
    }

    let header_len = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
    let header_end = 2 + header_len;
    if bytes.len() < header_end {
        return None;
    }

    let header_str = std::str::from_utf8(&bytes[2..header_end]).ok()?;
    if !header_str.contains("Path:audio") {
        return None;
    }

    Some(&bytes[header_end..])
}

/// Decode in-memory MP3 bytes into 16-bit PCM samples using pure-Rust Symphonia
pub fn decode_mp3_to_pcm(mp3_bytes: &[u8]) -> Result<(u32, u16, Vec<i16>)> {
    let cursor = Cursor::new(mp3_bytes.to_vec());
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("mp3");

    let meta_opts = MetadataOptions::default();
    let fmt_opts = FormatOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| VoxForgError::AudioProcessing(format!("Symphonia probe error: {}", e)))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| VoxForgError::AudioProcessing("No valid audio track in MP3 stream".to_string()))?;

    let track_id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| VoxForgError::AudioProcessing(format!("Symphonia decoder error: {}", e)))?;

    let mut pcm_samples = Vec::new();
    let mut sample_rate = SAMPLE_RATE;
    let mut channels = 1;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(SymphoniaError::ResetRequired) => break,
            Err(e) => return Err(VoxForgError::AudioProcessing(format!("Packet read error: {}", e))),
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                let spec = *audio_buf.spec();
                sample_rate = spec.rate;
                channels = spec.channels.count() as u16;

                let mut sample_buf = SampleBuffer::<i16>::new(audio_buf.capacity() as u64, spec);
                sample_buf.copy_interleaved_ref(audio_buf);
                pcm_samples.extend_from_slice(sample_buf.samples());
            }
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(SymphoniaError::ResetRequired) => break,
            Err(e) => return Err(VoxForgError::AudioProcessing(format!("Decode frame error: {}", e))),
        }
    }

    Ok((sample_rate, channels, pcm_samples))
}
