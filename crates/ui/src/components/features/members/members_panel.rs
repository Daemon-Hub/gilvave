use sycamore::{futures::spawn_local_scoped, prelude::*};

use crate::{components::common::ServerContext, gateway::service::WsService, http::api::Api};

use super::member_item::MemberItem;

#[component()]
pub fn MembersPanel() -> View {
    let server_context = use_context::<ServerContext>();

    let current_server_id = create_memo(move || {
        server_context
            .current
            .with(|s| s.as_ref().map(|srv| srv.id))
    });

    create_effect(move || {
        if let Some(server_id) = current_server_id.get() {
            spawn_local_scoped(async move {
                let _ = WsService::server_online(server_id).await;
                if let Ok(members) = Api::get_members(server_id).await {
                    server_context.members.set(members);
                }
            })
        } else {
            server_context.members.set(vec![]);
            server_context
                .online_users
                .set(std::collections::HashSet::new());
        }
    });

    let online_members = create_memo(move || {
        let online_set = server_context.online_users.get_clone();
        server_context.members.with(|members| {
            members
                .iter()
                .filter(|m| online_set.contains(&m.user_id))
                .cloned()
                .collect::<Vec<_>>()
        })
    });

    let offline_members = create_memo(move || {
        let online_set = server_context.online_users.get_clone();
        server_context.members.with(|members| {
            members
                .iter()
                .filter(|m| !online_set.contains(&m.user_id))
                .cloned()
                .collect::<Vec<_>>()
        })
    });

    let online_count = create_memo(move || online_members.with(|m| m.len()));
    let offline_count = create_memo(move || offline_members.with(|m| m.len()));

    view! {
        div(class="discord-members-panel") {
            div(class="panel-section") {
                h3 { (format!("Онлайн — {}", online_count.get())) }
                Keyed(
                    list=online_members,
                    view=|member| { view! { MemberItem(member_view=member, is_online=true) } },
                    key=|member| member.user_id,
                )
            }

            (if offline_count.get() > 0 {
                view! {
                    div(class="panel-section") {
                        h3 { (format!("Офлайн — {}", offline_count.get())) }
                        Keyed(
                            list=offline_members,
                            view=|member| { view! { MemberItem(member_view=member, is_online=false) } },
                            key=|member| member.user_id,
                        )
                    }
                }
            } else {
                view! {}
            })
        }
    }
}
