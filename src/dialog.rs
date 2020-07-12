use std::path::{Path, PathBuf};

// TODO Add good format/description and use that in the UI instead of Debug
#[derive(Debug, Clone)]
pub enum OpenError {
    AsyncError(String),
    NoneSelected,
    NotExists(PathBuf),
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
        return Ok(path);
    } else {
        return Err(OpenError::NotExists(path));
    }
}

// TODO Change the path to use Wrath instead of KingMaker
mod paths {
    use std::path::PathBuf;

    /*
    a) Windows: %systemdrive%\users\%username%\AppData\LocalLow\Owlcat Games\Pathfinder Kingmaker\Saved Games
    b) macOS: ~/Library/Application Support/unity.Owlcat Games.Pathfinder Kingmaker/Saved Games/
    c) Linux: ~/.config/unity3d/Owlcat Games/Pathfinder Kingmaker/Saved Games/
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
                .map(|h| h.join("..\\LocalLow\\Owlcat Games\\Pathfinder Kingmaker\\Saved Games"))
        }
    }
    #[cfg(target_os = "windows")]
    use win as os;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    mod mac {
        use std::path::PathBuf;

        pub fn save_game() -> Option<PathBuf> {
            dirs::home_dir().map(|h| h.join("Library/Application Support/unity.Owlcat Games.Pathfinder Kingmaker/Saved Games/"))
        }
    }
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    use mac as os;

    #[cfg(any(target_os = "linux"))]
    mod mac {
        use std::path::PathBuf;

        pub fn save_game() -> Option<PathBuf> {
            dirs::config_dir()
                .map(|h| h.join("unity3d/Owlcat Games/Pathfinder Kingmaker/Saved Games/"))
        }
    }
    #[cfg(any(target_os = "linux"))]
    use mac as os;
}
