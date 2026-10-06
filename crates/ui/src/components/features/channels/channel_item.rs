use gilvave_core::dto::channel::{ChannelType, ChannelView};
use sycamore::{futures::spawn_local_scoped, prelude::*, web::console_error};

use crate::{
    components::{
        common::{classes, ChannelContext},
        ui::icons::{HashIcon, Volume2Icon},
    },
    gateway::service::WsService,
};

#[component(inline_props)]
pub fn ChannelItem(channel_view: ChannelView) -> View {
    let channel_name = channel_view.name.clone();
    let channel_id = channel_view.id;
    let channel_type = channel_view.r#type;
    let is_voice = channel_type == ChannelType::VOICE;
    let channel_signal = create_signal(channel_view);

    let context = use_context::<ChannelContext>();
    let is_active = create_memo(move || {
        context
            .current
            .with(|c| c.as_ref().is_some_and(|channel| channel.id == channel_id))
    });

    let on_click = move |_| {
        spawn_local_scoped(async move {
            let channel_item = channel_signal.get_clone();
            let channel_item_id = channel_item.id;

            let context = use_context::<ChannelContext>();
            if let Some(prev) = context.current.with(|c| c.as_ref().cloned()) {
                if prev.id == channel_item_id {
                    return;
                }

                context.current.set(None);
                context.messages.set(vec![]);
                if prev.r#type == ChannelType::TEXT {
                    let _ = WsService::left_channel(prev.id).await;
                }
            }

            if channel_item.r#type == ChannelType::TEXT {
                web_sys::console::log_1(&format!("[CHANNEL_ITEM] clicked text channel: {channel_item_id}").into());
                let res = WsService::join_channel(channel_item_id).await;
                match res {
                    Ok(()) => {
                        context.current.set(Some(channel_item));
                    }
                    Err(err) => {
                        console_error!("join channel error: {err:#?}");
                    }
                }

                let _ = WsService::channel_history_before(
                    channel_item_id,
                    time::OffsetDateTime::now_utc(),
                )
                .await;
            } else {
                web_sys::console::log_1(&format!("[CHANNEL_ITEM] clicked voice channel: {channel_item_id}").into());
                // TODO [WS]: Отправка ServerSend::SubscribeVoiceChannel { channel_id: channel_item_id }
                // Подписка на поток статусов участников в данном голосовом канале для отображения их списка в сайдбаре.
                //
                // TODO [HTTP]: GET /channels/{channel_id}/voice/status
                // Запрос текущего количества участников и битрейта для предпросмотра в боковой панели.
                context.current.set(Some(channel_item));
            }
        });
    };

    view! {
        div(
            class=classes(vec![
                "channel-item".into(),
                ("voice", is_voice.into()).into(),
                ("active", is_active.into()).into(),
            ]),
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
    }
}
