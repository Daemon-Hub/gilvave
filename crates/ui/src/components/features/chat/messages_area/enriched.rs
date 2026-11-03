//! Enriched message presentation logic (grouping, date dividers, avatars, unread markers).

use gilvave_core::{
    dto::{messages::MessageView, servers::MemberView},
    ids::MessageId,
};

use crate::{
    components::features::chat::chat_scroll::format_date_divider,
    utils::get_local_offset,
};

#[derive(Clone, PartialEq)]
pub struct EnrichedMessage {
    pub message: MessageView,
    pub avatar_url: String,
    pub is_first: bool,
    pub is_last: bool,
    pub date_divider: Option<String>,
    pub is_unread_divider: bool,
}

#[inline]
pub fn hash_str(s: &str) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    std::hash::Hash::hash(s, &mut hasher);
    std::hash::Hasher::finish(&hasher)
}

pub fn compute_enriched_messages(
    msgs: &[MessageView],
    members: &[MemberView],
    my_username: &str,
    my_avatar: &str,
    cur_unread_marker: Option<MessageId>,
) -> Vec<EnrichedMessage> {
    let len = msgs.len();
    let local_offset = get_local_offset();
    let mut local_dts: Vec<time::OffsetDateTime> = Vec::with_capacity(len);
    for m in msgs.iter() {
        local_dts.push(m.created_at.to_offset(local_offset));
    }

    let mut avatar_by_username = std::collections::HashMap::with_capacity(members.len());
    let mut avatar_by_id = std::collections::HashMap::with_capacity(members.len());
    for mem in members.iter() {
        if !mem.avatar.is_empty() {
            avatar_by_username.insert(mem.username.as_str(), mem.avatar.as_str());
            avatar_by_id.insert(mem.user_id, mem.avatar.as_str());
        }
    }

    msgs.iter()
        .enumerate()
        .map(|(i, m)| {
            let m_local_dt = local_dts[i];
            let m_date = m_local_dt.date();
            let is_new_day = if i == 0 {
                true
            } else {
                local_dts[i - 1].date() != m_date
            };
            let date_divider = if is_new_day {
                Some(format_date_divider(m_date))
            } else {
                None
            };

            let is_first = if i == 0 {
                true
            } else {
                let prev_dt = local_dts[i - 1];
                let prev_m = &msgs[i - 1];
                prev_m.author_name != m.author_name
                    || (m.author_id.is_some() && prev_m.author_id != m.author_id)
                    || prev_dt.date() != m_date
                    || (m_local_dt - prev_dt) > time::Duration::minutes(5)
            };

            let is_last = if i + 1 == len {
                true
            } else {
                let next_dt = local_dts[i + 1];
                let next_m = &msgs[i + 1];
                next_m.author_name != m.author_name
                    || (m.author_id.is_some() && next_m.author_id != m.author_id)
                    || next_dt.date() != m_date
                    || (next_dt - m_local_dt) > time::Duration::minutes(5)
            };

            let avatar = if !my_avatar.is_empty() && m.author_name == my_username {
                my_avatar.to_string()
            } else if let Some(author_id) = m.author_id
                && let Some(&av) = avatar_by_id.get(&author_id)
            {
                av.to_string()
            } else if let Some(&av) = avatar_by_username.get(m.author_name.as_str()) {
                av.to_string()
            } else if m.author_name == my_username {
                my_avatar.to_string()
            } else {
                String::new()
            };

            let is_unread_divider = if let Some(marker_id) = cur_unread_marker {
                m.id == marker_id
            } else {
                false
            };

            EnrichedMessage {
                message: m.clone(),
                avatar_url: avatar,
                is_first,
                is_last,
                date_divider,
                is_unread_divider,
            }
        })
        .collect()
}
