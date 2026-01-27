use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json;
use crate::util::native::maxima_dir;

#[derive(Serialize, Deserialize, Clone)]
pub struct GameSettings {
    cloud_saves: bool,
    launch_args: String,
    exe_override: String,
    wine_prefix: String,
}

impl GameSettings 
{
    pub fn new() -> Self {
        Self {
            cloud_saves: true,
            launch_args: String::new(),
            exe_override: String::new(),
            wine_prefix: String::new(),
        }
    }

    pub fn new_with_slug(slug: &str) -> Self {
        let mut settings = Self::new();
        settings.wine_prefix = format!("/mnt/games/Games/{}/", slug);
        settings
    }
}

pub fn get_game_settings(slug: &str) -> GameSettings 
{
    let path = match maxima_dir() {
        Ok(dir) => dir.join("settings").join(format!("{}.json", slug)),
        Err(_) => return GameSettings::new_with_slug(slug),
    };

    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return GameSettings::new_with_slug(slug),
    };

    serde_json::from_str(&content).unwrap_or_else(|_| GameSettings::new_with_slug(slug))
}

pub fn save_game_settings(slug: &str, settings: &GameSettings) 
{
    if let Ok(dir) = maxima_dir() 
    {
        let path = dir.join("settings").join(format!("{}.json", slug));
        if let Ok(content) = serde_json::to_string_pretty(settings) 
        {
            let _ = std::fs::write(path, content);
        }
    }
}

#[derive(Clone)]
pub struct GameSettingsManager {
    settings: HashMap<String, GameSettings>,
}

impl GameSettingsManager {
    pub fn new() -> Self {
        Self {
            settings: HashMap::new(),
        }
    }

    pub fn get(&mut self, slug: &str) -> &GameSettings {
        self.settings.entry(slug.to_string())
            .or_insert_with(|| get_game_settings(slug))
    }

    pub fn save(&mut self, slug: &str, settings: GameSettings) {
        save_game_settings(slug, &settings);
        self.settings.insert(slug.to_string(), settings);
    }
}