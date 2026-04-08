use std::collections::HashMap;

use semver::{Version, VersionReq};

use crate::PluginSpec;

pub fn parse_version_range(constraint: &str) -> Result<VersionReq, semver::Error> {
    VersionReq::parse(constraint)
}

pub fn deduplicate_dependencies(specs: &[PluginSpec]) -> Vec<PluginSpec> {
    let mut seen: HashMap<String, PluginSpec> = HashMap::new();

    for spec in specs {
        for dep in &spec.dependencies {
            match seen.get_mut(&dep.url) {
                Some(existing) => {
                    if let Some(new_version) = &dep.version {
                        match &existing.version {
                            Some(existing_version) => {
                                existing.version =
                                    Some(format!("{}, {}", existing_version, new_version));
                            }
                            None => {
                                existing.version = Some(new_version.clone());
                            }
                        }
                    }
                }
                None => {
                    seen.insert(dep.url.clone(), dep.clone());
                }
            }
        }
    }

    seen.into_values().collect()
}

pub fn filter_tags_by_range<'a>(tags: &'a [String], range: &VersionReq) -> Vec<(&'a str, Version)> {
    let mut matching: Vec<(&str, Version)> = tags
        .iter()
        .filter_map(|tag| {
            let version_str = tag.strip_prefix('v').unwrap_or(tag);
            let version = Version::parse(version_str).ok()?;
            if range.matches(&version) {
                Some((tag.as_str(), version))
            } else {
                None
            }
        })
        .collect();

    matching.sort_by(|a, b| b.1.cmp(&a.1));
    matching
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_range() {
        let range = parse_version_range(">=1.0, <2.0").unwrap();
        assert!(range.matches(&Version::new(1, 5, 0)));
        assert!(!range.matches(&Version::new(2, 0, 0)));
        assert!(!range.matches(&Version::new(0, 9, 0)));
    }

    #[test]
    fn filter_tags_returns_sorted_matches() {
        let tags = vec![
            "v0.5.0".to_string(),
            "v1.0.0".to_string(),
            "v1.3.0".to_string(),
            "v2.0.0".to_string(),
        ];
        let range = parse_version_range(">=1.0, <2.0").unwrap();
        let result = filter_tags_by_range(&tags, &range);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "v1.3.0");
        assert_eq!(result[1].0, "v1.0.0");
    }

    #[test]
    fn filter_tags_no_match() {
        let tags = vec!["v0.1.0".to_string(), "v0.2.0".to_string()];
        let range = parse_version_range(">=1.0").unwrap();
        let result = filter_tags_by_range(&tags, &range);
        assert!(result.is_empty());
    }

    #[test]
    fn filter_tags_without_v_prefix() {
        let tags = vec!["1.0.0".to_string(), "2.0.0".to_string()];
        let range = parse_version_range(">=1.0, <2.0").unwrap();
        let result = filter_tags_by_range(&tags, &range);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "1.0.0");
    }

    #[test]
    fn parse_invalid_range() {
        assert!(parse_version_range("not a version").is_err());
    }

    #[test]
    fn dedup_same_dependency_merges_versions() {
        let specs = vec![
            PluginSpec {
                url: "https://github.com/a/plugin-a".to_string(),
                name: None,
                branch: None,
                version: None,
                dependencies: vec![PluginSpec {
                    url: "https://github.com/user/lib".to_string(),
                    name: None,
                    branch: None,
                    version: Some(">=1.0".to_string()),
                    dependencies: vec![],
                }],
            },
            PluginSpec {
                url: "https://github.com/b/plugin-b".to_string(),
                name: None,
                branch: None,
                version: None,
                dependencies: vec![PluginSpec {
                    url: "https://github.com/user/lib".to_string(),
                    name: None,
                    branch: None,
                    version: Some("<2.0".to_string()),
                    dependencies: vec![],
                }],
            },
        ];

        let deduped = deduplicate_dependencies(&specs);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].url, "https://github.com/user/lib");
        assert_eq!(deduped[0].version.as_deref(), Some(">=1.0, <2.0"));
    }

    #[test]
    fn dedup_identical_dependencies() {
        let specs = vec![
            PluginSpec {
                url: "https://github.com/a/plugin-a".to_string(),
                name: None,
                branch: None,
                version: None,
                dependencies: vec![PluginSpec {
                    url: "https://github.com/user/lib".to_string(),
                    name: None,
                    branch: None,
                    version: None,
                    dependencies: vec![],
                }],
            },
            PluginSpec {
                url: "https://github.com/b/plugin-b".to_string(),
                name: None,
                branch: None,
                version: None,
                dependencies: vec![PluginSpec {
                    url: "https://github.com/user/lib".to_string(),
                    name: None,
                    branch: None,
                    version: None,
                    dependencies: vec![],
                }],
            },
        ];

        let deduped = deduplicate_dependencies(&specs);
        assert_eq!(deduped.len(), 1);
        assert!(deduped[0].version.is_none());
    }
}
