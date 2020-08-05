use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum OpenError {
    AsyncError(String),
    NoneSelected,
    NotExists(PathBuf),
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenError::AsyncError(err) => write!(f, "Unknown error: {}", err),
            OpenError::NoneSelected => write!(f, "No save game selected"),
            OpenError::NotExists(path) => {
                write!(f, "The selected file does not exists ({})", path.display())
            }
        }
    }
}

pub async fn open_file() -> Result<PathBuf, OpenError> {
    let default_dir =
        match paths::default_save_game_dir().and_then(|p| p.to_str().map(|s| s.to_owned())) {
            Some(d) => d,
            None => panic!("No home directory set. On what can we fallback ?"),
        };

    let result: Result<Option<String>, tokio::task::JoinError> =
        tokio::task::spawn_blocking(move || {
            tinyfiledialogs::open_file_dialog(
                "Choose a file to open",
                &default_dir,
                Some((&["*.zks"], "PF Save Files")),
            )
        })
        .await;

    let file_path: String = match result {
        Ok(Some(path)) => path,
        Ok(None) => return Err(OpenError::NoneSelected),
        Err(e) => return Err(OpenError::AsyncError(format!("{}", e))),
    };

    let mut path: PathBuf = PathBuf::new();
    path.push(Path::new(&file_path));

    if path.exists() {
        Ok(path)
    } else {
        Err(OpenError::NotExists(path))
    }
}

mod paths {
    use std::path::PathBuf;

    /*
    a) Windows: %systemdrive%\users\%username%\AppData\LocalLow\Owlcat Games\Pathfinder Wrath Of The Righteous\Saved Games
    b) macOS: ~/Library/Application Support/unity.Owlcat Games.Pathfinder Wrath Of The Righteous/Saved Games/
    c) Linux: ~/.config/unity3d/Owlcat Games/Pathfinder Wrath Of The Righteous/Saved Games/
    */
    pub fn default_save_game_dir() -> Option<PathBuf> {
        os::save_game()
    }

    #[cfg(target_os = "windows")]
    mod win {
        use std::path::PathBuf;

        pub fn save_game() -> Option<PathBuf> {
            // It would be better to have directly access to a data_locallow_dir() function
            // Unfortunately the library doesn't offer it, so instead we do a path join
            dirs::data_local_dir()
                .map(|h| h.join("..\\LocalLow\\Owlcat Games\\Pathfinder Wrath Of The Righteous\\Saved Games"))
        }
    }
    #[cfg(target_os = "windows")]
    use win as os;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    mod mac {
        use std::path::PathBuf;

        pub fn save_game() -> Option<PathBuf> {
            dirs::home_dir().map(|h| h.join("Library/Application Support/unity.Owlcat Games.Pathfinder Wrath Of The Righteous/Saved Games/"))
        }
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    use mac as os;

    #[cfg(any(target_os = "linux"))]
    mod mac {
        use std::path::PathBuf;

        pub fn save_game() -> Option<PathBuf> {
            dirs::config_dir()
                .map(|h| h.join("unity3d/Owlcat Games/Pathfinder Wrath Of The Righteous/Saved Games/"))
        }
    }
    #[cfg(any(target_os = "linux"))]
    use mac as os;
}
