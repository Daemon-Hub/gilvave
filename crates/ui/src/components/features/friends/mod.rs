pub mod add_friend_tab;
pub mod all_friends_tab;
pub mod blocked_tab;
pub mod friends_panel;
pub mod requests_tab;

pub use friends_panel::FriendsPanel;

pub(crate) fn format_friend_date(dt: time::OffsetDateTime) -> String {
    let local = crate::utils::to_local_datetime(dt);
    let month_str = match local.month() {
        time::Month::January => "января",
        time::Month::February => "февраля",
        time::Month::March => "марта",
        time::Month::April => "апреля",
        time::Month::May => "мая",
        time::Month::June => "июня",
        time::Month::July => "июля",
        time::Month::August => "августа",
        time::Month::September => "сентября",
        time::Month::October => "октября",
        time::Month::November => "ноября",
        time::Month::December => "декабря",
    };
    format!("{} {} {}, {:02}:{:02}", local.day(), month_str, local.year(), local.hour(), local.minute())
}
