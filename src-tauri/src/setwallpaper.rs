use std::path::Path;

pub fn set(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(format!("file does not exist: {}", path.display()));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" => {}
        _ => return Err(format!("unsupported filetype: .{ext}")),
    }

    platform::set_wallpaper(path.to_str().unwrap())
}

#[cfg(target_os = "macos")]
mod platform {
    use objc::runtime::Object;
    use std::ffi::CString;

    pub fn set_wallpaper(path: &str) -> Result<(), String> {
        // Count attached screens first (fast, no alloc needed)
        let count: usize = unsafe {
            let screens: *mut Object = msg_send![class!(NSScreen), screens];
            msg_send![screens, count]
        };

        if count == 0 {
            return Err("no screens found".to_string());
        }

        // Spawn one thread per screen so all screens update in parallel.
        // Each thread creates its own Obj-C objects — no cross-thread sharing.
        let handles: Vec<_> = (0..count)
            .map(|i| {
                let path = path.to_owned();
                std::thread::spawn(move || -> Result<(), String> {
                    let path_c = CString::new(path.as_str())
                        .map_err(|e| format!("invalid path: {e}"))?;
                    unsafe {
                        let alloc: *mut Object = msg_send![class!(NSString), alloc];
                        let path_ns: *mut Object =
                            msg_send![alloc, initWithUTF8String: path_c.as_ptr()];
                        let url: *mut Object =
                            msg_send![class!(NSURL), fileURLWithPath: path_ns];
                        let options: *mut Object =
                            msg_send![class!(NSDictionary), dictionary];
                        let workspace: *mut Object =
                            msg_send![class!(NSWorkspace), sharedWorkspace];
                        let screens: *mut Object =
                            msg_send![class!(NSScreen), screens];
                        let screen: *mut Object =
                            msg_send![screens, objectAtIndex: i];

                        let ok: bool = msg_send![
                            workspace,
                            setDesktopImageURL: url
                            forScreen: screen
                            options: options
                            error: std::ptr::null_mut::<*mut Object>()
                        ];

                        let _: () = msg_send![path_ns, release];

                        if ok {
                            Ok(())
                        } else {
                            Err(format!("setDesktopImageURL failed for screen {}", i))
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle
                .join()
                .map_err(|_| "wallpaper thread panicked".to_string())??;
        }

        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use std::env;
    use std::process::Command;

    pub fn set_wallpaper(path: &str) -> Result<(), String> {
        let desktop = env::var("DESKTOP_SESSION").unwrap_or_default();
        let cmd = match desktop.as_str() {
            "plasma" => format!(
                "dbus-send --session --dest=org.kde.plasmashell --type=method_call /PlasmaShell org.kde.PlasmaShell.evaluateScript string:\"var allDesktops = desktops(); for (i = 0; i < allDesktops.length; i++) {{ d = allDesktops[i]; d.wallpaperPlugin = 'org.kde.image'; d.currentConfigGroup = Array('Wallpaper', 'org.kde.image', 'General'); d.writeConfig('Image', 'file://{path}'); }}\""
            ),
            "gnome" | "gnome-wayland" | "ubuntu" => format!(
                "gsettings set org.gnome.desktop.background picture-uri file://{path} && gsettings set org.gnome.desktop.background picture-uri-dark file://{path}"
            ),
            "cinnamon" => format!(
                "gsettings set org.cinnamon.desktop.background picture-uri file://{path}"
            ),
            "mate" => format!(
                "gsettings set org.mate.background picture-filename \"{path}\""
            ),
            "budgie-desktop" => format!(
                "gsettings set org.gnome.desktop.background picture-uri \"file://{path}\""
            ),
            "xfce" => format!(
                "for prop in $(xfconf-query -c xfce4-desktop -l | grep last-image); do xfconf-query -c xfce4-desktop -p $prop -s '{path}'; done"
            ),
            _ => return Err(format!("unsupported desktop environment: {desktop}")),
        };

        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .map_err(|e| format!("failed to run command: {e}"))?;

        if !output.status.success() {
            return Err(format!(
                "set wallpaper failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "user32")]
    extern "system" {
        fn SystemParametersInfoW(
            uiAction: u32,
            uiParam: u32,
            pvParam: *const u16,
            fWinIni: u32,
        ) -> i32;
    }

    const SPI_SETDESKWALLPAPER: u32 = 0x0014;
    const SPIF_UPDATEINIFILE: u32 = 0x01;
    const SPIF_SENDCHANGE: u32 = 0x02;

    pub fn set_wallpaper(path: &str) -> Result<(), String> {
        let wide: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let result = unsafe {
            SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                0,
                wide.as_ptr(),
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            )
        };

        if result == 0 {
            Err("SystemParametersInfoW failed".into())
        } else {
            Ok(())
        }
    }
}
