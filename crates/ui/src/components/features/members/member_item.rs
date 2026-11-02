use gilvave_core::dto::servers::MemberView;
use sycamore::prelude::*;

use crate::components::ui::icons::UserAvatar;

#[component(inline_props)]
pub fn MemberItem(member_view: MemberView) -> View {
    let avatar = if member_view.avatar.is_empty() {
        view! { UserAvatar() }
    } else {
        view! { img(class="avatar", src=member_view.avatar) }
    };
    let u_id_str = member_view.user_id.to_string();
    let u_name = member_view.username.clone();
    view! {
        div(
            class="member-item",
            data-user-id=u_id_str,
            data-username=u_name,
        ) {
            (avatar)
            (member_view.username)
        }
    }
}
