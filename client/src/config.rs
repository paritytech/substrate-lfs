use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use toml;

/// Our configuration for the cache
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "provider")]
pub enum CacheConfig {
	None,
	#[serde(alias = "memory")]
	InMemory {},
	#[serde(alias = "simple")]
	SimpleDiskCache {
		path: Option<PathBuf>,
	},
}

/// Our lfs configuration file
#[derive(Serialize, Deserialize, Debug)]
pub struct LfsConfig {
	pub cache: CacheConfig,
}

impl core::default::Default for CacheConfig {
	fn default() -> CacheConfig {
		CacheConfig::None
	}
}

impl std::default::Default for LfsConfig {
	fn default() -> LfsConfig {
		LfsConfig {
			cache: Default::default(),
		}
	}
}

pub fn load_config(config_file_path: &Path) -> Result<LfsConfig, String> {
	if !config_file_path.is_file() {
		// make sure the folder exists
		if let Some(dir) = config_file_path.parent() {
			if !dir.is_dir() {
				std::fs::create_dir_all(dir)
					.map_err(|e| format!("could not create config dir: {}", e))?
			}
		}
		// we write the defaults to the config
		fs::write(
			config_file_path,
			toml::to_string(&LfsConfig::default()).expect("Handcrafted to never fail"),
		)
		.expect("Writing the LFS configuration failed");
	}

	let content = fs::read(config_file_path)
		.map_err(|e| format!("failed to open LFS configuration: {}", e))?;
	toml::from_slice::<LfsConfig>(&content)
		.map_err(|e| format!("Error parsing LFS configuration : {}", e))
}
