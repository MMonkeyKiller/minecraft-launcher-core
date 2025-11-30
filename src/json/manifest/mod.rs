use std::{ collections::{ HashMap, HashSet }, path::{ Path, PathBuf, MAIN_SEPARATOR_STR } };

use argument::{ Argument, ArgumentType };
use assets::AssetIndexInfo;
use download::{ DownloadInfo, DownloadType };
use java::JavaVersionInfo;
use library::Library;
use logging::LoggingEntry;
use rule::{ OperatingSystem, Rule, RuleAction };
use serde::{ Deserialize, Serialize };

use super::{ Date, EnvironmentFeatures, MCVersion, ReleaseType, VersionInfo };

pub mod argument;
pub mod assets;
pub mod download;
pub mod java;
pub mod logging;
pub mod rule;
pub mod library;
pub mod artifact;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VersionManifest {
  #[serde(default, skip_serializing_if = "HashMap::is_empty")]
  pub arguments: HashMap<ArgumentType, Vec<Argument>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub minecraft_arguments: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub asset_index: Option<AssetIndexInfo>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub assets: Option<String>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub compatibility_rules: Vec<Rule>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub compliance_level: Option<u8>,
  #[serde(default, skip_serializing_if = "HashMap::is_empty")]
  pub downloads: HashMap<DownloadType, DownloadInfo>,
  pub id: MCVersion,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub inherits_from: Option<MCVersion>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub java_version: Option<JavaVersionInfo>,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub libraries: Vec<Library>,
  #[serde(default, skip_serializing_if = "HashMap::is_empty")]
  pub logging: HashMap<DownloadType, LoggingEntry>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub main_class: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub jar: Option<MCVersion>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub minimum_launcher_version: Option<u32>,
  pub release_time: Date,
  #[serde(rename = "time")]
  pub updated_time: Date,
  #[serde(rename = "type")]
  pub release_type: ReleaseType,
}

impl VersionManifest {
  pub fn get_relevant_libraries(&self, env_features: &EnvironmentFeatures) -> Vec<&Library> {
    self.libraries
      .iter()
      .filter(|lib| lib.applies_to_current_environment(env_features))
      .collect()
  }

  pub fn get_required_files(&self, os: &OperatingSystem, env_features: &EnvironmentFeatures) -> HashSet<String> {
    let mut set = HashSet::new();
    let libraries = self.get_relevant_libraries(env_features);
    for library in libraries {
      if !library.natives.is_empty() {
        if let Some(native) = library.natives.get(os) {
          set.insert(format!("libraries/{}", library.get_artifact_path(Some(native.clone()))));
        }
      } else {
        set.insert(format!("libraries/{}", library.get_artifact_path(None)));
      }
    }
    set
  }

  pub fn get_jar(&self) -> &MCVersion {
    self.jar.as_ref().unwrap_or(self.get_id())
  }

  pub fn get_main_class(&self) -> &String {
    self.main_class.as_ref().unwrap()
  }

  pub fn get_download_url(&self, download_type: DownloadType) -> Option<&DownloadInfo> {
    self.downloads.get(&download_type)
  }

  pub fn applies_to_current_environment(&self, env_features: &EnvironmentFeatures) -> bool {
    if self.compatibility_rules.is_empty() {
      return true;
    }

    let mut action = RuleAction::Disallow;
    for rule in &self.compatibility_rules {
      if let Some(applied_action) = rule.get_applied_action(env_features) {
        action = applied_action;
      }
    }

    action == RuleAction::Allow
  }

  pub fn get_classpath(&self, _os: &OperatingSystem, mc_dir: &Path, env_features: &EnvironmentFeatures) -> Vec<PathBuf> {
    let mut set = HashSet::new();
    let libraries = self.get_relevant_libraries(env_features);
    for library in libraries {
      if library.natives.is_empty() {
        set.insert(mc_dir.join("libraries").join(library.get_artifact_path(None).replace('/', MAIN_SEPARATOR_STR)));
      }
    }

    let jar_id = self.get_jar().to_string();
    set.insert(mc_dir.join("versions").join(&jar_id).join(format!("{jar_id}.jar")));
    set.into_iter().collect()
  }
}

impl VersionInfo for VersionManifest {
  fn get_id(&self) -> &MCVersion {
    &self.id
  }

  fn get_type(&self) -> &ReleaseType {
    &self.release_type
  }

  fn get_updated_time(&self) -> &Date {
    &self.updated_time
  }

  fn get_release_time(&self) -> &Date {
    &self.release_time
  }
}
