use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct InputEvent {
    kind: String,
    #[serde(default)]
    x_norm: Option<f64>,
    #[serde(default)]
    y_norm: Option<f64>,
    #[serde(default)]
    button: Option<String>,
    #[serde(default)]
    delta_y: Option<i32>,
    #[serde(default)]
    code: Option<String>,
}

pub fn inject_from_json(payload: &str) -> Result<(), String> {
    let event: InputEvent =
        serde_json::from_str(payload).map_err(|err| format!("input json parse failed: {err}"))?;
    inject_event(event)
}

#[cfg(windows)]
fn inject_event(event: InputEvent) -> Result<(), String> {
    use std::mem::size_of;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
        KEYEVENTF_SCANCODE, MAPVK_VK_TO_VSC, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
        MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE,
        MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT,
        MOUSE_EVENT_FLAGS, MapVirtualKeyW, VIRTUAL_KEY, VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6,
        VK_7, VK_8, VK_9, VK_A, VK_B, VK_BACK, VK_C, VK_CONTROL, VK_D, VK_DOWN, VK_E, VK_ESCAPE,
        VK_F, VK_G, VK_H, VK_I, VK_J, VK_K, VK_L, VK_LEFT, VK_M, VK_MENU, VK_N, VK_O, VK_P, VK_Q,
        VK_R, VK_RETURN, VK_RIGHT, VK_S, VK_SHIFT, VK_SPACE, VK_T, VK_TAB, VK_U, VK_UP, VK_V, VK_W,
        VK_X, VK_Y, VK_Z,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    fn send_inputs(inputs: &[INPUT]) -> Result<(), String> {
        let sent = unsafe { SendInput(inputs, size_of::<INPUT>() as i32) };
        if sent as usize != inputs.len() {
            return Err(format!("send_input partial send expected={} sent={sent}", inputs.len()));
        }
        Ok(())
    }

    fn clamp_norm(v: f64) -> f64 {
        v.clamp(0.0, 1.0)
    }

    fn make_mouse_input(flags: MOUSE_EVENT_FLAGS, data: i32, dx: i32, dy: i32) -> INPUT {
        INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx,
                    dy,
                    mouseData: data as u32,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn make_key_input(vk: VIRTUAL_KEY, key_up: bool) -> INPUT {
        let scan = unsafe { MapVirtualKeyW(vk.0 as u32, MAPVK_VK_TO_VSC) } as u16;
        let mut flags = KEYEVENTF_SCANCODE;
        if key_up {
            flags |= KEYEVENTF_KEYUP;
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(0),
                    wScan: scan,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn map_button(name: &str, down: bool) -> Option<MOUSE_EVENT_FLAGS> {
        match (name, down) {
            ("left", true) => Some(MOUSEEVENTF_LEFTDOWN),
            ("left", false) => Some(MOUSEEVENTF_LEFTUP),
            ("right", true) => Some(MOUSEEVENTF_RIGHTDOWN),
            ("right", false) => Some(MOUSEEVENTF_RIGHTUP),
            ("middle", true) => Some(MOUSEEVENTF_MIDDLEDOWN),
            ("middle", false) => Some(MOUSEEVENTF_MIDDLEUP),
            _ => None,
        }
    }

    fn map_code_to_vk(code: &str) -> Option<VIRTUAL_KEY> {
        match code {
            "KeyA" => Some(VK_A),
            "KeyB" => Some(VK_B),
            "KeyC" => Some(VK_C),
            "KeyD" => Some(VK_D),
            "KeyE" => Some(VK_E),
            "KeyF" => Some(VK_F),
            "KeyG" => Some(VK_G),
            "KeyH" => Some(VK_H),
            "KeyI" => Some(VK_I),
            "KeyJ" => Some(VK_J),
            "KeyK" => Some(VK_K),
            "KeyL" => Some(VK_L),
            "KeyM" => Some(VK_M),
            "KeyN" => Some(VK_N),
            "KeyO" => Some(VK_O),
            "KeyP" => Some(VK_P),
            "KeyQ" => Some(VK_Q),
            "KeyR" => Some(VK_R),
            "KeyS" => Some(VK_S),
            "KeyT" => Some(VK_T),
            "KeyU" => Some(VK_U),
            "KeyV" => Some(VK_V),
            "KeyW" => Some(VK_W),
            "KeyX" => Some(VK_X),
            "KeyY" => Some(VK_Y),
            "KeyZ" => Some(VK_Z),
            "Digit0" => Some(VK_0),
            "Digit1" => Some(VK_1),
            "Digit2" => Some(VK_2),
            "Digit3" => Some(VK_3),
            "Digit4" => Some(VK_4),
            "Digit5" => Some(VK_5),
            "Digit6" => Some(VK_6),
            "Digit7" => Some(VK_7),
            "Digit8" => Some(VK_8),
            "Digit9" => Some(VK_9),
            "Enter" => Some(VK_RETURN),
            "Backspace" => Some(VK_BACK),
            "Tab" => Some(VK_TAB),
            "Escape" => Some(VK_ESCAPE),
            "Space" => Some(VK_SPACE),
            "ShiftLeft" | "ShiftRight" => Some(VK_SHIFT),
            "ControlLeft" | "ControlRight" => Some(VK_CONTROL),
            "AltLeft" | "AltRight" => Some(VK_MENU),
            "ArrowUp" => Some(VK_UP),
            "ArrowDown" => Some(VK_DOWN),
            "ArrowLeft" => Some(VK_LEFT),
            "ArrowRight" => Some(VK_RIGHT),
            _ => None,
        }
    }

    match event.kind.as_str() {
        "mouse_move_abs" => {
            let x = clamp_norm(event.x_norm.ok_or_else(|| "x_norm missing".to_owned())?);
            let y = clamp_norm(event.y_norm.ok_or_else(|| "y_norm missing".to_owned())?);
            let width = unsafe { GetSystemMetrics(SM_CXSCREEN) }.max(1) as f64;
            let height = unsafe { GetSystemMetrics(SM_CYSCREEN) }.max(1) as f64;
            let abs_x = ((x * (width - 1.0)) * (65535.0 / (width - 1.0))).round() as i32;
            let abs_y = ((y * (height - 1.0)) * (65535.0 / (height - 1.0))).round() as i32;
            let input =
                make_mouse_input(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, 0, abs_x, abs_y);
            send_inputs(&[input])
        }
        "mouse_down" => {
            let button = event.button.ok_or_else(|| "button missing".to_owned())?;
            let flags = map_button(&button, true).ok_or_else(|| "invalid mouse button".to_owned())?;
            let input = make_mouse_input(flags, 0, 0, 0);
            send_inputs(&[input])
        }
        "mouse_up" => {
            let button = event.button.ok_or_else(|| "button missing".to_owned())?;
            let flags =
                map_button(&button, false).ok_or_else(|| "invalid mouse button".to_owned())?;
            let input = make_mouse_input(flags, 0, 0, 0);
            send_inputs(&[input])
        }
        "mouse_wheel" => {
            let delta = event.delta_y.ok_or_else(|| "delta_y missing".to_owned())?;
            let input = make_mouse_input(MOUSEEVENTF_WHEEL, delta, 0, 0);
            send_inputs(&[input])
        }
        "key_down" => {
            let code = event.code.ok_or_else(|| "code missing".to_owned())?;
            let vk = map_code_to_vk(&code).ok_or_else(|| format!("unmapped key code: {code}"))?;
            let input = make_key_input(vk, false);
            send_inputs(&[input])
        }
        "key_up" => {
            let code = event.code.ok_or_else(|| "code missing".to_owned())?;
            let vk = map_code_to_vk(&code).ok_or_else(|| format!("unmapped key code: {code}"))?;
            let input = make_key_input(vk, true);
            send_inputs(&[input])
        }
        _ => Err(format!("unsupported input kind: {}", event.kind)),
    }
}

#[cfg(not(windows))]
fn inject_event(_event: InputEvent) -> Result<(), String> {
    Err("input injection is only supported on Windows".to_owned())
}
