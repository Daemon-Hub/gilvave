use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{
    HtmlAudioElement, MediaStream, MediaStreamTrack, RtcConfiguration, RtcIceCandidateInit,
    RtcIceServer, RtcPeerConnection, RtcPeerConnectionIceEvent, RtcSdpType,
    RtcSessionDescriptionInit, RtcTrackEvent,
};

use gilvave_core::{
    dto::voice::{VoiceSessionType, VoiceSignalData},
    ids::{ChannelId, UserId},
};

use crate::{
    components::common::{UserProfileContext, VoiceConnectionStatus, VoiceContext},
    gateway::service::WsService,
    voice::vad::VadController,
};

type ScreenEndedClosure = Rc<RefCell<Option<Closure<dyn Fn()>>>>;

struct ActiveCall {
    channel_id: ChannelId,
    publisher_pc: RtcPeerConnection,
    subscriber_pc: RtcPeerConnection,
    pending_pub_candidates: Rc<RefCell<Vec<RtcIceCandidateInit>>>,
    pending_sub_candidates: Rc<RefCell<Vec<RtcIceCandidateInit>>>,
    _mic_stream: MediaStream,
    mic_track: MediaStreamTrack,
    vad: Option<VadController>,
    audio_elements: Rc<RefCell<Vec<HtmlAudioElement>>>,
    audio_ctx: Option<web_sys::AudioContext>,
    _on_publisher_ice: Closure<dyn Fn(RtcPeerConnectionIceEvent)>,
    _on_subscriber_ice: Closure<dyn Fn(RtcPeerConnectionIceEvent)>,
    _on_subscriber_track: Closure<dyn Fn(RtcTrackEvent)>,
    camera_stream: Rc<RefCell<Option<MediaStream>>>,
    camera_sender: Rc<RefCell<Option<web_sys::RtcRtpSender>>>,
    screen_stream: Rc<RefCell<Option<MediaStream>>>,
    screen_sender: Rc<RefCell<Option<web_sys::RtcRtpSender>>>,
    _on_screen_ended: ScreenEndedClosure,
}

thread_local! {
    static ACTIVE_CALL: Rc<RefCell<Option<ActiveCall>>> = Rc::new(RefCell::new(None));
    static PENDING_REMOTE_VIDEO_TRACKS: RefCell<Vec<(MediaStream, MediaStreamTrack, String, String)>> = const { RefCell::new(Vec::new()) };
}

fn get_active_call_cell() -> Rc<RefCell<Option<ActiveCall>>> {
    ACTIVE_CALL.with(|c| c.clone())
}

fn get_sdp_from_js(val: &JsValue) -> String {
    js_sys::Reflect::get(val, &JsValue::from_str("sdp"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

pub struct VoiceManager;

impl VoiceManager {
    pub fn is_connected() -> bool {
        let cell = get_active_call_cell();
        cell.borrow().is_some()
    }

    pub fn active_channel_id() -> Option<ChannelId> {
        let cell = get_active_call_cell();
        cell.borrow().as_ref().map(|c| c.channel_id)
    }

    pub async fn connect(
        channel_id: ChannelId,
        voice_ctx: VoiceContext,
        user_profile: UserProfileContext,
    ) -> Result<(), String> {
        if let Some(current_cid) = Self::active_channel_id() {
            if current_cid == channel_id {
                return Ok(());
            }
            Self::disconnect(voice_ctx, user_profile.user_id.get()).await;
        }

        voice_ctx
            .connection_status
            .set(VoiceConnectionStatus::Connecting);
        voice_ctx.current_channel_id.set(Some(channel_id));

        if let Err(e) = WsService::join_voice_channel(channel_id).await {
            voice_ctx
                .connection_status
                .set(VoiceConnectionStatus::Disconnected);
            voice_ctx.current_channel_id.set(None);
            return Err(format!("Failed to join voice channel via WS: {e}"));
        }

        let window = web_sys::window().ok_or_else(|| "Window not found".to_string())?;
        let nav = window.navigator();
        let media_devices = nav.media_devices().map_err(|e| format!("{e:?}"))?;

        let constraints = web_sys::MediaStreamConstraints::new();
        constraints.set_audio(&true.into());
        constraints.set_video(&false.into());

        let stream_promise = media_devices
            .get_user_media_with_constraints(&constraints)
            .map_err(|e| format!("Microphone access error: {e:?}"))?;

        let stream_js = wasm_bindgen_futures::JsFuture::from(stream_promise)
            .await
            .map_err(|e| format!("Failed to acquire microphone: {e:?}"))?;

        let mic_stream: MediaStream = stream_js.unchecked_into();
        let audio_tracks = mic_stream.get_audio_tracks();
        if audio_tracks.length() == 0 {
            return Err("No audio tracks in media stream".to_string());
        }
        let mic_track: MediaStreamTrack = audio_tracks.get(0).unchecked_into();

        if user_profile.is_muted.get() || user_profile.is_deafened.get() {
            mic_track.set_enabled(false);
        }

        let pub_config = create_rtc_configuration().map_err(|e| format!("{e:?}"))?;
        let publisher_pc =
            RtcPeerConnection::new_with_configuration(&pub_config).map_err(|e| format!("{e:?}"))?;

        publisher_pc.add_track_0(&mic_track, &mic_stream);

        let cid = channel_id;
        let on_publisher_ice = Closure::<dyn Fn(RtcPeerConnectionIceEvent)>::new(
            move |evt: RtcPeerConnectionIceEvent| {
                if let Some(candidate) = evt.candidate() {
                    let cand_str = candidate.candidate();
                    if cand_str.is_empty() {
                        return;
                    }
                    let sdp_mid = candidate.sdp_mid();
                    let sdp_m_line_index = candidate.sdp_m_line_index();
                    wasm_bindgen_futures::spawn_local(async move {
                        let _ = WsService::send_voice_signal(
                            cid,
                            VoiceSignalData::IceCandidate {
                                session: VoiceSessionType::Publisher,
                                candidate: cand_str,
                                sdp_mid,
                                sdp_mline_index: sdp_m_line_index,
                            },
                        )
                        .await;
                    });
                }
            },
        );
        publisher_pc.set_onicecandidate(Some(on_publisher_ice.as_ref().unchecked_ref()));

        let sub_config = create_rtc_configuration().map_err(|e| format!("{e:?}"))?;
        let subscriber_pc =
            RtcPeerConnection::new_with_configuration(&sub_config).map_err(|e| format!("{e:?}"))?;

        let audio_ctx = web_sys::AudioContext::new().ok();
        if let Some(ref ctx) = audio_ctx {
            let _ = ctx.resume();
        }
        let audio_ctx_for_track = audio_ctx.clone();

        let audio_elements = Rc::new(RefCell::new(Vec::new()));
        let audio_elements_for_track = audio_elements.clone();
        let is_deafened_signal = user_profile.is_deafened;
        let my_user_id_signal = user_profile.user_id;
        let remote_video_streams_signal = voice_ctx.remote_video_streams;
        let remote_screen_streams_signal = voice_ctx.remote_screen_streams;
        let participants_signal = voice_ctx.participants;

        let on_subscriber_track = Closure::<dyn Fn(RtcTrackEvent)>::new(
            move |evt: RtcTrackEvent| {
                let track = evt.track();
                let track_kind = track.kind();
                web_sys::console::log_1(
                    &format!(
                        "[VoiceManager] Incoming track: kind={}, id={}",
                        track_kind,
                        track.id()
                    )
                    .into(),
                );

                if track_kind == "audio" {
                    let stream = match MediaStream::new() {
                        Ok(s) => {
                            s.add_track(&track);
                            s
                        }
                        Err(_) => {
                            let streams = evt.streams();
                            if streams.length() > 0 {
                                streams.get(0).unchecked_into::<MediaStream>()
                            } else {
                                return;
                            }
                        }
                    };

                    // Connect to Web Audio destination for WASAPI device pipeline registration
                    if let Some(ctx) = audio_ctx_for_track.as_ref() {
                        let _ = ctx.resume();
                        if let Ok(source_node) = ctx.create_media_stream_source(&stream) {
                            let _ = source_node.connect_with_audio_node(&ctx.destination());
                        }
                    }

                    if let Ok(audio_el) = HtmlAudioElement::new() {
                        audio_el.set_autoplay(true);
                        let _ = audio_el.set_attribute("playsinline", "true");
                        audio_el.set_src_object(Some(&stream));
                        if is_deafened_signal.get() {
                            audio_el.set_volume(0.0);
                        } else {
                            audio_el.set_volume(1.0);
                        }

                        // Hide the element and append to DOM so browser autoplay policy allows playback
                        let _ = audio_el.set_attribute(
                            "style",
                            "display:none;position:fixed;pointer-events:none;",
                        );
                        if let Some(window) = web_sys::window()
                            && let Some(doc) = window.document()
                            && let Some(body) = doc.body()
                        {
                            let _ = body.append_child(&audio_el);
                        }

                        if let Ok(play_promise) = audio_el.play() {
                            wasm_bindgen_futures::spawn_local(async move {
                                let _ = wasm_bindgen_futures::JsFuture::from(play_promise).await;
                            });
                        }

                        audio_elements_for_track.borrow_mut().push(audio_el);
                    }
                } else if track_kind == "video" {
                    let incoming_streams = evt.streams();
                    let stream_id = if incoming_streams.length() > 0 {
                        incoming_streams.get(0).unchecked_into::<MediaStream>().id()
                    } else {
                        String::new()
                    };

                    let stream = match MediaStream::new() {
                        Ok(s) => {
                            s.add_track(&track);
                            s
                        }
                        Err(_) => {
                            if incoming_streams.length() > 0 {
                                incoming_streams.get(0).unchecked_into::<MediaStream>()
                            } else {
                                return;
                            }
                        }
                    };

                    let track_id = track.id();
                    web_sys::console::log_1(
                        &format!(
                            "[VoiceManager] Incoming video track: track_id={track_id}, stream_id={stream_id}"
                        )
                        .into(),
                    );

                    let participants_list = participants_signal.get_clone();
                    let my_uid = my_user_id_signal.get();
                    let mut matched_cam_user = None;
                    let mut matched_screen_user = None;

                    // Tier 1: Canonical prefix strip
                    if let Some(id_str) = stream_id
                        .strip_prefix("camera-stream-")
                        .or_else(|| stream_id.strip_prefix("camera-"))
                        .or_else(|| track_id.strip_prefix("camera-stream-"))
                        .or_else(|| track_id.strip_prefix("camera-"))
                        && let Ok(uid) = gilvave_core::uuid::Uuid::parse_str(id_str)
                    {
                        matched_cam_user = Some(gilvave_core::ids::UserId::from(uid));
                    }

                    if let Some(id_str) = stream_id
                        .strip_prefix("screen-stream-")
                        .or_else(|| stream_id.strip_prefix("screen-"))
                        .or_else(|| track_id.strip_prefix("screen-stream-"))
                        .or_else(|| track_id.strip_prefix("screen-"))
                        && let Ok(uid) = gilvave_core::uuid::Uuid::parse_str(id_str)
                    {
                        matched_screen_user = Some(gilvave_core::ids::UserId::from(uid));
                    }

                    // Tier 2: Search for participant UUID within stream_id or track_id (excluding self)
                    if matched_cam_user.is_none() && matched_screen_user.is_none() {
                        for p in &participants_list {
                            if Some(p.user_id) == my_uid {
                                continue;
                            }
                            let uid_str = p.user_id.to_string();
                            if stream_id.contains(&uid_str) || track_id.contains(&uid_str) {
                                if stream_id.contains("screen") || track_id.contains("screen") {
                                    matched_screen_user = Some(p.user_id);
                                    break;
                                } else {
                                    matched_cam_user = Some(p.user_id);
                                    break;
                                }
                            }
                        }
                    }

                    // Tier 3: Heuristic fallback based on known participant broadcast flags (excluding self)
                    if matched_cam_user.is_none() && matched_screen_user.is_none() {
                        let screen_candidates: Vec<gilvave_core::ids::UserId> = participants_list
                            .iter()
                            .filter(|p| {
                                Some(p.user_id) != my_uid
                                    && p.is_screen_sharing
                                    && !remote_screen_streams_signal
                                        .with(|m| m.contains_key(&p.user_id))
                            })
                            .map(|p| p.user_id)
                            .collect();

                        let camera_candidates: Vec<gilvave_core::ids::UserId> = participants_list
                            .iter()
                            .filter(|p| {
                                Some(p.user_id) != my_uid
                                    && p.is_camera_on
                                    && !remote_video_streams_signal
                                        .with(|m| m.contains_key(&p.user_id))
                            })
                            .map(|p| p.user_id)
                            .collect();

                        if stream_id.contains("screen") || track_id.contains("screen") {
                            if let Some(&uid) = screen_candidates.first() {
                                matched_screen_user = Some(uid);
                            }
                        } else if stream_id.contains("camera") || track_id.contains("camera") {
                            if let Some(&uid) = camera_candidates.first() {
                                matched_cam_user = Some(uid);
                            }
                        } else if screen_candidates.len() == 1 && camera_candidates.is_empty() {
                            matched_screen_user = Some(screen_candidates[0]);
                        } else if camera_candidates.len() == 1 && screen_candidates.is_empty() {
                            matched_cam_user = Some(camera_candidates[0]);
                        } else if let Some(&uid) = camera_candidates.first() {
                            matched_cam_user = Some(uid);
                        } else if let Some(&uid) = screen_candidates.first() {
                            matched_screen_user = Some(uid);
                        }
                    }

                    // Tier 4: Room-level fallback when VideoState has a race condition
                    if matched_cam_user.is_none() && matched_screen_user.is_none() {
                        let other_participants: Vec<gilvave_core::ids::UserId> = participants_list
                            .iter()
                            .filter(|p| Some(p.user_id) != my_uid)
                            .map(|p| p.user_id)
                            .collect();

                        if other_participants.len() == 1 {
                            let other_uid = other_participants[0];
                            if stream_id.contains("screen")
                                || track_id.contains("screen")
                                || remote_video_streams_signal.with(|m| m.contains_key(&other_uid))
                            {
                                matched_screen_user = Some(other_uid);
                            } else {
                                matched_cam_user = Some(other_uid);
                            }
                        }
                    }

                    if let Some(user_id) = matched_cam_user {
                        web_sys::console::log_1(
                            &format!("[VoiceManager] Matched camera stream for user {user_id}")
                                .into(),
                        );
                        remote_video_streams_signal.update(|map| {
                            map.insert(user_id, stream.clone());
                        });
                    } else if let Some(user_id) = matched_screen_user {
                        web_sys::console::log_1(
                            &format!("[VoiceManager] Matched screen stream for user {user_id}")
                                .into(),
                        );
                        remote_screen_streams_signal.update(|map| {
                            map.insert(user_id, stream.clone());
                        });
                    } else {
                        web_sys::console::warn_1(
                            &format!(
                                "[VoiceManager] Queuing pending video track (track_id={track_id}, stream_id={stream_id})"
                            )
                            .into(),
                        );
                        PENDING_REMOTE_VIDEO_TRACKS.with(|cell| {
                            cell.borrow_mut().push((
                                stream.clone(),
                                track.clone(),
                                stream_id.clone(),
                                track_id.clone(),
                            ));
                        });
                    }
                }
            },
        );
        subscriber_pc.set_ontrack(Some(on_subscriber_track.as_ref().unchecked_ref()));

        let on_subscriber_ice = Closure::<dyn Fn(RtcPeerConnectionIceEvent)>::new(
            move |evt: RtcPeerConnectionIceEvent| {
                if let Some(candidate) = evt.candidate() {
                    let cand_str = candidate.candidate();
                    if cand_str.is_empty() {
                        return;
                    }
                    let sdp_mid = candidate.sdp_mid();
                    let sdp_m_line_index = candidate.sdp_m_line_index();
                    wasm_bindgen_futures::spawn_local(async move {
                        let _ = WsService::send_voice_signal(
                            cid,
                            VoiceSignalData::IceCandidate {
                                session: VoiceSessionType::Subscriber,
                                candidate: cand_str,
                                sdp_mid,
                                sdp_mline_index: sdp_m_line_index,
                            },
                        )
                        .await;
                    });
                }
            },
        );
        subscriber_pc.set_onicecandidate(Some(on_subscriber_ice.as_ref().unchecked_ref()));

        let vad = VadController::start(
            &mic_stream,
            channel_id,
            voice_ctx.is_speaking,
            user_profile.is_muted,
        )
        .ok();

        let pending_pub_candidates = Rc::new(RefCell::new(Vec::new()));
        let pending_sub_candidates = Rc::new(RefCell::new(Vec::new()));

        let call = ActiveCall {
            channel_id,
            publisher_pc: publisher_pc.clone(),
            subscriber_pc: subscriber_pc.clone(),
            pending_pub_candidates,
            pending_sub_candidates,
            _mic_stream: mic_stream,
            mic_track,
            vad,
            audio_elements,
            audio_ctx,
            _on_publisher_ice: on_publisher_ice,
            _on_subscriber_ice: on_subscriber_ice,
            _on_subscriber_track: on_subscriber_track,
            camera_stream: Rc::new(RefCell::new(None)),
            camera_sender: Rc::new(RefCell::new(None)),
            screen_stream: Rc::new(RefCell::new(None)),
            screen_sender: Rc::new(RefCell::new(None)),
            _on_screen_ended: Rc::new(RefCell::new(None)),
        };

        let cell = get_active_call_cell();
        *cell.borrow_mut() = Some(call);

        voice_ctx
            .connection_status
            .set(VoiceConnectionStatus::Connected);

        let pub_offer_promise = publisher_pc.create_offer();
        let pub_offer_val = wasm_bindgen_futures::JsFuture::from(pub_offer_promise)
            .await
            .map_err(|e| format!("{e:?}"))?;
        let pub_offer_desc: RtcSessionDescriptionInit = pub_offer_val.clone().unchecked_into();
        let pub_offer_sdp = get_sdp_from_js(&pub_offer_val);

        let set_pub_local = publisher_pc.set_local_description(&pub_offer_desc);
        wasm_bindgen_futures::JsFuture::from(set_pub_local)
            .await
            .map_err(|e| format!("{e:?}"))?;

        WsService::send_voice_signal(
            channel_id,
            VoiceSignalData::Offer {
                session: VoiceSessionType::Publisher,
                sdp: pub_offer_sdp,
            },
        )
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn disconnect(voice_ctx: VoiceContext, user_id: Option<UserId>) {
        if let Some(uid) = user_id {
            voice_ctx.channel_participants.update(|map| {
                for list in map.values_mut() {
                    list.retain(|p| p.user_id != uid);
                }
            });
        }

        let call_opt = {
            let cell = get_active_call_cell();
            cell.borrow_mut().take()
        };

        if let Some(call) = call_opt {
            let _ = WsService::left_voice_channel(call.channel_id).await;

            if let Some(vad) = call.vad {
                vad.stop();
            }

            if let Some(ctx) = call.audio_ctx {
                let _ = ctx.close();
            }

            if let Some(cam) = call.camera_stream.borrow_mut().take() {
                for track in cam.get_tracks().iter() {
                    let t: MediaStreamTrack = track.unchecked_into();
                    t.stop();
                }
            }

            if let Some(scr) = call.screen_stream.borrow_mut().take() {
                for track in scr.get_tracks().iter() {
                    let t: MediaStreamTrack = track.unchecked_into();
                    t.stop();
                }
            }

            call.mic_track.stop();
            call.publisher_pc.close();
            call.subscriber_pc.close();

            for el in call.audio_elements.borrow().iter() {
                let _ = el.pause();
                el.set_src_object(None);
                el.remove();
            }
        }

        voice_ctx
            .connection_status
            .set(VoiceConnectionStatus::Disconnected);
        voice_ctx.current_channel_id.set(None);
        voice_ctx.participants.set(Vec::new());
        voice_ctx.is_speaking.set(false);
        voice_ctx.is_camera_on.set(false);
        voice_ctx.is_screen_sharing.set(false);
        voice_ctx.local_camera_stream.set(None);
        voice_ctx.local_screen_stream.set(None);
        voice_ctx
            .remote_video_streams
            .set(std::collections::HashMap::new());
        voice_ctx
            .remote_screen_streams
            .set(std::collections::HashMap::new());
        PENDING_REMOTE_VIDEO_TRACKS.with(|cell| {
            cell.borrow_mut().clear();
        });
        if let Some(uid) = user_id {
            voice_ctx.channel_participants.update(|map| {
                for list in map.values_mut() {
                    list.retain(|p| p.user_id != uid);
                }
            });
        }
    }

    pub fn set_muted(is_muted: bool, voice_ctx: VoiceContext, user_profile: UserProfileContext) {
        user_profile.is_muted.set(is_muted);

        let cell = get_active_call_cell();
        let guard = cell.borrow();
        if let Some(call) = guard.as_ref() {
            call.mic_track.set_enabled(!is_muted);
            let cid = call.channel_id;
            let deafened = user_profile.is_deafened.get();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = WsService::update_voice_state(cid, is_muted, deafened).await;
            });
        }

        if is_muted {
            voice_ctx.is_speaking.set(false);
        }
    }

    pub fn set_deafened(
        is_deafened: bool,
        voice_ctx: VoiceContext,
        user_profile: UserProfileContext,
    ) {
        user_profile.is_deafened.set(is_deafened);

        if is_deafened {
            user_profile.is_muted.set(true);
        }

        let cell = get_active_call_cell();
        let guard = cell.borrow();
        if let Some(call) = guard.as_ref() {
            if is_deafened {
                call.mic_track.set_enabled(false);
            } else if !user_profile.is_muted.get() {
                call.mic_track.set_enabled(true);
            }

            let vol = if is_deafened { 0.0 } else { 1.0 };
            for el in call.audio_elements.borrow().iter() {
                el.set_volume(vol);
            }

            if let Some(ref ctx) = call.audio_ctx {
                if is_deafened {
                    let _ = ctx.suspend();
                } else {
                    let _ = ctx.resume();
                }
            }

            let cid = call.channel_id;
            let muted = user_profile.is_muted.get();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = WsService::update_voice_state(cid, muted, is_deafened).await;
            });
        }

        if is_deafened {
            voice_ctx.is_speaking.set(false);
        }
    }

    pub async fn handle_signal(signal: VoiceSignalData) -> Result<(), String> {
        let (channel_id, publisher_pc, subscriber_pc, pending_pub_ice, pending_sub_ice) = {
            let cell = get_active_call_cell();
            let guard = cell.borrow();
            let call = guard
                .as_ref()
                .ok_or_else(|| "No active call for incoming signal".to_string())?;
            (
                call.channel_id,
                call.publisher_pc.clone(),
                call.subscriber_pc.clone(),
                call.pending_pub_candidates.clone(),
                call.pending_sub_candidates.clone(),
            )
        };

        match signal {
            VoiceSignalData::Answer { session, sdp } => {
                web_sys::console::log_1(
                    &format!(
                        "[VoiceManager] Received VoiceSignalData::Answer for session={session:?}"
                    )
                    .into(),
                );
                let desc = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
                desc.set_sdp(&sdp);
                let (target_pc, pending_ice) = match session {
                    VoiceSessionType::Publisher => (publisher_pc, pending_pub_ice),
                    VoiceSessionType::Subscriber => (subscriber_pc, pending_sub_ice),
                };
                let promise = target_pc.set_remote_description(&desc);
                wasm_bindgen_futures::JsFuture::from(promise)
                    .await
                    .map_err(|e| format!("{e:?}"))?;

                let pending = pending_ice.borrow_mut().drain(..).collect::<Vec<_>>();
                for cand_init in pending {
                    let _ = wasm_bindgen_futures::JsFuture::from(
                        target_pc
                            .add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(&cand_init)),
                    )
                    .await;
                }
            }
            VoiceSignalData::Offer { session, sdp } => {
                web_sys::console::log_1(
                    &format!(
                        "[VoiceManager] Received VoiceSignalData::Offer for session={session:?}"
                    )
                    .into(),
                );
                if session == VoiceSessionType::Subscriber {
                    let offer_desc = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
                    offer_desc.set_sdp(&sdp);
                    let promise = subscriber_pc.set_remote_description(&offer_desc);
                    wasm_bindgen_futures::JsFuture::from(promise)
                        .await
                        .map_err(|e| format!("{e:?}"))?;

                    let pending = pending_sub_ice.borrow_mut().drain(..).collect::<Vec<_>>();
                    for cand_init in pending {
                        let _ = wasm_bindgen_futures::JsFuture::from(
                            subscriber_pc.add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(
                                &cand_init,
                            )),
                        )
                        .await;
                    }

                    let answer_promise = subscriber_pc.create_answer();
                    let answer_val = wasm_bindgen_futures::JsFuture::from(answer_promise)
                        .await
                        .map_err(|e| format!("{e:?}"))?;
                    let answer_desc: RtcSessionDescriptionInit =
                        answer_val.clone().unchecked_into();
                    let answer_sdp = get_sdp_from_js(&answer_val);

                    let set_local = subscriber_pc.set_local_description(&answer_desc);
                    wasm_bindgen_futures::JsFuture::from(set_local)
                        .await
                        .map_err(|e| format!("{e:?}"))?;

                    web_sys::console::log_1(
                        &"[VoiceManager] Sending VoiceSignalData::Answer for Subscriber".into(),
                    );
                    WsService::send_voice_signal(
                        channel_id,
                        VoiceSignalData::Answer {
                            session: VoiceSessionType::Subscriber,
                            sdp: answer_sdp,
                        },
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                }
            }
            VoiceSignalData::IceCandidate {
                session,
                candidate,
                sdp_mid,
                sdp_mline_index,
            } => {
                let cand_init = RtcIceCandidateInit::new(&candidate);
                if let Some(ref mid) = sdp_mid {
                    cand_init.set_sdp_mid(Some(mid));
                }
                if let Some(idx) = sdp_mline_index {
                    cand_init.set_sdp_m_line_index(Some(idx));
                }

                let (target_pc, pending_ice) = match session {
                    VoiceSessionType::Publisher => (publisher_pc, pending_pub_ice),
                    VoiceSessionType::Subscriber => (subscriber_pc, pending_sub_ice),
                };

                if target_pc.remote_description().is_some() {
                    let promise = target_pc
                        .add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(&cand_init));
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                } else {
                    pending_ice.borrow_mut().push(cand_init);
                }
            }
        }

        Ok(())
    }

    pub fn apply_codec_preference(
        transceiver: &web_sys::RtcRtpTransceiver,
        pref: gilvave_core::dto::voice::VideoCodecPreference,
    ) {
        let target_mime = match pref.mime_type() {
            Some(m) => m,
            None => return,
        };

        let capabilities = match web_sys::RtcRtpReceiver::get_capabilities("video") {
            Some(c) => c,
            None => return,
        };

        let codecs_arr = capabilities.get_codecs();
        let preferred = js_sys::Array::new();
        let others = js_sys::Array::new();

        for i in 0..codecs_arr.length() {
            let codec_val = codecs_arr.get(i);
            let codec: web_sys::RtcRtpCodecCapability = codec_val.clone().unchecked_into();
            let mime = codec.get_mime_type();
            if mime.eq_ignore_ascii_case(target_mime) {
                preferred.push(&codec_val);
            } else {
                others.push(&codec_val);
            }
        }

        let final_codecs = preferred.concat(&others);
        transceiver.set_codec_preferences(&final_codecs);
    }

    pub async fn set_codec_preference(
        pref: gilvave_core::dto::voice::VideoCodecPreference,
    ) -> Result<(), String> {
        let (channel_id, publisher_pc) = {
            let cell = get_active_call_cell();
            let guard = cell.borrow();
            let call = match guard.as_ref() {
                Some(c) => c,
                None => return Ok(()),
            };
            (call.channel_id, call.publisher_pc.clone())
        };

        for trans in publisher_pc.get_transceivers().iter() {
            let t: web_sys::RtcRtpTransceiver = trans.unchecked_into();
            let kind = t.receiver().track().kind();
            if kind == "video" {
                Self::apply_codec_preference(&t, pref);
            }
        }

        Self::renegotiate_publisher(channel_id, &publisher_pc).await?;
        Ok(())
    }

    pub fn try_match_pending_video_tracks(
        user_id: UserId,
        is_camera_on: bool,
        is_screen_sharing: bool,
        voice_ctx: VoiceContext,
    ) {
        PENDING_REMOTE_VIDEO_TRACKS.with(|cell| {
            let mut list = cell.borrow_mut();
            if list.is_empty() {
                return;
            }

            if is_screen_sharing
                && !voice_ctx
                    .remote_screen_streams
                    .with(|m| m.contains_key(&user_id))
            {
                let idx = list
                    .iter()
                    .position(|(_, _, s_id, t_id)| {
                        s_id.contains("screen") || t_id.contains("screen")
                    })
                    .or_else(|| if is_camera_on { None } else { Some(0) });

                if let Some(i) = idx
                    && i < list.len()
                {
                    let (stream, _, _, _) = list.remove(i);
                    web_sys::console::log_1(
                        &format!("[VoiceManager] Matched pending screen track for user {user_id}")
                            .into(),
                    );
                    voice_ctx.remote_screen_streams.update(|map| {
                        map.insert(user_id, stream);
                    });
                }
            }

            if is_camera_on
                && !voice_ctx
                    .remote_video_streams
                    .with(|m| m.contains_key(&user_id))
            {
                let idx = list
                    .iter()
                    .position(|(_, _, s_id, t_id)| {
                        s_id.contains("camera") || t_id.contains("camera")
                    })
                    .or_else(|| Some(0));

                if let Some(i) = idx
                    && i < list.len()
                {
                    let (stream, _, _, _) = list.remove(i);
                    web_sys::console::log_1(
                        &format!("[VoiceManager] Matched pending camera track for user {user_id}")
                            .into(),
                    );
                    voice_ctx.remote_video_streams.update(|map| {
                        map.insert(user_id, stream);
                    });
                }
            }
        });
    }

    pub async fn toggle_camera(voice_ctx: VoiceContext) -> Result<(), String> {
        let (channel_id, publisher_pc, cam_stream_cell, cam_sender_cell) = {
            let cell = get_active_call_cell();
            let guard = cell.borrow();
            let call = guard
                .as_ref()
                .ok_or_else(|| "No active call for camera toggle".to_string())?;
            (
                call.channel_id,
                call.publisher_pc.clone(),
                call.camera_stream.clone(),
                call.camera_sender.clone(),
            )
        };

        if voice_ctx.is_camera_on.get() {
            if let Some(stream) = cam_stream_cell.borrow_mut().take() {
                let tracks = stream.get_tracks();
                for i in 0..tracks.length() {
                    let t: MediaStreamTrack = tracks.get(i).unchecked_into();
                    t.stop();
                }
            }
            if let Some(stream) = voice_ctx.local_camera_stream.get_clone() {
                let tracks = stream.get_tracks();
                for i in 0..tracks.length() {
                    let t: MediaStreamTrack = tracks.get(i).unchecked_into();
                    t.stop();
                }
            }
            if let Some(sender) = cam_sender_cell.borrow_mut().take() {
                if let Some(track) = sender.track() {
                    track.stop();
                }
                publisher_pc.remove_track(&sender);
            }

            voice_ctx.is_camera_on.set(false);
            voice_ctx.local_camera_stream.set(None);

            let is_scr = voice_ctx.is_screen_sharing.get();
            let _ = WsService::update_voice_video_state(channel_id, false, is_scr).await;

            Self::renegotiate_publisher(channel_id, &publisher_pc).await?;
        } else {
            let window = web_sys::window().ok_or_else(|| "Window not found".to_string())?;
            let nav = window.navigator();
            let media_devices = nav.media_devices().map_err(|e| format!("{e:?}"))?;

            let constraints = web_sys::MediaStreamConstraints::new();
            constraints.set_audio(&false.into());
            constraints.set_video(&true.into());

            let stream_promise = media_devices
                .get_user_media_with_constraints(&constraints)
                .map_err(|e| format!("Camera access error: {e:?}"))?;

            let stream_js = wasm_bindgen_futures::JsFuture::from(stream_promise)
                .await
                .map_err(|e| format!("Failed to acquire camera: {e:?}"))?;

            let stream: MediaStream = stream_js.unchecked_into();
            let video_tracks = stream.get_video_tracks();
            if video_tracks.length() == 0 {
                return Err("No video track found in camera stream".to_string());
            }
            let track: MediaStreamTrack = video_tracks.get(0).unchecked_into();

            let sender = publisher_pc.add_track_0(&track, &stream);

            for trans in publisher_pc.get_transceivers().iter() {
                let t: web_sys::RtcRtpTransceiver = trans.unchecked_into();
                if t.sender() == sender {
                    Self::apply_codec_preference(&t, voice_ctx.video_codec_preference.get());
                }
            }

            *cam_stream_cell.borrow_mut() = Some(stream.clone());
            *cam_sender_cell.borrow_mut() = Some(sender);

            voice_ctx.is_camera_on.set(true);
            voice_ctx.local_camera_stream.set(Some(stream));

            let is_scr = voice_ctx.is_screen_sharing.get();
            let _ = WsService::update_voice_video_state(channel_id, true, is_scr).await;

            Self::renegotiate_publisher(channel_id, &publisher_pc).await?;
        }

        Ok(())
    }

    pub async fn toggle_screen_share(voice_ctx: VoiceContext) -> Result<(), String> {
        if voice_ctx.is_screen_sharing.get() {
            Self::stop_screen_share(voice_ctx).await
        } else {
            Self::start_screen_share(voice_ctx).await
        }
    }

    async fn start_screen_share(voice_ctx: VoiceContext) -> Result<(), String> {
        let (channel_id, publisher_pc, scr_stream_cell, scr_sender_cell, on_ended_cell) = {
            let cell = get_active_call_cell();
            let guard = cell.borrow();
            let call = guard
                .as_ref()
                .ok_or_else(|| "No active call for screen share".to_string())?;
            (
                call.channel_id,
                call.publisher_pc.clone(),
                call.screen_stream.clone(),
                call.screen_sender.clone(),
                call._on_screen_ended.clone(),
            )
        };

        let window = web_sys::window().ok_or_else(|| "Window not found".to_string())?;
        let nav = window.navigator();
        let media_devices = nav.media_devices().map_err(|e| format!("{e:?}"))?;

        let stream_promise = media_devices
            .get_display_media()
            .map_err(|e| format!("Display media error: {e:?}"))?;

        let stream_js = wasm_bindgen_futures::JsFuture::from(stream_promise)
            .await
            .map_err(|e| format!("Failed to acquire screen: {e:?}"))?;

        let stream: MediaStream = stream_js.unchecked_into();
        let video_tracks = stream.get_video_tracks();
        if video_tracks.length() == 0 {
            return Err("No video track found in screen stream".to_string());
        }
        let track: MediaStreamTrack = video_tracks.get(0).unchecked_into();

        let v_ctx = voice_ctx;
        let on_ended = Closure::<dyn Fn()>::new(move || {
            wasm_bindgen_futures::spawn_local(async move {
                let _ = VoiceManager::stop_screen_share(v_ctx).await;
            });
        });
        track.set_onended(Some(on_ended.as_ref().unchecked_ref()));
        *on_ended_cell.borrow_mut() = Some(on_ended);

        let sender = publisher_pc.add_track_0(&track, &stream);

        for trans in publisher_pc.get_transceivers().iter() {
            let t: web_sys::RtcRtpTransceiver = trans.unchecked_into();
            if t.sender() == sender {
                Self::apply_codec_preference(&t, voice_ctx.video_codec_preference.get());
            }
        }

        *scr_stream_cell.borrow_mut() = Some(stream.clone());
        *scr_sender_cell.borrow_mut() = Some(sender);

        voice_ctx.is_screen_sharing.set(true);
        voice_ctx.local_screen_stream.set(Some(stream));

        let is_cam = voice_ctx.is_camera_on.get();
        let _ = WsService::update_voice_video_state(channel_id, is_cam, true).await;

        Self::renegotiate_publisher(channel_id, &publisher_pc).await?;

        Ok(())
    }

    pub async fn stop_screen_share(voice_ctx: VoiceContext) -> Result<(), String> {
        let (channel_id, publisher_pc, scr_stream_cell, scr_sender_cell, on_ended_cell) = {
            let cell = get_active_call_cell();
            let guard = cell.borrow();
            let call = guard
                .as_ref()
                .ok_or_else(|| "No active call for screen share".to_string())?;
            (
                call.channel_id,
                call.publisher_pc.clone(),
                call.screen_stream.clone(),
                call.screen_sender.clone(),
                call._on_screen_ended.clone(),
            )
        };

        if let Some(stream) = scr_stream_cell.borrow_mut().take() {
            let tracks = stream.get_tracks();
            for i in 0..tracks.length() {
                let t: MediaStreamTrack = tracks.get(i).unchecked_into();
                t.stop();
            }
        }
        if let Some(stream) = voice_ctx.local_screen_stream.get_clone() {
            let tracks = stream.get_tracks();
            for i in 0..tracks.length() {
                let t: MediaStreamTrack = tracks.get(i).unchecked_into();
                t.stop();
            }
        }
        if let Some(sender) = scr_sender_cell.borrow_mut().take() {
            if let Some(track) = sender.track() {
                track.stop();
            }
            publisher_pc.remove_track(&sender);
        }
        *on_ended_cell.borrow_mut() = None;

        voice_ctx.is_screen_sharing.set(false);
        voice_ctx.local_screen_stream.set(None);

        let is_cam = voice_ctx.is_camera_on.get();
        let _ = WsService::update_voice_video_state(channel_id, is_cam, false).await;

        Self::renegotiate_publisher(channel_id, &publisher_pc).await?;

        Ok(())
    }

    async fn renegotiate_publisher(
        channel_id: ChannelId,
        publisher_pc: &RtcPeerConnection,
    ) -> Result<(), String> {
        let offer_promise = publisher_pc.create_offer();
        let offer_val = wasm_bindgen_futures::JsFuture::from(offer_promise)
            .await
            .map_err(|e| format!("{e:?}"))?;
        let offer_desc: RtcSessionDescriptionInit = offer_val.clone().unchecked_into();
        let offer_sdp = get_sdp_from_js(&offer_val);

        let set_pub_local = publisher_pc.set_local_description(&offer_desc);
        wasm_bindgen_futures::JsFuture::from(set_pub_local)
            .await
            .map_err(|e| format!("{e:?}"))?;

        WsService::send_voice_signal(
            channel_id,
            VoiceSignalData::Offer {
                session: VoiceSessionType::Publisher,
                sdp: offer_sdp,
            },
        )
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }
}

fn create_rtc_configuration() -> Result<RtcConfiguration, JsValue> {
    let config = RtcConfiguration::new();
    let ice_servers = js_sys::Array::new();
    let ice_server = RtcIceServer::new();
    let urls = js_sys::Array::new();
    urls.push(&js_sys::JsString::from("stun:stun.l.google.com:19302"));
    ice_server.set_urls(&urls.into());
    ice_servers.push(&ice_server.into());
    config.set_ice_servers(&ice_servers.into());
    Ok(config)
}
