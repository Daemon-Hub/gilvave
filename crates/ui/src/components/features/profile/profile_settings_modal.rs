use sycamore::{futures::spawn_local_scoped, prelude::*};

use wasm_bindgen::JsCast;
use crate::components::ui::icons::{
    ArrowLeftIcon, CatalogIcon, CheckIcon, CloseSmallIcon, CodeIcon,
    DownloadIcon, EditIcon, FileTextIcon, LockIcon, LogOutIcon, PaletteIcon, TrashIcon,
    UploadIcon, UserIcon,
};
use crate::components::common::{
    ActiveScreen, AppTheme, CustomTheme, ScreenWrapper, UiModalContext, UserProfileContext,
    classes, download_json_file,
};

#[derive(Clone, Copy, PartialEq)]
enum SettingsTab {
    Profile,
    Appearance,
    Security,
}

#[component]
pub fn ProfileSettingsModal() -> View {
    let modal_context = use_context::<UiModalContext>();
    let user_profile = use_context::<UserProfileContext>();
    let screen_wrapper = use_context::<ScreenWrapper>();

    let active_tab = create_signal(SettingsTab::Profile);

    let name_input = create_signal(String::new());
    let avatar_input = create_signal(String::new());
    let banner_input = create_signal(String::new());
    let bio_input = create_signal(String::new());

    let old_password = create_signal(String::new());
    let new_password = create_signal(String::new());
    let confirm_password = create_signal(String::new());
    let password_msg = create_signal(String::new());

    let theme_status_msg = create_signal(String::new());
    let theme_status_is_error = create_signal(false);

    // Sync from user_profile when modal opens
    create_effect(move || {
        if modal_context.is_profile_settings_open.get() {
            name_input.set(user_profile.username.get_clone());
            avatar_input.set(user_profile.avatar.get_clone());
            banner_input.set(user_profile.banner.get_clone());
            bio_input.set(user_profile.bio.get_clone());
            password_msg.set(String::new());
            theme_status_msg.set(String::new());
        }
    });

    let close = move |_| {
        modal_context.is_profile_settings_open.set(false);
    };

    let handle_save = move |_| {
        if name_input.with(|n| !n.trim().is_empty()) {
            user_profile.username.set(name_input.get_clone());
        }
        user_profile.avatar.set(avatar_input.get_clone());
        user_profile.banner.set(banner_input.get_clone());
        user_profile.bio.set(bio_input.get_clone());
        modal_context.is_profile_settings_open.set(false);
    };

    let handle_change_password = move |_| {
        if new_password.with(|p| p.len() < 6) {
            password_msg.set("Пароль должен быть не менее 6 символов".to_string());
            return;
        }
        if new_password.with(|np| confirm_password.with(|cp| np != cp)) {
            password_msg.set("Пароли не совпадают".to_string());
            return;
        }
        password_msg.set("Пароль успешно обновлён!".to_string());
        old_password.set(String::new());
        new_password.set(String::new());
        confirm_password.set(String::new());
    };

    let handle_logout = move |_| {
        modal_context.is_profile_settings_open.set(false);
        screen_wrapper.set(ActiveScreen::Login);
    };

    view! {
        div(
            class="server-modal-overlay large",
            on:click=close,
        ) {
            div(
                class="server-modal profile-settings-modal",
                on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
            ) {
                div(class="profile-modal-top-bar") {
                    span(class="profile-modal-title") { "Настройки профиля" }
                    button(class="modal-close-icon-btn", on:click=close, title="Закрыть") { CloseSmallIcon() }
                }
                div(class="profile-modal-layout") {
                    div(class="profile-sidebar-tabs") {
                        h4 { "НАСТРОЙКИ" }
                        div(
                            class=classes(vec![
                                "profile-tab-item".into(),
                                ("active", { active_tab.get() == SettingsTab::Profile }.into()).into(),
                            ]),
                            on:click=move |_| active_tab.set(SettingsTab::Profile),
                        ) {
                            UserIcon()
                            span { "Профиль" }
                        }
                        div(
                            class=classes(vec![
                                "profile-tab-item".into(),
                                ("active", { active_tab.get() == SettingsTab::Appearance }.into()).into(),
                            ]),
                            on:click=move |_| active_tab.set(SettingsTab::Appearance),
                        ) {
                            PaletteIcon()
                            span { "Внешний вид" }
                        }
                        div(
                            class=classes(vec![
                                "profile-tab-item".into(),
                                ("active", { active_tab.get() == SettingsTab::Security }.into()).into(),
                            ]),
                            on:click=move |_| active_tab.set(SettingsTab::Security),
                        ) {
                            LockIcon()
                            span { "Безопасность" }
                        }

                        div(class="profile-tab-divider")

                        div(class="profile-tab-item logout", on:click=handle_logout) {
                            LogOutIcon()
                            span { "Выйти из аккаунта" }
                        }
                    }

                    div(class="profile-content-body") {
                        (if active_tab.get() == SettingsTab::Profile {
                            view! {
                                div(class="tab-pane") {
                                    h3(class="tab-title") { "Мой профиль" }

                                    // Live preview card
                                    div(class="profile-card-preview") {
                                        div(class="profile-preview-banner") {
                                            (if banner_input.with(|b| !b.is_empty()) {
                                                let b_url = banner_input.get_clone();
                                                view! { img(src=b_url, alt="") }
                                            } else {
                                                view! { div(class="default-banner") }
                                            })
                                        }

                                        div(class="profile-preview-header") {
                                            div(class="profile-preview-avatar") {
                                                (if avatar_input.with(|a| !a.is_empty()) {
                                                    let a_url = avatar_input.get_clone();
                                                    view! { img(src=a_url, alt="") }
                                                } else {
                                                    let initial = name_input.with(|n| n.chars().next().unwrap_or('?').to_uppercase().to_string());
                                                    view! { span { (initial) } }
                                                })
                                                div(class="status-dot online")
                                            }
                                            div(class="profile-preview-names") {
                                                span(class="preview-name") { (name_input.get_clone()) }
                                                span(class="preview-tag") { "@" (name_input.with(|n| n.to_lowercase())) }
                                            }
                                        }

                                        (if bio_input.with(|b| !b.is_empty()) {
                                            let bio = bio_input.get_clone();
                                            view! {
                                                div(class="profile-preview-bio") {
                                                    span(class="bio-label") { "О СЕБЕ" }
                                                    p { (bio) }
                                                }
                                            }
                                        } else {
                                            view! {}
                                        })
                                    }

                                    div(class="input-group") {
                                        label { "ОТОБРАЖАЕМОЕ ИМЯ" }
                                        input(r#type="text", placeholder="Ваше имя", bind:value=name_input)
                                    }

                                    div(class="input-group") {
                                        label { "URL АВАТАРКИ" }
                                        input(r#type="text", placeholder="https://example.com/avatar.png", bind:value=avatar_input)
                                    }

                                    div(class="input-group") {
                                        label { "URL БАННЕРА" }
                                        input(r#type="text", placeholder="https://example.com/banner.png", bind:value=banner_input)
                                    }

                                    div(class="input-group") {
                                        label { "О СЕБЕ" }
                                        textarea(rows="2", placeholder="Расскажите немного о себе...", bind:value=bio_input)
                                    }

                                    div(class="create-form-actions modal-footer-buttons") {
                                        button(class="back-btn", on:click=close) { "Отмена" }
                                        button(class="submit-btn create-submit", on:click=handle_save) { "Сохранить изменения" }
                                    }
                                }
                            }
                        } else if active_tab.get() == SettingsTab::Appearance {
                            let current_theme = modal_context.app_theme;
                            let custom_themes_sig = modal_context.custom_themes;
                            let is_windowed_sig = modal_context.is_windowed_mode;

                            let install_custom_theme_from_json = move |raw_json: String| {
                                match CustomTheme::from_json_str(&raw_json) {
                                    Ok(new_theme) => {
                                        let new_id = new_theme.id.clone();
                                        let new_name = new_theme.name.clone();
                                        let mut list = custom_themes_sig.get_clone();
                                        if let Some(existing) = list.iter_mut().find(|t| t.id == new_id) {
                                            *existing = new_theme;
                                        } else {
                                            list.push(new_theme);
                                        }
                                        CustomTheme::save_all(&list);
                                        custom_themes_sig.set(list);
                                        current_theme.set(AppTheme::Custom(new_id));
                                        if modal_context.is_profile_settings_open.get() {
                                            theme_status_is_error.set(false);
                                            theme_status_msg.set(format!(
                                                "✓ Тема «{new_name}» успешно установлена и активирована!"
                                            ));
                                        }
                                    }
                                    Err(err_msg) => {
                                        if modal_context.is_profile_settings_open.get() {
                                            theme_status_is_error.set(true);
                                            theme_status_msg.set(err_msg);
                                        }
                                    }
                                }
                            };

                            let on_select_change = move |e: web_sys::Event| {
                                if let Some(target) = e.target() {
                                    if let Ok(select) = target.dyn_into::<web_sys::HtmlSelectElement>() {
                                        current_theme.set(AppTheme::from_id(&select.value()));
                                    }
                                }
                            };

                            let on_file_upload = move |e: web_sys::Event| {
                                if let Some(target) = e.target() {
                                    if let Ok(input) = target.dyn_into::<web_sys::HtmlInputElement>() {
                                        if let Some(files) = input.files() {
                                            if let Some(file) = files.get(0) {
                                                let promise = file.text();
                                                wasm_bindgen_futures::spawn_local(async move {
                                                    if let Ok(js_val) =
                                                        wasm_bindgen_futures::JsFuture::from(promise).await
                                                    {
                                                        if let Some(content) = js_val.as_string() {
                                                            install_custom_theme_from_json(content);
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                        input.set_value("");
                                    }
                                }
                            };

                            let on_download_template = move |_| {
                                let tpl = CustomTheme::template_json();
                                download_json_file("my-theme.gilvave-theme.json", &tpl);
                            };

                            view! {
                                div(class="tab-pane theme-settings-pane") {
                                    h3(class="tab-title") { "Внешний вид" }

                                    div(class="vscode-setting-item") {
                                        div(class="vscode-setting-header") {
                                            span(class="vscode-setting-category") { "Workbench › Appearance: " }
                                            span(class="vscode-setting-name") { "Color Theme" }
                                        }
                                        p(class="vscode-setting-desc") {
                                            "Выберите встроенную или установленную пользовательскую тему интерфейса Gilvave. Как в VS Code, вы можете скачивать темы из каталога или загружать JSON-файлы."
                                        }
                                        div(class="vscode-select-wrapper") {
                                            select(
                                                class="vscode-theme-select",
                                                on:change=on_select_change,
                                            ) {
                                                option(
                                                    value="advanced",
                                                    selected=move || current_theme.get_clone() == AppTheme::Advanced,
                                                ) {
                                                    "Gilvave Advanced (Cosmic Icon Palette)"
                                                }
                                                option(
                                                    value="standard",
                                                    selected=move || current_theme.get_clone() == AppTheme::Standard,
                                                ) {
                                                    "Gilvave Standard (Classic Dark)"
                                                }
                                                option(
                                                    value="light",
                                                    selected=move || current_theme.get_clone() == AppTheme::Light,
                                                ) {
                                                    "Gilvave Light (Clean Modern Light)"
                                                }
                                                (move || {
                                                    let list = custom_themes_sig.get_clone();
                                                    let active = current_theme.get_clone();
                                                    let opts: Vec<View> = list
                                                        .into_iter()
                                                        .map(|t| {
                                                            let val = t.id.clone();
                                                            let is_sel = active == AppTheme::Custom(t.id.clone());
                                                            let label = format!("{} (by @{})", t.name, t.author);
                                                            view! {
                                                                option(value=val, selected=is_sel) { (label) }
                                                            }
                                                        })
                                                        .collect();
                                                    opts
                                                })
                                            }
                                        }
                                    }

                                    // Windowed Mode vs Frameless Fullbleed Toggle
                                    div(class="vscode-setting-item") {
                                        div(class="vscode-setting-header") {
                                            span(class="vscode-setting-category") { "Workbench › Appearance: " }
                                            span(class="vscode-setting-name") { "Windowed Mode (Floating Frame)" }
                                        }
                                        p(class="vscode-setting-desc") {
                                            "Отображать отступы и фоновое свечение вокруг рабочей панели. Отключите, чтобы рабочая область заполнила весь экран без зазоров по краям."
                                        }
                                        div(
                                            class="vscode-toggle-row",
                                            on:click=move |_| {
                                                if !crate::components::common::is_mobile_device() {
                                                    is_windowed_sig.set(!is_windowed_sig.get());
                                                }
                                            },
                                        ) {
                                            div(
                                                class=move || {
                                                    if crate::components::common::is_mobile_device() {
                                                        "vscode-switch disabled"
                                                    } else if is_windowed_sig.get() {
                                                        "vscode-switch active"
                                                    } else {
                                                        "vscode-switch"
                                                    }
                                                },
                                            ) {
                                                div(class="vscode-switch-thumb")
                                            }
                                            span(class="vscode-switch-label") {
                                                (move || {
                                                    if crate::components::common::is_mobile_device() {
                                                        "Оконный режим автоматически отключён на мобильных устройствах"
                                                    } else if is_windowed_sig.get() {
                                                        "Оконный режим включён (с отступами и фоном по краям)"
                                                    } else {
                                                        "Оконный режим отключён (панель заполняет всё пространство без зазоров)"
                                                    }
                                                })
                                            }
                                        }
                                    }

                                    // Custom Themes Toolbar (VS Code Extensions / JSON style)
                                    div(class="custom-themes-toolbar") {
                                        button(
                                            class="toolbar-btn primary",
                                            on:click=move |_| {
                                                modal_context.is_theme_catalog_open.set(true);
                                            },
                                        ) {
                                            CatalogIcon()
                                            span { "Каталог тем" }
                                        }
                                        label(
                                            r#for="custom-theme-file-input",
                                            class="toolbar-btn",
                                        ) {
                                            DownloadIcon()
                                            span { "Загрузить тему (.json)" }
                                        }
                                        input(
                                            id="custom-theme-file-input",
                                            r#type="file",
                                            accept=".json",
                                            class="hidden-file-input",
                                            on:change=on_file_upload,
                                        )
                                        button(
                                            class="toolbar-btn",
                                            on:click=move |_| {
                                                modal_context.theme_json_editor_content.set(CustomTheme::template_json());
                                                modal_context.is_theme_json_editor_open.set(true);
                                            },
                                        ) {
                                            CodeIcon()
                                            span { "Открыть JSON-редактор" }
                                        }
                                        button(
                                            class="toolbar-btn",
                                            on:click=on_download_template,
                                        ) {
                                            FileTextIcon()
                                            span { "Скачать шаблон (.json)" }
                                        }
                                    }

                                    // Status / validation banner
                                    (move || {
                                        let msg = theme_status_msg.get_clone();
                                        if msg.is_empty() {
                                            view! {}
                                        } else {
                                            let cls = if theme_status_is_error.get() {
                                                "theme-status-banner error"
                                            } else {
                                                "theme-status-banner"
                                            };
                                            view! {
                                                div(class=cls) { (msg) }
                                            }
                                        }
                                    })

                                    div(class="theme-cards-grid") {
                                        // Advanced Theme Card
                                        div(
                                            class=move || {
                                                if current_theme.get_clone() == AppTheme::Advanced {
                                                    "theme-preview-card advanced active"
                                                } else {
                                                    "theme-preview-card advanced"
                                                }
                                            },
                                            on:click=move |_| current_theme.set(AppTheme::Advanced),
                                        ) {
                                            div(class="theme-mockup advanced-mockup") {
                                                div(class="mockup-sidebar") {
                                                    div(class="mockup-dot home")
                                                    div(class="mockup-dot")
                                                    div(class="mockup-dot plus")
                                                }
                                                div(class="mockup-channels") {
                                                    div(class="mockup-line active")
                                                    div(class="mockup-line")
                                                    div(class="mockup-line")
                                                }
                                                div(class="mockup-chat") {
                                                    div(class="mockup-msg") {
                                                        div(class="mockup-avatar")
                                                        div(class="mockup-bubble")
                                                    }
                                                    div(class="mockup-divider")
                                                    div(class="mockup-input-row") {
                                                        div(class="mockup-input")
                                                        div(class="mockup-send")
                                                    }
                                                }
                                            }
                                            div(class="theme-card-info") {
                                                div(class="theme-card-title-row") {
                                                    span(class="theme-card-name") { "Advanced" }
                                                    span(class="theme-card-badge advanced-badge") { "Cosmic Icon" }
                                                    (move || if current_theme.get_clone() == AppTheme::Advanced {
                                                        view! { span(class="theme-active-check") { CheckIcon() span { "Активна" } } }
                                                    } else {
                                                        view! {}
                                                    })
                                                }
                                                p(class="theme-card-desc") {
                                                    "Глубокий космический индиго, неоновое кольцо (циан, индиго, маджента) и тёплая импульсная волна (коралл, оранжевый, янтарь) с иконки Gilvave."
                                                }
                                                div(class="theme-swatches") {
                                                    span(class="swatch adv-1")
                                                    span(class="swatch adv-2")
                                                    span(class="swatch adv-3")
                                                    span(class="swatch adv-4")
                                                    span(class="swatch adv-5")
                                                    span(class="swatch adv-6")
                                                    span(class="swatch adv-7")
                                                }
                                            }
                                        }

                                        // Standard Theme Card
                                        div(
                                            class=move || {
                                                if current_theme.get_clone() == AppTheme::Standard {
                                                    "theme-preview-card standard active"
                                                } else {
                                                    "theme-preview-card standard"
                                                }
                                            },
                                            on:click=move |_| current_theme.set(AppTheme::Standard),
                                        ) {
                                            div(class="theme-mockup standard-mockup") {
                                                div(class="mockup-sidebar") {
                                                    div(class="mockup-dot home")
                                                    div(class="mockup-dot")
                                                    div(class="mockup-dot plus")
                                                }
                                                div(class="mockup-channels") {
                                                    div(class="mockup-line active")
                                                    div(class="mockup-line")
                                                    div(class="mockup-line")
                                                }
                                                div(class="mockup-chat") {
                                                    div(class="mockup-msg") {
                                                        div(class="mockup-avatar")
                                                        div(class="mockup-bubble")
                                                    }
                                                    div(class="mockup-divider")
                                                    div(class="mockup-input-row") {
                                                        div(class="mockup-input")
                                                        div(class="mockup-send")
                                                    }
                                                }
                                            }
                                            div(class="theme-card-info") {
                                                div(class="theme-card-title-row") {
                                                    span(class="theme-card-name") { "Standard" }
                                                    span(class="theme-card-badge standard-badge") { "Classic Dark" }
                                                    (move || if current_theme.get_clone() == AppTheme::Standard {
                                                        view! { span(class="theme-active-check") { CheckIcon() span { "Активна" } } }
                                                    } else {
                                                        view! {}
                                                    })
                                                }
                                                p(class="theme-card-desc") {
                                                    "Классическая тёмно-серая палитра панелей (#36393f / #2f3136 / #202225) со сдержанными лавандово-розовыми акцентами."
                                                }
                                                div(class="theme-swatches") {
                                                    span(class="swatch std-1")
                                                    span(class="swatch std-2")
                                                    span(class="swatch std-3")
                                                    span(class="swatch std-4")
                                                    span(class="swatch std-5")
                                                    span(class="swatch std-6")
                                                    span(class="swatch std-7")
                                                }
                                            }
                                        }

                                        // Light Theme Card
                                        div(
                                            class=move || {
                                                if current_theme.get_clone() == AppTheme::Light {
                                                    "theme-preview-card light active"
                                                } else {
                                                    "theme-preview-card light"
                                                }
                                            },
                                            on:click=move |_| current_theme.set(AppTheme::Light),
                                        ) {
                                            div(class="theme-mockup light-mockup") {
                                                div(class="mockup-sidebar") {
                                                    div(class="mockup-dot home")
                                                    div(class="mockup-dot")
                                                    div(class="mockup-dot plus")
                                                }
                                                div(class="mockup-channels") {
                                                    div(class="mockup-line active")
                                                    div(class="mockup-line")
                                                    div(class="mockup-line")
                                                }
                                                div(class="mockup-chat") {
                                                    div(class="mockup-msg") {
                                                        div(class="mockup-avatar")
                                                        div(class="mockup-bubble")
                                                    }
                                                    div(class="mockup-divider")
                                                    div(class="mockup-input-row") {
                                                        div(class="mockup-input")
                                                        div(class="mockup-send")
                                                    }
                                                }
                                            }
                                            div(class="theme-card-info") {
                                                div(class="theme-card-title-row") {
                                                    span(class="theme-card-name") { "Light" }
                                                    span(class="theme-card-badge light-badge") { "Clean Light" }
                                                    (move || if current_theme.get_clone() == AppTheme::Light {
                                                        view! { span(class="theme-active-check") { CheckIcon() span { "Активна" } } }
                                                    } else {
                                                        view! {}
                                                    })
                                                }
                                                p(class="theme-card-desc") {
                                                    "Светлая, чистая и современная тема с мягким контрастом, сине-фиолетовыми акцентами и отличной читаемостью."
                                                }
                                                div(class="theme-swatches") {
                                                    span(class="swatch light-1")
                                                    span(class="swatch light-2")
                                                    span(class="swatch light-3")
                                                    span(class="swatch light-4")
                                                    span(class="swatch light-5")
                                                    span(class="swatch light-6")
                                                    span(class="swatch light-7")
                                                }
                                            }
                                        }

                                        // User / Community Custom Theme Cards
                                        (move || {
                                            let list = custom_themes_sig.get_clone();
                                            let cards: Vec<View> = list
                                                .into_iter()
                                                .map(|theme| {
                                                    let theme_id = theme.id.clone();
                                                    let theme_id_click = theme.id.clone();
                                                    let theme_id_active = theme.id.clone();
                                                    let theme_id_check = theme.id.clone();
                                                    let theme_id_delete = theme.id.clone();
                                                    let theme_name_delete = theme.name.clone();
                                                    let theme_for_export = theme.clone();
                                                    let theme_for_edit = theme.clone();
                                                    let name = theme.name.clone();
                                                    let author = format!("by @{}", theme.author);
                                                    let desc = if theme.description.trim().is_empty() {
                                                        format!(
                                                            "Пользовательская тема ({}, v{})",
                                                            theme.base, theme.version
                                                        )
                                                    } else {
                                                        theme.description.clone()
                                                    };

                                                    view! {
                                                        div(
                                                            class=move || {
                                                                if current_theme.get_clone()
                                                                    == AppTheme::Custom(theme_id_active.clone())
                                                                {
                                                                    "theme-preview-card custom active"
                                                                } else {
                                                                    "theme-preview-card custom"
                                                                }
                                                            },
                                                            data-custom-theme=theme_id,
                                                            on:click=move |_| {
                                                                current_theme.set(AppTheme::Custom(theme_id_click.clone()));
                                                            },
                                                        ) {
                                                            div(class="theme-mockup custom-mockup") {
                                                                div(class="mockup-sidebar") {
                                                                    div(class="mockup-dot home")
                                                                    div(class="mockup-dot")
                                                                    div(class="mockup-dot plus")
                                                                }
                                                                div(class="mockup-channels") {
                                                                    div(class="mockup-line active")
                                                                    div(class="mockup-line")
                                                                    div(class="mockup-line")
                                                                }
                                                                div(class="mockup-chat") {
                                                                    div(class="mockup-msg") {
                                                                        div(class="mockup-avatar")
                                                                        div(class="mockup-bubble")
                                                                    }
                                                                    div(class="mockup-divider")
                                                                    div(class="mockup-input-row") {
                                                                        div(class="mockup-input")
                                                                        div(class="mockup-send")
                                                                    }
                                                                }
                                                            }
                                                            div(class="theme-card-info") {
                                                                div(class="theme-card-title-row") {
                                                                    span(class="theme-card-name") { (name) }
                                                                    span(class="theme-card-badge custom-badge") { (author) }
                                                                    (move || {
                                                                        if current_theme.get_clone()
                                                                            == AppTheme::Custom(theme_id_check.clone())
                                                                        {
                                                                            view! { span(class="theme-active-check") { CheckIcon() span { "Активна" } } }
                                                                        } else {
                                                                            view! {}
                                                                        }
                                                                    })
                                                                }
                                                                p(class="theme-card-desc") { (desc) }
                                                                div(class="theme-swatches") {
                                                                    span(class="swatch custom-1")
                                                                    span(class="swatch custom-2")
                                                                    span(class="swatch custom-3")
                                                                    span(class="swatch custom-4")
                                                                    span(class="swatch custom-5")
                                                                    span(class="swatch custom-6")
                                                                    span(class="swatch custom-7")
                                                                }
                                                                div(
                                                                    class="theme-card-actions",
                                                                    on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
                                                                ) {
                                                                    button(
                                                                        class="theme-card-btn",
                                                                        on:click=move |_| {
                                                                            modal_context.theme_json_editor_content.set(theme_for_edit.to_pretty_json());
                                                                            modal_context.is_theme_json_editor_open.set(true);
                                                                            theme_status_msg.set(String::new());
                                                                        },
                                                                    ) {
                                                                        EditIcon()
                                                                        span { "JSON" }
                                                                    }
                                                                    button(
                                                                        class="theme-card-btn",
                                                                        on:click=move |_| {
                                                                            let filename = format!(
                                                                                "{}.gilvave-theme.json",
                                                                                theme_for_export.id
                                                                            );
                                                                            let content = theme_for_export.to_pretty_json();
                                                                            download_json_file(&filename, &content);
                                                                        },
                                                                    ) {
                                                                        UploadIcon()
                                                                        span { "Экспорт" }
                                                                    }
                                                                    button(
                                                                        class="theme-card-btn delete",
                                                                        title="Удалить тему",
                                                                        on:click=move |_| {
                                                                            let mut list = custom_themes_sig.get_clone();
                                                                            list.retain(|t| t.id != theme_id_delete);
                                                                            CustomTheme::save_all(&list);
                                                                            custom_themes_sig.set(list);
                                                                            if current_theme.get_clone()
                                                                                == AppTheme::Custom(theme_id_delete.clone())
                                                                            {
                                                                                current_theme.set(AppTheme::Advanced);
                                                                            }
                                                                            theme_status_is_error.set(false);
                                                                            theme_status_msg.set(format!(
                                                                                "Тема «{theme_name_delete}» удалена."
                                                                            ));
                                                                        },
                                                                    ) { TrashIcon() }
                                                                }
                                                            }
                                                        }
                                                    }
                                                })
                                                .collect();
                                            cards
                                        })
                                    }
                                }
                            }
                        } else {
                            view! {
                                div(class="tab-pane") {
                                    h3(class="tab-title") { "Безопасность и вход" }

                                    div(class="input-group") {
                                        label { "ТЕКУЩИЙ ПАРОЛЬ" }
                                        input(r#type="password", placeholder="••••••••", bind:value=old_password)
                                    }

                                    div(class="input-group") {
                                        label { "НОВЫЙ ПАРОЛЬ" }
                                        input(r#type="password", placeholder="Минимум 6 символов", bind:value=new_password)
                                    }

                                    div(class="input-group") {
                                        label { "ПОДТВЕРДИТЕ НОВЫЙ ПАРОЛЬ" }
                                        input(r#type="password", placeholder="••••••••", bind:value=confirm_password)
                                    }

                                    (if password_msg.with(|m| !m.is_empty()) {
                                        let msg = password_msg.get_clone();
                                        view! {
                                            div(class="password-status-msg") { (msg) }
                                        }
                                    } else {
                                        view! {}
                                    })

                                    div(class="create-form-actions modal-footer-buttons") {
                                        button(class="back-btn", on:click=close) { "Отмена" }
                                        button(class="submit-btn", on:click=handle_change_password) { "Обновить пароль" }
                                    }

                                    div(class="security-divider")

                                    div(class="danger-zone") {
                                        h4 { "СЕССИЯ" }
                                        p { "Выход из текущей учётной записи на этом устройстве." }
                                        button(class="danger-btn logout-btn", on:click=handle_logout) {
                                            LogOutIcon()
                                            span { "Выйти из аккаунта" }
                                        }
                                    }
                                }
                            }
                        })
                    }
                }
            }
        }
    }
}

#[component]
pub fn ThemeCatalogModal() -> View {
    let modal_context = use_context::<UiModalContext>();
    let current_theme = modal_context.app_theme;
    let custom_themes_sig = modal_context.custom_themes;

    let search_query = create_signal(String::new());
    let server_themes = create_signal::<Vec<CustomTheme>>(CustomTheme::catalog_themes());

    // Keep preview CSS synced for any themes returned from the server catalog
    create_effect(move || {
        let mut combined = custom_themes_sig.get_clone();
        for theme in server_themes.get_clone() {
            if !combined.iter().any(|t| t.id == theme.id) {
                combined.push(theme);
            }
        }
        CustomTheme::sync_preview_styles(&combined);
    });

    // Fetch public themes from the server (similar to open_join_modal in server_sidebar.rs)
    spawn_local_scoped(async move {
        // TODO: Сделать запрос на сервер для получения списка публичных тем каталога
        // (аналогично Api::get_public_servers(1).await в server_sidebar.rs):
        //
        // if let Ok(themes) = Api::get_public_themes().await
        //     && !themes.is_empty()
        // {
        //     if modal_context.is_theme_catalog_open.get() {
        //         server_themes.set(themes);
        //     }
        // }
    });

    let close_catalog = move |_| {
        modal_context.is_theme_catalog_open.set(false);
    };

    view! {
        div(
            class="server-modal-overlay large theme-catalog-overlay",
            on:click=close_catalog,
        ) {
            div(
                class="server-modal join-modal theme-catalog-modal",
                on:click=move |e: web_sys::MouseEvent| e.stop_propagation(),
            ) {
                div(class="server-modal-header") {
                    span { CatalogIcon()
                                            span { "Каталог тем" } }
                    button(
                        class="modal-close-icon-btn",
                        on:click=close_catalog,
                        title="Закрыть",
                    ) { CloseSmallIcon() }
                    div(class="join-modal-search theme-catalog-search") {
                        span(class="theme-catalog-search-icon") {
                            svg(viewBox="0 0 24 24") {
                                circle(cx="11", cy="11", r="7")
                                line(x1="20", y1="20", x2="16.35", y2="16.35")
                            }
                        }
                        input(
                            r#type="text",
                            placeholder="Поиск тем по названию, автору или описанию...",
                            bind:value=search_query,
                        )
                    }
                }

                div(class="join-modal-body theme-catalog-body") {
                    (move || {
                        let query = search_query.get_clone().trim().to_lowercase();
                        let filtered: Vec<CustomTheme> = server_themes
                            .get_clone()
                            .into_iter()
                            .filter(|t| {
                                if query.is_empty() {
                                    true
                                } else {
                                    t.name.to_lowercase().contains(&query)
                                        || t.author.to_lowercase().contains(&query)
                                        || t.description.to_lowercase().contains(&query)
                                }
                            })
                            .collect();

                        if filtered.is_empty() {
                            view! {
                                div(class="join-modal-empty") {
                                    span { "По вашему запросу темы не найдены" }
                                }
                            }
                        } else {
                            let cards: Vec<View> = filtered
                                .into_iter()
                                .map(|cat_theme| {
                                    let tid = cat_theme.id.clone();
                                    let tid_check = cat_theme.id.clone();
                                    let theme_for_install = cat_theme.clone();
                                    let theme_for_download = cat_theme.clone();
                                    let name = cat_theme.name.clone();
                                    let author = format!("by @{}", cat_theme.author);
                                    let desc = cat_theme.description.clone();

                                    view! {
                                        div(
                                            class="theme-preview-card custom",
                                            data-custom-theme=tid,
                                        ) {
                                            div(class="theme-mockup custom-mockup") {
                                                div(class="mockup-sidebar") {
                                                    div(class="mockup-dot home")
                                                    div(class="mockup-dot")
                                                    div(class="mockup-dot plus")
                                                }
                                                div(class="mockup-channels") {
                                                    div(class="mockup-line active")
                                                    div(class="mockup-line")
                                                    div(class="mockup-line")
                                                }
                                                div(class="mockup-chat") {
                                                    div(class="mockup-msg") {
                                                        div(class="mockup-avatar")
                                                        div(class="mockup-bubble")
                                                    }
                                                    div(class="mockup-divider")
                                                    div(class="mockup-input-row") {
                                                        div(class="mockup-input")
                                                        div(class="mockup-send")
                                                    }
                                                }
                                            }
                                            div(class="theme-card-info") {
                                                div(class="theme-card-title-row") {
                                                    span(class="theme-card-name") { (name) }
                                                    span(class="theme-card-badge custom-badge") { (author) }
                                                }
                                                p(class="theme-card-desc") { (desc) }
                                                div(class="theme-swatches") {
                                                    span(class="swatch custom-1")
                                                    span(class="swatch custom-2")
                                                    span(class="swatch custom-3")
                                                    span(class="swatch custom-4")
                                                    span(class="swatch custom-5")
                                                    span(class="swatch custom-6")
                                                    span(class="swatch custom-7")
                                                }
                                                div(class="theme-card-actions") {
                                                    button(
                                                        class="theme-card-btn install",
                                                        on:click=move |_| {
                                                            let new_theme = theme_for_install.clone();
                                                            let new_id = new_theme.id.clone();
                                                            let mut list = custom_themes_sig.get_clone();
                                                            if let Some(existing) = list.iter_mut().find(|t| t.id == new_id) {
                                                                *existing = new_theme;
                                                            } else {
                                                                list.push(new_theme);
                                                            }
                                                            CustomTheme::save_all(&list);
                                                            custom_themes_sig.set(list);
                                                            current_theme.set(AppTheme::Custom(new_id));
                                                        },
                                                    ) {
                                                        (move || {
                                                            let installed = custom_themes_sig
                                                                .get_clone()
                                                                .iter()
                                                                .any(|t| t.id == tid_check);
                                                            if installed {
                                                                view! { CheckIcon() span { "Установлена (Применить)" } }
                                                            } else {
                                                                view! { DownloadIcon() span { "Установить тему" } }
                                                            }
                                                        })
                                                    }
                                                    button(
                                                        class="theme-card-btn",
                                                        on:click=move |_| {
                                                            let filename = format!(
                                                                "{}.gilvave-theme.json",
                                                                theme_for_download.id
                                                            );
                                                            let content = theme_for_download.to_pretty_json();
                                                            download_json_file(&filename, &content);
                                                        },
                                                    ) {
                                                                                        DownloadIcon()
                                                                                        span { "Скачать .json" }
                                                                                    }
                                                }
                                            }
                                        }
                                    }
                                })
                                .collect();

                            view! {
                                div(class="theme-cards-grid") {
                                    (cards)
                                }
                            }
                        }
                    })
                }

                div(class="join-modal-footer") {
                    button(class="back-btn", on:click=close_catalog) {
                        ArrowLeftIcon()
                        span { "Назад" }
                    }
                }
            }
        }
    }
}

