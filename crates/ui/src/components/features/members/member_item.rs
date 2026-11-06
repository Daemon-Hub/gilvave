use gilvave_core::dto::servers::MemberView;
use sycamore::prelude::*;

use crate::components::ui::icons::UserAvatar;

#[component(inline_props)]
pub fn MemberItem(member_view: MemberView, is_online: bool) -> View {
    let avatar = if member_view.avatar.is_empty() {
        view! { UserAvatar() }
    } else {
        view! { img(class="avatar", src=member_view.avatar) }
    };
    let u_id_str = member_view.user_id.to_string();
    let u_name = member_view.username.clone();
    let item_class = if is_online {
        "member-item"
    } else {
        "member-item offline"
    };
    let status_class = if is_online {
        "avatar-status avatar-status--online"
    } else {
        "avatar-status avatar-status--offline"
    };

    view! {
        div(
            class=item_class,
            data-user-id=u_id_str,
            data-username=u_name,
        ) {
            div(class="avatar-wrapper") {
                (avatar)
                div(class=status_class)
            }
            div(class="member-info") {
                span(class="member-name") { (member_view.username) }
            }
        }
    }
}
