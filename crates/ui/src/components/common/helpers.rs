use sycamore::prelude::*;

/// Правило добавления класса
pub enum ClassRule {
    /// Всегда включать этот класс
    Always(&'static str),
    /// Включать класс только если условие истинно
    IfTrue(&'static str, MaybeDyn<bool>),
    /// Включать один из двух классов в зависимости от условия
    Ternary(MaybeDyn<bool>, &'static str, &'static str),
}

// Эргономичные конверсии
impl From<&'static str> for ClassRule {
    fn from(class: &'static str) -> Self {
        ClassRule::Always(class)
    }
}

impl From<(&'static str, MaybeDyn<bool>)> for ClassRule {
    fn from((class, cond): (&'static str, MaybeDyn<bool>)) -> Self {
        ClassRule::IfTrue(class, cond)
    }
}

impl From<(MaybeDyn<bool>, &'static str, &'static str)> for ClassRule {
    fn from(
        (condition, if_class, else_class): (MaybeDyn<bool>, &'static str, &'static str),
    ) -> Self {
        ClassRule::Ternary(condition, if_class, else_class)
    }
}

/// Создаёт реактивное замыкание для генерации классов.
///
/// Возвращает `impl Fn() -> String`, который Sycamore автоматически конвертирует в `StringAttribute`.
pub fn classes(rules: Vec<ClassRule>) -> impl Fn() -> String {
    move || {
        rules
            .iter()
            .filter_map(|rule| match rule {
                ClassRule::Always(class) => Some(*class),
                ClassRule::IfTrue(class, cond) => {
                    if cond.get() {
                        Some(*class)
                    } else {
                        None
                    }
                }
                ClassRule::Ternary(cond, true_class, false_class) => {
                    if cond.get() {
                        Some(*true_class)
                    } else {
                        Some(*false_class)
                    }
                }
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Проверяет, запущено ли приложение на мобильном устройстве (по ширине экрана, media query или user-agent).
pub fn is_mobile_device() -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    if let Ok(width_val) = window.inner_width() {
        if let Some(w) = width_val.as_f64() {
            if w <= 768.0 {
                return true;
            }
        }
    }
    if let Ok(ua) = window.navigator().user_agent() {
        let ua_lower = ua.to_lowercase();
        if ua_lower.contains("mobile")
            || ua_lower.contains("android")
            || ua_lower.contains("iphone")
            || ua_lower.contains("ipad")
        {
            return true;
        }
    }
    false
}
