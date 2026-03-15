use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    pub blocked: bool,
    pub whitelist: HashSet<String>,
}

impl NetworkPolicy {
    pub fn new(blocked: bool, whitelist: Vec<String>) -> Self {
        Self {
            blocked,
            whitelist: whitelist.into_iter().collect(),
        }
    }

    pub fn allow_all() -> Self {
        Self {
            blocked: false,
            whitelist: HashSet::new(),
        }
    }

    pub fn block_all() -> Self {
        Self {
            blocked: true,
            whitelist: HashSet::new(),
        }
    }

    pub fn is_allowed(&self, url: &str) -> bool {
        if !self.blocked {
            return true;
        }
        
        // Extract domain from URL
        let domain = extract_domain(url);
        self.whitelist.iter().any(|d| domain.ends_with(d))
    }

    pub fn apply_iptables(&self, pid: u32) -> Result<()> {
        if !self.blocked {
            return Ok(());
        }

        // Create a network namespace for the process
        // This requires root privileges, so we log what would be done
        println!("   Network policy: BLOCKED (whitelist: {:?})", self.whitelist);
        
        // In production, we'd use:
        // unshare(CLONE_NEWNET) for network namespace isolation
        // or iptables rules:
        //   iptables -A OUTPUT -m owner --uid-owner <uid> -j DROP
        //   iptables -A OUTPUT -d <whitelisted> -j ACCEPT
        
        Ok(())
    }

    pub fn describe(&self) -> String {
        if !self.blocked {
            "Network: OPEN".to_string()
        } else if self.whitelist.is_empty() {
            "Network: BLOCKED".to_string()
        } else {
            format!("Network: BLOCKED (allow: {:?})", self.whitelist)
        }
    }
}

fn extract_domain(url: &str) -> String {
    let url = url.trim_start_matches("http://").trim_start_matches("https://");
    url.split('/').next().unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_all() {
        let policy = NetworkPolicy::allow_all();
        assert!(policy.is_allowed("https://example.com"));
        assert!(policy.is_allowed("https://google.com"));
    }

    #[test]
    fn test_block_all() {
        let policy = NetworkPolicy::block_all();
        assert!(!policy.is_allowed("https://example.com"));
    }

    #[test]
    fn test_whitelist() {
        let policy = NetworkPolicy::new(true, vec!["api.openai.com".into(), "github.com".into()]);
        assert!(policy.is_allowed("https://api.openai.com/v1/chat"));
        assert!(policy.is_allowed("https://github.com/user/repo"));
        assert!(!policy.is_allowed("https://evil.com"));
    }
}
