//! Global hotkey registration via `global-hotkey`. Default `CmdOrCtrl+Shift+W`.

use anyhow::{Context, Result};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use std::sync::{Arc, OnceLock};
use tauri::AppHandle;

use crate::popover;
use crate::state::AppState;

static MANAGER: OnceLock<GlobalHotKeyManager> = OnceLock::new();

pub fn install(handle: &AppHandle, state: &Arc<AppState>) -> Result<()> {
    let mgr = GlobalHotKeyManager::new().context("init GlobalHotKeyManager")?;
    let _ = MANAGER.set(mgr);
    let mgr = MANAGER.get().expect("just set");

    let combo = state.config.read().hotkey.clone();
    let hk = parse(&combo).unwrap_or_else(|_| HotKey::new(
        Some(Modifiers::SUPER | Modifiers::SHIFT),
        Code::KeyW,
    ));
    mgr.register(hk).context("register hotkey")?;
    let target_id = hk.id();

    let handle = handle.clone();
    std::thread::spawn(move || {
        let receiver = GlobalHotKeyEvent::receiver();
        for event in receiver {
            if event.id == target_id {
                let h = handle.clone();
                let _ = tauri::async_runtime::spawn(async move {
                    if let Err(e) = popover::toggle(&h) {
                        tracing::error!("popover toggle: {e:#}");
                    }
                });
            }
        }
    });

    Ok(())
}

/// Parses a string like "CmdOrCtrl+Shift+W" into a [`HotKey`].
pub fn parse(s: &str) -> Result<HotKey> {
    let mut mods = Modifiers::empty();
    let mut code: Option<Code> = None;
    for part in s.split('+').map(|p| p.trim()) {
        match part.to_ascii_lowercase().as_str() {
            "cmd" | "command" | "super" | "meta" | "cmdorctrl" => {
                mods |= Modifiers::SUPER;
            }
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "shift" => mods |= Modifiers::SHIFT,
            "alt" | "option" | "opt" => mods |= Modifiers::ALT,
            other => {
                code = Some(parse_code(other).with_context(|| format!("unknown key: {other}"))?);
            }
        }
    }
    let c = code.context("hotkey requires a non-modifier key")?;
    Ok(HotKey::new(Some(mods), c))
}

fn parse_code(s: &str) -> Result<Code> {
    use global_hotkey::hotkey::Code as C;
    Ok(match s {
        "a" => C::KeyA, "b" => C::KeyB, "c" => C::KeyC, "d" => C::KeyD,
        "e" => C::KeyE, "f" => C::KeyF, "g" => C::KeyG, "h" => C::KeyH,
        "i" => C::KeyI, "j" => C::KeyJ, "k" => C::KeyK, "l" => C::KeyL,
        "m" => C::KeyM, "n" => C::KeyN, "o" => C::KeyO, "p" => C::KeyP,
        "q" => C::KeyQ, "r" => C::KeyR, "s" => C::KeyS, "t" => C::KeyT,
        "u" => C::KeyU, "v" => C::KeyV, "w" => C::KeyW, "x" => C::KeyX,
        "y" => C::KeyY, "z" => C::KeyZ,
        "0" => C::Digit0, "1" => C::Digit1, "2" => C::Digit2, "3" => C::Digit3,
        "4" => C::Digit4, "5" => C::Digit5, "6" => C::Digit6, "7" => C::Digit7,
        "8" => C::Digit8, "9" => C::Digit9,
        "space" => C::Space,
        "esc" | "escape" => C::Escape,
        "tab" => C::Tab,
        "enter" | "return" => C::Enter,
        other => anyhow::bail!("unknown key: {other}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default() {
        let hk = parse("CmdOrCtrl+Shift+W").unwrap();
        assert!(hk.mods.contains(Modifiers::SUPER));
        assert!(hk.mods.contains(Modifiers::SHIFT));
    }

    #[test]
    fn parses_alt_combo() {
        let hk = parse("Cmd+Alt+P").unwrap();
        assert!(hk.mods.contains(Modifiers::ALT));
    }
}
