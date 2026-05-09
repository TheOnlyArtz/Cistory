use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotkeyBinding {
    pub value: String,
}

impl HotkeyBinding {
    pub fn parse(value: impl Into<String>) -> Result<Self, HotkeyError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(HotkeyError::InvalidBinding);
        }
        Ok(Self { value })
    }

    pub fn to_global_hotkey(&self) -> Result<HotKey, HotkeyError> {
        let mut modifiers = Modifiers::empty();
        let mut key = None;

        for token in self.value.split('+').map(str::trim).filter(|value| !value.is_empty()) {
            match normalize_token(token).as_str() {
                "CTRL" | "CONTROL" => modifiers |= Modifiers::CONTROL,
                "SHIFT" => modifiers |= Modifiers::SHIFT,
                "ALT" => modifiers |= Modifiers::ALT,
                "SUPER" | "META" | "WIN" | "LOGO" => modifiers |= Modifiers::SUPER,
                other => key = Some(parse_code(other)?),
            }
        }

        let key = key.ok_or(HotkeyError::MissingKey)?;
        Ok(HotKey::new(Some(modifiers), key))
    }

    pub fn to_portal_trigger(&self) -> Result<String, HotkeyError> {
        let mut trigger = Vec::new();

        for token in self.value.split('+').map(str::trim).filter(|value| !value.is_empty()) {
            let normalized = normalize_token(token);
            let mapped = match normalized.as_str() {
                "CTRL" | "CONTROL" => "CTRL".to_string(),
                "SHIFT" => "SHIFT".to_string(),
                "ALT" => "ALT".to_string(),
                "SUPER" | "META" | "WIN" | "LOGO" => "SUPER".to_string(),
                other => normalize_key_name(other)?,
            };
            trigger.push(mapped);
        }

        if trigger.is_empty() {
            return Err(HotkeyError::InvalidBinding);
        }

        Ok(trigger.join("+"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyBackendKind {
    X11,
    Portal,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotkeyRegistration {
    pub binding: HotkeyBinding,
    pub backend: HotkeyBackendKind,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HotkeyError {
    #[error("hotkey binding is invalid")]
    InvalidBinding,
    #[error("hotkey binding is missing a non-modifier key")]
    MissingKey,
    #[error("unsupported hotkey token: {0}")]
    UnsupportedToken(String),
    #[error("hotkey backend is unavailable on this session")]
    UnsupportedBackend,
}

pub fn detect_backend(session_type: Option<&str>) -> HotkeyBackendKind {
    match session_type {
        Some("x11") => HotkeyBackendKind::X11,
        Some("wayland") => HotkeyBackendKind::Portal,
        _ => HotkeyBackendKind::Unsupported,
    }
}

pub fn plan_registration(
    binding: HotkeyBinding,
    session_type: Option<&str>,
) -> Result<HotkeyRegistration, HotkeyError> {
    let backend = detect_backend(session_type);
    if matches!(backend, HotkeyBackendKind::Unsupported) {
        return Err(HotkeyError::UnsupportedBackend);
    }

    Ok(HotkeyRegistration { binding, backend })
}

fn normalize_token(value: &str) -> String {
    value.trim().replace('-', "_").to_ascii_uppercase()
}

fn parse_code(token: &str) -> Result<Code, HotkeyError> {
    let token = normalize_token(token);

    if token.len() == 1 {
        let byte = token.as_bytes()[0];
        return match byte {
            b'A'..=b'Z' => format!("Key{}", token)
                .parse::<Code>()
                .map_err(|_| HotkeyError::UnsupportedToken(token)),
            b'0'..=b'9' => format!("Digit{}", token)
                .parse::<Code>()
                .map_err(|_| HotkeyError::UnsupportedToken(token)),
            _ => Err(HotkeyError::UnsupportedToken(token)),
        };
    }

    if let Some(number) = token.strip_prefix('F') {
        if number.chars().all(|value| value.is_ascii_digit()) {
            return token
                .parse::<Code>()
                .map_err(|_| HotkeyError::UnsupportedToken(token));
        }
    }

    match token.as_str() {
        "SPACE" => Ok(Code::Space),
        "ENTER" | "RETURN" => Ok(Code::Enter),
        "ESC" | "ESCAPE" => Ok(Code::Escape),
        "PERIOD" => Ok(Code::Period),
        "COMMA" => Ok(Code::Comma),
        "MINUS" => Ok(Code::Minus),
        _ => Err(HotkeyError::UnsupportedToken(token)),
    }
}

fn normalize_key_name(token: &str) -> Result<String, HotkeyError> {
    let token = normalize_token(token);

    if token.len() == 1 {
        return Ok(token);
    }

    if token.starts_with('F') && token[1..].chars().all(|value| value.is_ascii_digit()) {
        return Ok(token);
    }

    match token.as_str() {
        "SPACE" => Ok("space".to_string()),
        "ENTER" | "RETURN" => Ok("Return".to_string()),
        "ESC" | "ESCAPE" => Ok("Escape".to_string()),
        "PERIOD" => Ok("period".to_string()),
        "COMMA" => Ok("comma".to_string()),
        "MINUS" => Ok("minus".to_string()),
        _ => Err(HotkeyError::UnsupportedToken(token)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_binding() {
        assert_eq!(HotkeyBinding::parse("   "), Err(HotkeyError::InvalidBinding));
    }

    #[test]
    fn maps_x11_to_x11_backend() {
        let plan = plan_registration(HotkeyBinding::parse("Super+V").expect("binding"), Some("x11"))
            .expect("plan");
        assert_eq!(plan.backend, HotkeyBackendKind::X11);
    }

    #[test]
    fn maps_wayland_to_portal_backend() {
        let plan = plan_registration(HotkeyBinding::parse("Super+V").expect("binding"), Some("wayland"))
            .expect("plan");
        assert_eq!(plan.backend, HotkeyBackendKind::Portal);
    }

    #[test]
    fn parses_letter_shortcut_for_x11() {
        let hotkey = HotkeyBinding::parse("Super+V").expect("binding").to_global_hotkey();
        assert!(hotkey.is_ok());
    }

    #[test]
    fn converts_binding_to_portal_trigger() {
        let trigger = HotkeyBinding::parse("Ctrl+Shift+V")
            .expect("binding")
            .to_portal_trigger()
            .expect("portal trigger");

        assert_eq!(trigger, "CTRL+SHIFT+V");
    }
}
