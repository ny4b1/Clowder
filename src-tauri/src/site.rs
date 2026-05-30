use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Site {
    #[default]
    E621,
    E6ai,
}

impl Site {
    pub const ALL: [Site; 2] = [Site::E621, Site::E6ai];

    pub fn host(self) -> &'static str {
        match self {
            Site::E621 => "e621.net",
            Site::E6ai => "e6ai.net",
        }
    }

    pub fn media_hosts(self) -> &'static [&'static str] {
        match self {
            Site::E621 => &[
                "e621.net",
                "static1.e621.net",
                "static2.e621.net",
                "e926.net",
                "static1.e926.net",
                "static2.e926.net",
            ],
            Site::E6ai => &["e6ai.net", "static1.e6ai.net", "static2.e6ai.net"],
        }
    }

    pub fn credential_domains(self) -> &'static [&'static str] {
        match self {
            Site::E621 => &["e621.net", "e926.net"],
            Site::E6ai => &["e6ai.net"],
        }
    }

    pub fn keychain_account(self) -> &'static str {
        match self {
            Site::E621 => "e621",
            Site::E6ai => "e6ai",
        }
    }

    pub fn from_media_host(host: &str) -> Option<Site> {
        Site::ALL
            .into_iter()
            .find(|site| site.media_hosts().contains(&host))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_media_host_matches_known_hosts() {
        assert_eq!(Site::from_media_host("static1.e621.net"), Some(Site::E621));
        assert_eq!(Site::from_media_host("e926.net"), Some(Site::E621));
        assert_eq!(Site::from_media_host("static2.e6ai.net"), Some(Site::E6ai));
        assert_eq!(Site::from_media_host("e6ai.net"), Some(Site::E6ai));
        assert_eq!(Site::from_media_host("example.com"), None);
    }

    #[test]
    fn keychain_accounts_are_distinct() {
        assert_ne!(Site::E621.keychain_account(), Site::E6ai.keychain_account());
    }

    #[test]
    fn serde_uses_snake_case() {
        assert_eq!(serde_json::to_string(&Site::E621).unwrap(), "\"e621\"");
        assert_eq!(serde_json::to_string(&Site::E6ai).unwrap(), "\"e6ai\"");
        let parsed: Site = serde_json::from_str("\"e6ai\"").unwrap();
        assert_eq!(parsed, Site::E6ai);
    }
}
