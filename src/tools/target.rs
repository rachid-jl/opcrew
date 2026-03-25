use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum KnownHostsPolicy {
    #[default]
    Strict,
    TrustOnFirstUse,
    Disabled,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum TargetHost {
    #[default]
    Local,
    Remote {
        host: String,
        user: String,
        key_path: Option<String>,
        #[serde(default)]
        known_hosts_policy: KnownHostsPolicy,
        #[serde(default = "default_connect_timeout")]
        connect_timeout_secs: u64,
    },
}

fn default_connect_timeout() -> u64 {
    10
}

impl TargetHost {
    pub fn parse_target(target: &str) -> Option<Self> {
        if let Some((user, host)) = target.split_once('@') {
            Some(Self::Remote {
                host: host.to_string(),
                user: user.to_string(),
                key_path: None,
                known_hosts_policy: KnownHostsPolicy::default(),
                connect_timeout_secs: default_connect_timeout(),
            })
        } else {
            None
        }
    }

    pub fn ssh_args(&self) -> Vec<String> {
        match self {
            Self::Local => vec![],
            Self::Remote {
                host,
                user,
                key_path,
                known_hosts_policy,
                connect_timeout_secs,
            } => {
                let mut args = vec![
                    "ssh".to_string(),
                    "-o".to_string(),
                    format!("ConnectTimeout={connect_timeout_secs}"),
                ];

                match known_hosts_policy {
                    KnownHostsPolicy::Strict => {
                        args.push("-o".to_string());
                        args.push("StrictHostKeyChecking=yes".to_string());
                    }
                    KnownHostsPolicy::TrustOnFirstUse => {
                        args.push("-o".to_string());
                        args.push("StrictHostKeyChecking=accept-new".to_string());
                    }
                    KnownHostsPolicy::Disabled => {
                        args.push("-o".to_string());
                        args.push("StrictHostKeyChecking=no".to_string());
                        args.push("-o".to_string());
                        args.push("UserKnownHostsFile=/dev/null".to_string());
                    }
                }

                if let Some(key) = key_path {
                    args.push("-i".to_string());
                    args.push(key.clone());
                }

                args.push(format!("{user}@{host}"));
                args
            }
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_target_string() {
        let target = TargetHost::parse_target("root@192.168.1.1").unwrap();
        match target {
            TargetHost::Remote { host, user, .. } => {
                assert_eq!(host, "192.168.1.1");
                assert_eq!(user, "root");
            }
            _ => panic!("Expected Remote"),
        }
    }

    #[test]
    fn parse_invalid_target() {
        assert!(TargetHost::parse_target("just-a-host").is_none());
    }

    #[test]
    fn ssh_args_strict() {
        let target = TargetHost::Remote {
            host: "server.com".into(),
            user: "admin".into(),
            key_path: Some("/home/user/.ssh/id_rsa".into()),
            known_hosts_policy: KnownHostsPolicy::Strict,
            connect_timeout_secs: 10,
        };
        let args = target.ssh_args();
        assert!(args.contains(&"ConnectTimeout=10".to_string()));
        assert!(args.contains(&"StrictHostKeyChecking=yes".to_string()));
        assert!(args.contains(&"admin@server.com".to_string()));
    }
}
