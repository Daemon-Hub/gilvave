use gilvave_core::dto::server::MemberView;
use sycamore::prelude::*;

use crate::components::ui::icons::UserAvatar;

#[component(inline_props)]
pub fn MemberItem(member_view: MemberView) -> View {
    let avatar = if member_view.avatar.is_empty() {
        view! { UserAvatar() }
    } else {
        view! { img(class="avatar", src=member_view.avatar) }
    };
    view! {
        div(class="member-item") {
            (avatar)
            (member_view.username)
        }
    }
}
