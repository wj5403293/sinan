use crate::constants::APP_ROOT;
use crate::errors::AppError;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

static APP_CONFIG: OnceLock<RwLock<AppConfig>> = OnceLock::new();

pub fn init() -> Result<()> {
    let config = AppConfig::load()?;

    let _ = APP_CONFIG.set(RwLock::new(config));

    Ok(())
}

fn global() -> &'static RwLock<AppConfig> {
    APP_CONFIG.get().expect("AppConfig not initialized！")
}

pub fn get_global() -> Result<RwLockReadGuard<'static, AppConfig>> {
    global()
        .read()
        .map_err(|e| anyhow!("AppConfig 读锁已中毒，错误：{e}"))
}

pub fn get_mut_global() -> Result<RwLockWriteGuard<'static, AppConfig>> {
    global()
        .write()
        .map_err(|e| anyhow!("AppConfig 写锁已中毒，错误：{e}"))
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum StringOrVec {
    String(String),
    Vec(Vec<String>),
}

impl StringOrVec {
    pub fn as_vec(&self) -> Vec<String> {
        match self {
            StringOrVec::String(s) => vec![s.clone()],
            StringOrVec::Vec(v) => v.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AppConfig {
    pub base: BaseConfig,

    #[serde(flatten)]
    pub processes: HashMap<String, ProcessConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct BaseConfig {
    pub target: String,
    pub range: String,
    pub mode: String,
}

impl Default for BaseConfig {
    fn default() -> Self {
        Self {
            target: "default".to_string(),
            range: "default".to_string(),
            mode: "exhaustive".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ProcessConfig {
    pub depth: usize,
    pub offset: usize,
    pub addresses: Vec<u64>,
    pub modules: StringOrVec,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            depth: 6,
            offset: 400,
            addresses: vec![0x12345678],
            modules: StringOrVec::Vec(vec!["default".to_string()]),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut processes = HashMap::new();
        processes.insert("default".to_string(), ProcessConfig::default());

        Self {
            base: BaseConfig::default(),
            processes,
        }
    }
}

impl AppConfig {
    pub fn get_path() -> PathBuf {
        Path::new(APP_ROOT).join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::get_path();

        if let Ok(content) = fs::read_to_string(&path) {
            let config: Self = toml::from_str(&content)
                .map_err(|err| AppError::ConfigurationParsingFailed(err.to_string()))?;

            let updated_content = toml::to_string_pretty(&config)
                .map_err(|err| AppError::ConfigurationGenerationFailed(err.to_string()))?;

            if content != updated_content {
                fs::write(&path, &updated_content)?;
            }

            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(Self::get_path(), content)?;

        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        *self = Self::default();
        self.save()?;

        Ok(())
    }

    pub fn set_target(&mut self, new_target: String) -> Result<()> {
        if !self.processes.contains_key(&new_target) {
            self.processes.insert(
                new_target.clone(),
                ProcessConfig {
                    modules: StringOrVec::String("all".to_string()),
                    ..ProcessConfig::default()
                },
            );
        }
        self.base.target = new_target;
        self.save()?;

        Ok(())
    }

    pub fn get_target_config(&self) -> Result<&ProcessConfig> {
        let target = &self.base.target;
        let process = self
            .processes
            .get(target)
            .ok_or(AppError::TargetNotFound(target.to_string()))?;

        Ok(process)
    }

    pub fn get_mut_target_config(&mut self) -> Result<&mut ProcessConfig> {
        let target = &self.base.target;
        let process = self
            .processes
            .get_mut(target)
            .ok_or(AppError::TargetNotFound(target.to_string()))?;

        Ok(process)
    }
}
