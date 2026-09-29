use crate::models::{AppSettings, AppError, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

const APP_NAME: &str = "mp3-tagger";
const SETTINGS_FILE: &str = "settings.json";

pub struct SettingsService {
    config_dir: PathBuf,
}

impl SettingsService {
    pub fn new() -> Result<Self> {
        let proj_dirs = ProjectDirs::from("", "", APP_NAME)
            .ok_or_else(|| AppError::Config("Could not determine config directory".to_string()))?;
        
        let config_dir = proj_dirs.config_dir().to_path_buf();
        fs::create_dir_all(&config_dir)?;
        
        Ok(Self { config_dir })
    }

    fn settings_path(&self) -> PathBuf {
        self.config_dir.join(SETTINGS_FILE)
    }

    pub fn load(&self) -> Result<AppSettings> {
        let path = self.settings_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let settings: AppSettings = serde_json::from_str(&content)?;
            Ok(settings)
        } else {
            Ok(AppSettings::default())
        }
    }

    pub fn save(&self, settings: &AppSettings) -> Result<()> {
        let path = self.settings_path();
        let content = serde_json::to_string_pretty(settings)?;
        fs::write(&path, content)?;
        Ok(())
    }
}

impl Default for SettingsService {
    fn default() -> Self {
        Self::new().expect("Failed to create SettingsService")
    }
}