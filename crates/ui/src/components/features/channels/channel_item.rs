use sycamore::{futures::spawn_local_scoped, prelude::*, web::console_error};

use crate::{
    components::{
        common::{ChannelContext, VoiceContext, classes},
        ui::icons::{HashIcon, Volume2Icon},
    },
    gateway::service::WsService,
};
use gilvave_core::dto::servers::channels::{ChannelType, ChannelView};

#[component(inline_props)]
fn MiniVoiceMember(participant: gilvave_core::dto::voice::VoiceParticipantView) -> View {
    let initial = participant
        .username
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    let is_speaking = participant.is_speaking && !participant.is_muted;
    let avatar = participant.avatar;
    let username = participant.username;

    let avatar_view = if !avatar.is_empty() {
        view! { img(src=avatar, class="voice-member-avatar", alt="") }
    } else {
        view! { span(class="voice-member-initial") { (initial) } }
    };

    view! {
        div(class=classes(vec![
            "channel-voice-member".into(),
            ("speaking", is_speaking.into()).into(),
            ("muted", participant.is_muted.into()).into(),
        ])) {
            (avatar_view)
            span(class="voice-member-name") { (username) }
        }
    }
}

#[component(inline_props)]
pub fn ChannelItem(channel_view: ChannelView) -> View {
    let channel_name = channel_view.name.clone();
    let channel_name_attr = channel_name.clone();
    let channel_id = channel_view.id;
    let channel_type = channel_view.r#type;
    let is_voice = channel_type == ChannelType::VOICE;
    let channel_signal = create_signal(channel_view);

    let context = use_context::<ChannelContext>();
    let voice_ctx = use_context::<VoiceContext>();

    let is_active = create_memo(move || {
        context
            .current
            .with(|c| c.as_ref().is_some_and(|channel| channel.id == channel_id))
    });

    let channel_members = create_memo(move || {
        voice_ctx
            .channel_participants
            .with(|map| map.get(&channel_id).cloned().unwrap_or_default())
    });

    let has_voice_members =
        create_memo(move || is_voice && !channel_members.with(|m| m.is_empty()));

    let on_click = move |_| {
        spawn_local_scoped(async move {
            let channel_item = channel_signal.get_clone();
            let channel_item_id = channel_item.id;

            let context = use_context::<ChannelContext>();
            if let Some(prev) = context.current.with(|c| c.as_ref().cloned()) {
                if prev.id == channel_item_id {
                    return;
                }

                if prev.r#type == ChannelType::TEXT {
                    let prev_id = prev.id;
                    spawn_local_scoped(async move {
                        let _ = WsService::left_channel(prev_id).await;
                    });
                }
            }

            if channel_item.r#type == ChannelType::TEXT {
                web_sys::console::log_1(
                    &format!("[CHANNEL_ITEM] clicked text channel: {channel_item_id}").into(),
                );
                context.current.set(Some(channel_item));

                let res = WsService::join_channel(channel_item_id).await;
                if let Err(err) = res {
                    console_error!("join channel error: {err:#?}");
                }

                let _ = WsService::channel_history_before(
                    channel_item_id,
                    time::OffsetDateTime::now_utc(),
                )
                .await;
            } else {
                web_sys::console::log_1(
                    &format!("[CHANNEL_ITEM] clicked voice channel: {channel_item_id}").into(),
                );
                context.current.set(Some(channel_item));
            }
        });
    };

    view! {
        div(class="channel-item-wrapper") {
            div(
                class=classes(vec![
                    "channel-item".into(),
                    ("voice", is_voice.into()).into(),
                    ("active", is_active.into()).into(),
                ]),
                data-channel-id=channel_id.to_string(),
                data-channel-name=channel_name_attr,
                on:click=on_click,
            ) {
                span(class="channel-icon") {
                    (if is_voice {
                        view! { Volume2Icon() }
                    } else {
                        view! { HashIcon() }
                    })
                }
                span(class="channel-name-text") {
                    (channel_name)
                }
            }

            (if has_voice_members.get() {
                view! {
                    div(class="channel-voice-members") {
                        Keyed(
                            list=channel_members,
                            view=|p| view! { MiniVoiceMember(participant=p) },
                            key=|p| p.user_id,
                        )
                    }
                }
            } else {
                view! {}
            })
        }
    }
}
