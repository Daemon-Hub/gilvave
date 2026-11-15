use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use sycamore::prelude::Signal;
use web_sys::{AnalyserNode, AudioContext, MediaStream};

use gilvave_core::ids::ChannelId;

use crate::gateway::service::WsService;

pub struct VadController {
    is_active: Arc<AtomicBool>,
    audio_ctx: AudioContext,
}

impl VadController {
    pub fn start(
        stream: &MediaStream,
        channel_id: ChannelId,
        is_speaking_signal: Signal<bool>,
        is_muted_signal: Signal<bool>,
    ) -> Result<Self, String> {
        let audio_ctx = AudioContext::new().map_err(|e| format!("{e:?}"))?;
        let source = audio_ctx
            .create_media_stream_source(stream)
            .map_err(|e| format!("{e:?}"))?;

        let analyser = audio_ctx.create_analyser().map_err(|e| format!("{e:?}"))?;
        analyser.set_fft_size(256);
        analyser.set_smoothing_time_constant(0.2);

        source
            .connect_with_audio_node(&analyser)
            .map_err(|e| format!("{e:?}"))?;

        let is_active = Arc::new(AtomicBool::new(true));
        let active_flag = is_active.clone();

        wasm_bindgen_futures::spawn_local(async move {
            vad_loop(
                analyser,
                channel_id,
                is_speaking_signal,
                is_muted_signal,
                active_flag,
            )
            .await;
        });

        Ok(Self {
            is_active,
            audio_ctx,
        })
    }

    pub fn stop(&self) {
        self.is_active.store(false, Ordering::Relaxed);
        let _ = self.audio_ctx.close();
    }
}

impl Drop for VadController {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn vad_loop(
    analyser: AnalyserNode,
    channel_id: ChannelId,
    is_speaking_signal: Signal<bool>,
    is_muted_signal: Signal<bool>,
    active_flag: Arc<AtomicBool>,
) {
    let bin_count = analyser.frequency_bin_count() as usize;
    let mut buffer = vec![0u8; bin_count];

    let mut current_speaking = false;
    let mut hangover_ticks: u8 = 0;
    const HANGOVER_MAX: u8 = 8; // ~800ms hangover to prevent syllable flicker and ensure wave completion
    const VOLUME_THRESHOLD: u32 = 18;

    while active_flag.load(Ordering::Relaxed) {
        gloo_timers::future::TimeoutFuture::new(100).await;

        if !active_flag.load(Ordering::Relaxed) {
            break;
        }

        if is_muted_signal.get() {
            if current_speaking {
                current_speaking = false;
                hangover_ticks = 0;
                is_speaking_signal.set(false);
                let _ = WsService::send_voice_speaking(channel_id, false).await;
            }
            continue;
        }

        analyser.get_byte_frequency_data(&mut buffer);

        let sum: u32 = buffer.iter().map(|&x| x as u32).sum();
        let avg = sum / (bin_count.max(1) as u32);

        if avg >= VOLUME_THRESHOLD {
            hangover_ticks = HANGOVER_MAX;
            if !current_speaking {
                current_speaking = true;
                is_speaking_signal.set(true);
                let _ = WsService::send_voice_speaking(channel_id, true).await;
            }
        } else if hangover_ticks > 0 {
            hangover_ticks -= 1;
        } else if current_speaking {
            current_speaking = false;
            is_speaking_signal.set(false);
            let _ = WsService::send_voice_speaking(channel_id, false).await;
        }
    }

    if current_speaking {
        is_speaking_signal.set(false);
        let _ = WsService::send_voice_speaking(channel_id, false).await;
    }
}
