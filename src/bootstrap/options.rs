use std::{ path::PathBuf, collections::HashMap, fmt::Debug, sync::Arc };

use derive_builder::Builder;
use serde_json::Value;

use crate::{ json::{ manifest::rule::RuleFeatureType, EnvironmentFeatures, MCVersion }, progress_reporter::ProgressReporter };

use super::auth::UserAuthentication;

#[derive(Debug, Clone, Copy)]
pub struct MinecraftResolution(u32, u32);

impl MinecraftResolution {
  pub fn new(width: u32, height: u32) -> Self {
    Self(width, height)
  }

  pub fn width(&self) -> u32 {
    self.0
  }

  pub fn height(&self) -> u32 {
    self.1
  }
}

#[derive(Debug, Clone)]
pub struct LauncherOptions {
  pub launcher_name: String,
  pub launcher_version: String,
}

impl LauncherOptions {
  pub fn new(launcher_name: &str, launcher_version: &str) -> Self {
    Self { launcher_name: launcher_name.to_string(), launcher_version: launcher_version.to_string() }
  }
}

#[derive(Debug, Clone, Default)]
pub enum ProxyOptions {
  Proxy {
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
  },
  #[default] NoProxy,
}

impl ProxyOptions {
  pub fn create_http_proxy(&self) -> Option<reqwest::Proxy> {
    if let ProxyOptions::Proxy { host, port, username, password } = self {
      let mut proxy = reqwest::Proxy::all(format!("{}:{}", host, port)).ok()?;
      if let (Some(username), Some(password)) = (username, password) {
        proxy = proxy.basic_auth(username, password);
      }
      Some(proxy)
    } else {
      None
    }
  }
}

#[derive(Debug, Builder)]
#[builder(pattern = "owned", setter(strip_option))]
pub struct GameOptions {
  pub version: MCVersion,
  pub game_dir: PathBuf,
  #[builder(default)]
  pub proxy: ProxyOptions,
  #[builder(default)]
  pub resolution: Option<MinecraftResolution>,
  pub java_path: PathBuf,
  pub authentication: UserAuthentication,
  #[builder(default)]
  pub demo: Option<bool>,
  #[builder(default)]
  pub launcher_options: Option<LauncherOptions>,
  #[builder(default)]
  pub substitutor_overrides: HashMap<String, String>,
  #[builder(default)]
  pub jvm_args: Option<Vec<String>>,

  #[builder(default, setter(custom))]
  pub progress_reporter: Arc<ProgressReporter>,

  #[builder(default = "16")]
  pub max_concurrent_downloads: u16,
  #[builder(default = "5")]
  pub max_download_attempts: u8,
}

impl GameOptions {
  pub fn env_features(&self) -> EnvironmentFeatures {
    let mut env_features = EnvironmentFeatures::new();
    if let Some(demo) = self.demo {
      env_features.set_feature(RuleFeatureType::IsDemoUser, Value::Bool(demo));
    }
    if self.resolution.is_some() {
      env_features.set_feature(RuleFeatureType::HasCustomResolution, Value::Bool(true));
    }
    // TODO:
    env_features
  }
}

impl GameOptionsBuilder {
  pub fn progress_reporter(self, progress_reporter: ProgressReporter) -> Self {
    self.progress_reporter_arc(&Arc::new(progress_reporter))
  }

  pub fn progress_reporter_arc(mut self, arc: &Arc<ProgressReporter>) -> Self {
    self.progress_reporter = Some(Arc::clone(arc));
    self
  }
}
