//! Automatic derivation of 120+ CSS custom properties from a custom theme palette.

use std::collections::BTreeMap;
use super::css_engine::{darken_color, is_allowed_css_var, is_safe_css_value, with_alpha};
use super::custom_theme::ThemePalette;

pub(crate) fn build_palette_variable_map(
    palette: &ThemePalette,
    custom_variables: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut vars = BTreeMap::new();
    let p = palette;

    let clean = |opt: &Option<String>| -> Option<String> {
        opt.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty() && is_safe_css_value(s))
            .map(ToString::to_string)
    };

    let bg_page = clean(&p.bg_page);
    let bg_sidebar = clean(&p.bg_sidebar);
    let bg_channels = clean(&p.bg_channels);
    let bg_chat = clean(&p.bg_chat);
    let bg_input = clean(&p.bg_input);
    let bg_modal = clean(&p.bg_modal);
    let bg_message = clean(&p.bg_message);
    let border = clean(&p.border);
    let primary = clean(&p.primary);
    let primary_end = clean(&p.primary_end).or_else(|| primary.clone());
    let accent = clean(&p.accent).or_else(|| primary.clone());
    let accent_warm = clean(&p.accent_warm)
        .or_else(|| accent.clone())
        .or_else(|| primary.clone());
    let text_heading = clean(&p.text_heading);
    let text_chat = clean(&p.text_chat);
    let text_muted = clean(&p.text_muted);
    let text_author = clean(&p.text_author).or_else(|| accent.clone());
    let online = clean(&p.online);
    let idle = clean(&p.idle);
    let danger = clean(&p.danger);

    if let Some(ref c) = bg_page {
        vars.insert("--color-bg-page".into(), c.clone());
        vars.insert("--color-bg-page-mid".into(), c.clone());
        vars.insert("--color-bg-page-end".into(), c.clone());
        vars.insert("--color-bg-page-overlay".into(), with_alpha(c, 0.72));
        let mid = bg_channels.as_ref().or(bg_chat.as_ref()).unwrap_or(c);
        vars.insert(
            "--gradient-body-main".into(),
            format!("linear-gradient(135deg, {c}, {mid}, {c})"),
        );
        vars.insert("--gradient-body-before".into(), "none".into());
        vars.insert("--gradient-body-after".into(), "none".into());
    }

    if let Some(ref c) = bg_sidebar {
        vars.insert("--color-bg-panel-darkest".into(), c.clone());
        vars.insert(
            "--color-bg-panel-darkest-hover".into(),
            format!("color-mix(in srgb, {c} 85%, black)"),
        );
        vars.insert(
            "--color-bg-panel-footer".into(),
            format!("color-mix(in srgb, {c} 92%, white)"),
        );
    }

    if let Some(ref c) = bg_channels {
        vars.insert("--color-bg-panel-dark".into(), c.clone());
        vars.insert("--color-char-counter-bg".into(), with_alpha(c, 0.95));
    }

    if let Some(ref c) = bg_chat {
        vars.insert("--color-bg-panel".into(), c.clone());
        vars.insert(
            "--color-bg-panel-hover".into(),
            format!("color-mix(in srgb, {c} 88%, white)"),
        );
        vars.insert(
            "--color-bg-panel-active".into(),
            format!("color-mix(in srgb, {c} 78%, white)"),
        );
        if let Some(ref ch) = bg_channels {
            vars.insert(
                "--gradient-server-banner".into(),
                format!("linear-gradient(135deg, {ch}, {c})"),
            );
        }
    }

    if let Some(ref c) = bg_input {
        vars.insert("--color-bg-input".into(), c.clone());
        vars.insert("--color-bg-input-panel".into(), c.clone());
        vars.insert(
            "--color-bg-input-focus".into(),
            format!("color-mix(in srgb, {c} 88%, white)"),
        );
        vars.insert(
            "--color-bg-input-panel-focus".into(),
            format!("color-mix(in srgb, {c} 90%, white)"),
        );
    }

    if let Some(ref c) = bg_modal {
        vars.insert("--color-bg-container".into(), c.clone());
        vars.insert(
            "--color-bg-container-dark".into(),
            format!("color-mix(in srgb, {c} 82%, black)"),
        );
        vars.insert("--color-bg-glass".into(), with_alpha(c, 0.95));
        vars.insert(
            "--color-bg-social".into(),
            format!("color-mix(in srgb, {c} 88%, white)"),
        );
    }

    if let Some(ref c) = bg_message {
        vars.insert("--color-bg-message".into(), c.clone());
    } else if let Some(ref c) = bg_sidebar {
        vars.insert("--color-bg-message".into(), with_alpha(c, 0.55));
    }

    if let Some(ref c) = border {
        vars.insert("--color-border".into(), c.clone());
        vars.insert("--color-border-header".into(), c.clone());
        vars.insert("--color-icon-slate-arc".into(), c.clone());
        vars.insert("--color-border-subtle".into(), with_alpha(c, 0.35));
        vars.insert("--color-border-container".into(), with_alpha(c, 0.55));
        vars.insert("--color-border-bubble".into(), with_alpha(c, 0.45));
        vars.insert("--color-border-input-panel".into(), with_alpha(c, 0.42));
        vars.insert("--color-border-slate-arc".into(), with_alpha(c, 0.45));
        vars.insert("--color-editor-counter-bg".into(), with_alpha(c, 0.35));
    }

    if let Some(ref c) = primary {
        let hover = format!("color-mix(in srgb, {c} 85%, black)");
        let light = format!("color-mix(in srgb, {c} 76%, white)");
        let lighter = format!("color-mix(in srgb, {c} 58%, white)");
        let dark = format!("color-mix(in srgb, {c} 78%, black)");

        vars.insert("--color-primary".into(), c.clone());
        vars.insert("--color-primary-hover".into(), hover);
        vars.insert("--color-primary-light".into(), light);
        vars.insert("--color-primary-lighter".into(), lighter);
        vars.insert("--color-primary-dark".into(), dark);
        vars.insert("--color-icon-violet".into(), c.clone());
        vars.insert("--color-icon-blue".into(), c.clone());

        for (suffix, alpha) in [
            ("14", 0.14),
            ("15", 0.15),
            ("16", 0.16),
            ("20", 0.20),
            ("24", 0.24),
            ("25", 0.25),
            ("28", 0.28),
            ("30", 0.30),
            ("32", 0.32),
            ("38", 0.38),
            ("40", 0.40),
            ("45", 0.45),
            ("50", 0.50),
        ] {
            vars.insert(
                format!("--color-primary-alpha-{suffix}"),
                with_alpha(c, alpha),
            );
        }

        vars.insert(
            "--shadow-submit-hover".into(),
            format!("0 5px 20px {}", with_alpha(c, 0.4)),
        );
        vars.insert("--color-quick-settings-border".into(), with_alpha(c, 0.5));
        vars.insert(
            "--shadow-quick-settings".into(),
            format!("0 6px 20px {}", with_alpha(c, 0.15)),
        );
        vars.insert("--color-quick-settings-text".into(), c.clone());
    }

    if let Some(ref pe) = primary_end {
        let p_start = primary.as_ref().unwrap_or(pe);
        vars.insert("--color-primary-end".into(), pe.clone());
        vars.insert(
            "--color-primary-end-light".into(),
            format!("color-mix(in srgb, {pe} 75%, white)"),
        );
        vars.insert("--color-icon-magenta".into(), pe.clone());
        vars.insert("--color-primary-end-alpha-15".into(), with_alpha(pe, 0.15));
        vars.insert("--color-primary-end-alpha-25".into(), with_alpha(pe, 0.25));
        vars.insert("--color-primary-end-alpha-30".into(), with_alpha(pe, 0.30));

        let grad = format!("linear-gradient(135deg, {p_start}, {pe})");
        vars.insert("--gradient-home-btn".into(), grad.clone());
        vars.insert("--gradient-home-btn-ring".into(), grad.clone());
        vars.insert("--gradient-server-icon".into(), grad.clone());
        vars.insert("--gradient-message-avatar".into(), grad.clone());
        vars.insert("--gradient-send-btn".into(), grad.clone());
        vars.insert("--gradient-group-icon".into(), grad.clone());
        vars.insert(
            "--gradient-banner-default".into(),
            format!(
                "linear-gradient(135deg, {}, {})",
                with_alpha(p_start, 0.6),
                with_alpha(pe, 0.6)
            ),
        );
        vars.insert(
            "--shadow-send-btn".into(),
            format!("0 2px 10px {}", with_alpha(p_start, 0.4)),
        );
        vars.insert(
            "--shadow-send-btn-hover".into(),
            format!("0 4px 16px {}", with_alpha(pe, 0.5)),
        );
        vars.insert("--color-quick-dms-border".into(), with_alpha(pe, 0.5));
        vars.insert(
            "--shadow-quick-dms".into(),
            format!("0 6px 20px {}", with_alpha(pe, 0.15)),
        );
        vars.insert("--color-quick-dms-text".into(), pe.clone());
        vars.insert("--gradient-voice-btn".into(), grad.clone());
        vars.insert(
            "--shadow-voice-btn".into(),
            format!("0 4px 16px {}", with_alpha(p_start, 0.25)),
        );
        vars.insert(
            "--shadow-voice-btn-hover".into(),
            format!("0 6px 20px {}", with_alpha(pe, 0.35)),
        );
        vars.insert("--gradient-voice-radar".into(), grad.clone());
        vars.insert(
            "--shadow-voice-radar".into(),
            format!("0 4px 16px {}", with_alpha(p_start, 0.25)),
        );
        vars.insert("--color-voice-ring".into(), with_alpha(p_start, 0.25));
    }

    if let Some(ref acc) = accent {
        let acc_bright = format!("color-mix(in srgb, {acc} 72%, white)");
        vars.insert("--color-icon-cyan".into(), acc.clone());
        vars.insert("--color-icon-cyan-bright".into(), acc_bright.clone());
        vars.insert("--color-primary-blue".into(), acc.clone());
        vars.insert("--color-channel-hash".into(), acc.clone());
        vars.insert("--color-channel-hash-hover".into(), acc_bright.clone());
        vars.insert("--color-channel-hash-active".into(), acc.clone());
        vars.insert(
            "--shadow-channel-active".into(),
            format!("inset 2px 0 0 {acc}"),
        );
        vars.insert(
            "--shadow-channel-hash-glow".into(),
            format!("0 0 10px {}", with_alpha(acc, 0.35)),
        );
        vars.insert(
            "--gradient-channel-active".into(),
            format!(
                "linear-gradient(90deg, {} 0%, {} 100%)",
                with_alpha(acc, 0.2),
                with_alpha(acc, 0.08)
            ),
        );
        vars.insert("--gradient-pill-hover".into(), acc.clone());
        vars.insert(
            "--shadow-server-icon-hover".into(),
            format!("0 0 0 2.5px {acc}, 0 0 14px {}", with_alpha(acc, 0.4)),
        );
        vars.insert(
            "--shadow-home-btn-hover".into(),
            format!(
                "0 0 0 3px {}, 0 0 16px {}",
                with_alpha(acc, 0.6),
                with_alpha(acc, 0.4)
            ),
        );
        vars.insert("--color-border-bubble-hover".into(), with_alpha(acc, 0.32));
        vars.insert("--color-border-input-focus".into(), with_alpha(acc, 0.55));
        vars.insert(
            "--shadow-input-focus".into(),
            format!("0 0 0 3px {}", with_alpha(acc, 0.2)),
        );
        vars.insert("--color-server-new-border".into(), with_alpha(acc, 0.45));
        vars.insert("--color-server-new-text".into(), acc_bright.clone());
        vars.insert("--color-editor-counter-text".into(), acc_bright.clone());
        vars.insert("--color-quick-join-border".into(), with_alpha(acc, 0.5));
        vars.insert(
            "--shadow-quick-join".into(),
            format!("0 6px 20px {}", with_alpha(acc, 0.15)),
        );
        vars.insert("--color-quick-join-text".into(), acc_bright);
    }

    if let Some(ref warm) = accent_warm {
        let spark = format!("color-mix(in srgb, {warm} 60%, white)");
        vars.insert("--color-icon-amber".into(), warm.clone());
        vars.insert("--color-icon-orange".into(), warm.clone());
        vars.insert("--color-icon-spark".into(), spark.clone());
        vars.insert("--gradient-pill-active".into(), warm.clone());
        vars.insert(
            "--filter-pill-active".into(),
            format!("drop-shadow(0 0 6px {})", with_alpha(warm, 0.65)),
        );
        vars.insert(
            "--shadow-server-icon-active".into(),
            format!("0 0 0 2.5px {warm}, 0 0 16px {}", with_alpha(warm, 0.4)),
        );
        vars.insert(
            "--shadow-home-btn-active".into(),
            format!("0 0 0 2.5px {warm}, 0 0 18px {}", with_alpha(warm, 0.45)),
        );
        vars.insert("--color-home-badge-border".into(), warm.clone());
        vars.insert("--color-home-badge-border-hover".into(), spark.clone());
        vars.insert("--color-channel-add-hover-text".into(), warm.clone());
        vars.insert(
            "--color-channel-add-hover-bg".into(),
            with_alpha(warm, 0.16),
        );
        vars.insert("--color-char-counter-border".into(), with_alpha(warm, 0.45));
        vars.insert("--color-char-counter-text".into(), spark.clone());
        vars.insert("--color-read-more-border".into(), with_alpha(warm, 0.4));
        vars.insert("--color-read-more-text".into(), warm.clone());
        vars.insert(
            "--color-read-more-hover-border".into(),
            with_alpha(warm, 0.65),
        );
        vars.insert("--color-read-more-hover-text".into(), spark.clone());
        vars.insert("--gradient-read-more".into(), with_alpha(warm, 0.16));
        vars.insert("--gradient-read-more-hover".into(), with_alpha(warm, 0.28));
        vars.insert("--color-editor-badge-border".into(), with_alpha(warm, 0.4));
        vars.insert("--color-editor-badge-text".into(), warm.clone());
        vars.insert("--gradient-editor-badge".into(), with_alpha(warm, 0.18));
        vars.insert("--color-server-new-hover-border".into(), warm.clone());
        vars.insert("--color-server-new-hover-text".into(), spark);
        vars.insert("--color-server-new-hover-bg".into(), with_alpha(warm, 0.18));
        vars.insert(
            "--shadow-server-new-hover".into(),
            format!(
                "0 0 0 2px {}, 0 0 14px {}",
                with_alpha(warm, 0.55),
                with_alpha(warm, 0.35)
            ),
        );
    }

    if let (Some(acc), Some(pr)) = (&accent, &primary) {
        let warm = accent_warm.as_ref().unwrap_or(pr);
        let sep_1 = with_alpha(acc, 0.75);
        let sep_2 = with_alpha(pr, 0.85);
        let sep_3 = with_alpha(warm, 0.75);
        vars.insert("--color-server-separator-stop-1".into(), sep_1.clone());
        vars.insert("--color-server-separator-stop-2".into(), sep_2.clone());
        vars.insert("--color-server-separator-stop-3".into(), sep_3.clone());
        vars.insert(
            "--gradient-server-separator".into(),
            format!("linear-gradient(90deg, {sep_1}, {sep_2}, {sep_3})"),
        );
        vars.insert("--color-server-flap-stop-1".into(), darken_color(acc, 0.45));
        vars.insert("--color-server-flap-stop-2".into(), darken_color(pr, 0.45));
        vars.insert(
            "--color-server-flap-stop-3".into(),
            darken_color(warm, 0.45),
        );
        vars.insert("--color-server-flap-stroke".into(), darken_color(pr, 0.65));
        vars.insert("--color-server-peel-glow".into(), with_alpha(warm, 0.8));
        vars.insert(
            "--color-server-peel-glow-ambient".into(),
            with_alpha(pr, 0.45),
        );
        vars.insert(
            "--gradient-footer-top-line".into(),
            format!(
                "linear-gradient(90deg, transparent 0%, {} 25%, {} 55%, {} 85%, transparent 100%)",
                with_alpha(acc, 0.5),
                with_alpha(pr, 0.55),
                with_alpha(warm, 0.5)
            ),
        );
        vars.insert(
            "--gradient-date-divider".into(),
            format!(
                "linear-gradient(90deg, transparent 0%, {} 22%, {} 50%, {} 78%, transparent 100%)",
                with_alpha(acc, 0.35),
                with_alpha(pr, 0.45),
                with_alpha(warm, 0.35)
            ),
        );
        vars.insert(
            "--gradient-hero-banner".into(),
            format!(
                "linear-gradient(135deg, {} 0%, {} 50%, {} 100%)",
                with_alpha(acc, 0.2),
                with_alpha(pr, 0.22),
                with_alpha(warm, 0.18)
            ),
        );
        vars.insert("--color-hero-banner-border".into(), with_alpha(acc, 0.3));
    }

    if let Some(ref pr) = primary {
        let acc = accent.as_ref().unwrap_or(pr);
        let pr_end = primary_end.as_ref().unwrap_or(pr);
        let warm = accent_warm.as_ref().unwrap_or(acc);
        vars.insert(
            "--gradient-aurora-orb-1".into(),
            format!("radial-gradient(circle, {acc} 0%, {pr} 40%, transparent 70%)"),
        );
        vars.insert(
            "--gradient-aurora-orb-2".into(),
            format!("radial-gradient(circle, {pr} 0%, {pr_end} 45%, transparent 70%)"),
        );
        vars.insert(
            "--gradient-aurora-orb-3".into(),
            format!("radial-gradient(circle, {warm} 0%, transparent 65%)"),
        );
    }

    if let Some(ref c) = text_heading {
        vars.insert("--color-text-heading".into(), c.clone());
        vars.insert("--color-text-white".into(), c.clone());
    }

    if let Some(ref c) = text_chat {
        vars.insert("--color-text-chat".into(), c.clone());
        vars.insert("--color-text-member".into(), c.clone());
        vars.insert("--color-text-social".into(), c.clone());
    }

    if let Some(ref c) = text_muted {
        vars.insert("--color-text-muted".into(), c.clone());
        vars.insert("--color-text-label".into(), c.clone());
        vars.insert("--color-text-channel".into(), c.clone());
        vars.insert("--color-text-tab".into(), c.clone());
        vars.insert(
            "--color-text-dim".into(),
            format!("color-mix(in srgb, {c} 75%, black)"),
        );
    }

    if let Some(ref c) = text_author {
        vars.insert("--color-text-author".into(), c.clone());
        vars.insert("--color-text-date-badge".into(), c.clone());
        vars.insert(
            "--color-text-author-hover".into(),
            format!("color-mix(in srgb, {c} 75%, white)"),
        );
    }

    if let Some(ref c) = online {
        vars.insert("--color-status-online".into(), c.clone());
        vars.insert("--color-success".into(), c.clone());
        vars.insert("--color-success-alpha-20".into(), with_alpha(c, 0.2));
    }

    if let Some(ref c) = idle {
        vars.insert("--color-status-idle".into(), c.clone());
    }

    if let Some(ref c) = danger {
        vars.insert("--color-danger".into(), c.clone());
        vars.insert("--color-error".into(), c.clone());
        vars.insert("--color-status-dnd".into(), c.clone());
        vars.insert("--color-icon-coral".into(), c.clone());
        vars.insert(
            "--color-danger-hover".into(),
            format!("color-mix(in srgb, {c} 85%, black)"),
        );
        vars.insert(
            "--color-danger-light".into(),
            format!("color-mix(in srgb, {c} 72%, white)"),
        );
        vars.insert("--color-error-alpha-15".into(), with_alpha(c, 0.15));
        vars.insert("--color-danger-alpha-08".into(), with_alpha(c, 0.08));
        vars.insert("--color-danger-alpha-15".into(), with_alpha(c, 0.15));
        vars.insert("--color-danger-alpha-18".into(), with_alpha(c, 0.18));
        vars.insert("--color-danger-alpha-40".into(), with_alpha(c, 0.40));
        vars.insert("--color-danger-alpha-75".into(), with_alpha(c, 0.75));
        vars.insert("--gradient-unread-badge".into(), c.clone());
        vars.insert(
            "--shadow-unread-badge".into(),
            format!("0 2px 8px {}", with_alpha(c, 0.45)),
        );
        vars.insert("--color-quick-create-border".into(), with_alpha(c, 0.5));
        vars.insert(
            "--shadow-quick-create".into(),
            format!("0 6px 20px {}", with_alpha(c, 0.15)),
        );
        vars.insert("--color-quick-create-text".into(), c.clone());
    }

    for (raw_key, raw_val) in custom_variables {
        let key = raw_key.trim();
        let val = raw_val.trim();
        if is_allowed_css_var(key) && is_safe_css_value(val) {
            vars.insert(key.to_string(), val.to_string());
        }
    }

    if !vars.is_empty() {
        vars.entry("--color-text-on-accent".into())
            .or_insert_with(|| "#ffffff".into());
    }

    vars
}
