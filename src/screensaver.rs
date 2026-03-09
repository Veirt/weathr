//! Screensaver support

#[cfg(target_os = "windows")]
pub mod windows {
    use std::{env, process::Command};

    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Console::{
        CONSOLE_SCREEN_BUFFER_INFO, COORD, GetConsoleScreenBufferInfo, GetConsoleWindow,
        GetStdHandle, STD_OUTPUT_HANDLE, SetConsoleScreenBufferSize,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GWL_STYLE, GetWindowLongW, SW_MAXIMIZE, SetWindowLongW, ShowWindow, WS_CAPTION,
        WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME,
    };

    use crate::config::Config;

    pub fn is_screensaver() -> bool {
        let binding = env::args().collect::<Vec<String>>();
        let args = binding.first().unwrap();
        args.contains(&".scr".to_string())
    }

    pub fn normalize_args() -> Vec<String> {
        env::args()
            .map(|arg| {
                if let Some(rest) = arg.strip_prefix('/') {
                    if rest.len() == 1 && rest.chars().next().unwrap().is_ascii_alphabetic() {
                        match rest {
                            "S" | "s" | "c" => "--fullscreen".to_string(),

                            _ => {
                                format!("-{rest}")
                            }
                        }
                    } else {
                        arg
                    }
                } else {
                    arg
                }
            })
            .collect()
    }

    /// Only useful in Windows 11, cannot run inside Windows Terminal
    /// This handles all the logic for handling default CLI arguments for `.scr` files
    /// Also includes logic for relaunching in conhost
    pub fn init_screensaver() -> windows::core::Result<()> {
        let args = normalize_args();

        let cfg_path = Config::get_config_path().unwrap();

        if args.len() == 1 {
            println!(
                "Opening config file in system text editor: {}",
                cfg_path.display()
            );
            Command::new("notepad.exe")
                .arg(cfg_path)
                .spawn()
                .expect("Failed to open config file in system text editor");
            std::process::exit(1);
        }

        if let Some(config) = args.get(1)
            && config.contains("/c:")
        {
            println!("Waiting for notepad to close...");
            Command::new("notepad.exe")
                .arg(cfg_path)
                .output()
                .expect("Failed to open config file in system text editor");
        }

        if !args.contains(&"--forked".to_string()) {
            relaunch_in_conhost();
        } // Always relaunch in conhost - Best way to avoid Win Terminal in Win11
        // make_full_screen()

        Ok(())
    }

    /// Makes any console window fullscreen
    /// In Windows 11 this creates some rather weird artifacts due to `Windows Terminal`
    /// But it works fine in Windows 10, as a result ensure conhost.exe is the owner of the console
    pub fn make_full_screen() -> windows::core::Result<()> {
        unsafe {
            let hwnd: HWND = GetConsoleWindow();

            let style = GetWindowLongW(hwnd, GWL_STYLE);

            let new_style = style
                & !(WS_CAPTION.0 as i32
                    | WS_THICKFRAME.0 as i32
                    | WS_MINIMIZEBOX.0 as i32
                    | WS_MAXIMIZEBOX.0 as i32
                    | WS_SYSMENU.0 as i32);

            SetWindowLongW(hwnd, GWL_STYLE, new_style);

            if !hwnd.0.is_null() {
                ShowWindow(hwnd, SW_MAXIMIZE).unwrap();
            } else {
                println!("No console window found.");
            }

            let handle = GetStdHandle(STD_OUTPUT_HANDLE)?;

            let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
            GetConsoleScreenBufferInfo(handle, &mut info)?;

            let window_width = info.srWindow.Right - info.srWindow.Left + 1;
            let window_height = info.srWindow.Bottom - info.srWindow.Top + 1;

            SetConsoleScreenBufferSize(
                handle,
                COORD {
                    X: window_width,
                    Y: window_height,
                },
            )?;
        }

        Ok(())
    }

    /// Only useful in Windows 11, cannot run inside Windows Terminal
    pub fn relaunch_in_conhost() -> ! {
        let mut args = normalize_args();

        args.push("--forked".to_string());

        Command::new("conhost")
            .args(args)
            .spawn()
            .expect("Failed to relaunch in conhost");

        std::process::exit(0);
    }
}
