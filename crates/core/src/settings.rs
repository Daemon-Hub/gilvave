pub const BASE_HTTP_URL: &str = "https://10.109.97.168:3131"; //"http://127.0.0.1:3131"; //"http://kim.netbird.cloud:3131";
pub const BASE_WS_URL: &str = "wss://10.109.97.168:3131/ws"; //"ws://127.0.0.1:3131/ws"; //"ws://kim.netbird.cloud:3131/ws";

use serde::{Deserialize, Serialize};

pub fn deserialize_optional_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrNumberVisitor;

    impl<'de> serde::de::Visitor<'de> for StringOrNumberVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string, number, or null")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value.to_string()))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value.to_string()))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value.to_string()))
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value.to_string()))
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value.to_string()))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(StringOrNumberVisitor)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(StringOrNumberVisitor)
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(default)]
pub struct DeviceInfo {
    pub platform: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_number")]
    pub os_version: Option<String>,
    pub client: String, // "desktop" | "mobile" | "web"
    pub client_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_string_or_number"
    )]
    pub browser_version: Option<String>,
}

impl DeviceInfo {
    /// client_version берётся из Cargo.toml пакета — не захардкожен в коде
    pub fn client_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Готовая JSON-строка для отправки на сервер и записи в JSONB-колонку
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
pub fn collect_desktop() -> DeviceInfo {
    use sysinfo::System;

    // name() → "Windows" / "macOS" / "Linux" / "Android"
    // os_version() → "11" / "15.1" / "Ubuntu 22.04"
    // host_name() → "DESKTOP-7K2F9A" / hostname машины
    // cpu_arch()  → "x86_64" / "aarch64"
    DeviceInfo {
        platform: System::name(),
        os_version: System::os_version(),
        client: "desktop".into(),
        client_version: DeviceInfo::client_version(),
        device_name: System::host_name(),
        browser: None,
        browser_version: None,
    }
}
#[cfg(target_arch = "wasm32")]
pub fn collect_web() -> DeviceInfo {
    let user_agent = web_sys::window()
        .and_then(|w| Some(w.navigator()))
        .and_then(|n| n.user_agent().ok())
        .unwrap_or_default();

    let parser = woothee::parser::Parser::new();
    let parsed = parser.parse(&user_agent).unwrap_or_default();

    DeviceInfo {
        // woothee даёт "Windows" / "Mac OSX" / "Android" / "Linux"
        platform: Some(parsed.os.to_string()),
        // "NT 10.0" для Windows / "10.15.6" для macOS
        os_version: Some(parsed.os_version.to_string()),
        client: "web".into(),
        client_version: DeviceInfo::client_version(),
        device_name: None,
        browser: Some(parsed.name.to_string()),
        browser_version: Some(parsed.version.to_string()),
    }
}
#[cfg(target_os = "android")]
fn collect_android() -> DeviceInfo {
    DeviceInfo {
        platform: Some("Android".into()),
        os_version: None,
        client: "mobile".into(),
        client_version: DeviceInfo::client_version(),
        device_name: None,
        browser: None,
        browser_version: None,
    }
}

pub fn collect_device_info() -> DeviceInfo {
    #[cfg(target_arch = "wasm32")]
    {
        collect_web()
    }
    #[cfg(target_os = "android")]
    {
        collect_android()
    }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    {
        collect_desktop()
    }
}
