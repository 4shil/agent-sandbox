use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: Option<u64>,
    pub max_cpu_seconds: Option<u64>,
    pub max_timeout_seconds: Option<u64>,
    pub max_disk_bytes: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(2 * 1024 * 1024 * 1024),  // 2GB
            max_cpu_seconds: Some(600),                        // 10 min
            max_timeout_seconds: Some(1800),                   // 30 min
            max_disk_bytes: Some(10 * 1024 * 1024 * 1024),    // 10GB
        }
    }
}

impl ResourceLimits {
    pub fn from_args(memory: Option<&str>, cpu: Option<&str>, timeout: Option<&str>, disk: Option<&str>) -> Result<Self> {
        Ok(Self {
            max_memory_bytes: memory.map(parse_bytes).transpose()?,
            max_cpu_seconds: cpu.map(parse_seconds).transpose()?,
            max_timeout_seconds: timeout.map(parse_seconds).transpose()?,
            max_disk_bytes: disk.map(parse_bytes).transpose()?,
        })
    }

    pub fn apply_to_pid(&self, pid: u32) -> Result<()> {
        // Apply cgroup limits to a process
        let cgroup_path = format!("/sys/fs/cgroup/agent-sandbox-{}", pid);
        std::fs::create_dir_all(&cgroup_path)?;

        if let Some(mem_limit) = self.max_memory_bytes {
            std::fs::write(format!("{}/memory.max", cgroup_path), mem_limit.to_string())?;
        }

        if let Some(cpu_limit) = self.max_cpu_seconds {
            // cgroups v2: cpu.max format is "$quota $period"
            std::fs::write(format!("{}/cpu.max", cgroup_path), format!("{} 100000", cpu_limit * 100000))?;
        }

        // Add the process to the cgroup
        std::fs::write(format!("{}/cgroup.procs", cgroup_path), pid.to_string())?;

        Ok(())
    }

    pub fn check_violations(&self, pid: u32) -> Result<Vec<String>> {
        let mut violations = Vec::new();
        let cgroup_path = format!("/sys/fs/cgroup/agent-sandbox-{}", pid);

        if let (Some(max_mem), Ok(mem_usage)) = (
            self.max_memory_bytes,
            std::fs::read_to_string(format!("{}/memory.current", cgroup_path))
                .and_then(|s| s.trim().parse::<u64>().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
        ) {
            if mem_usage > max_mem {
                violations.push(format!("Memory limit exceeded: {} > {}", format_bytes(mem_usage), format_bytes(max_mem)));
            }
        }

        Ok(violations)
    }

    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        if let Some(mem) = self.max_memory_bytes {
            parts.push(format!("MEM: {}", format_bytes(mem)));
        }
        if let Some(cpu) = self.max_cpu_seconds {
            parts.push(format!("CPU: {}s", cpu));
        }
        if let Some(timeout) = self.max_timeout_seconds {
            parts.push(format!("TIME: {}s", timeout));
        }
        parts.join(", ")
    }
}

fn parse_bytes(s: &str) -> Result<u64> {
    let s = s.trim().to_lowercase();
    if let Some(num) = s.strip_suffix("gb") {
        Ok(num.trim().parse::<u64>()? * 1024 * 1024 * 1024)
    } else if let Some(num) = s.strip_suffix("mb") {
        Ok(num.trim().parse::<u64>()? * 1024 * 1024)
    } else if let Some(num) = s.strip_suffix("kb") {
        Ok(num.trim().parse::<u64>()? * 1024)
    } else {
        Ok(s.parse::<u64>()?)
    }
}

fn parse_seconds(s: &str) -> Result<u64> {
    let s = s.trim().to_lowercase();
    if let Some(num) = s.strip_suffix("m") {
        Ok(num.trim().parse::<u64>()? * 60)
    } else if let Some(num) = s.strip_suffix("h") {
        Ok(num.trim().parse::<u64>()? * 3600)
    } else {
        Ok(s.parse::<u64>()?)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bytes() {
        assert_eq!(parse_bytes("2gb").unwrap(), 2 * 1024 * 1024 * 1024);
        assert_eq!(parse_bytes("512mb").unwrap(), 512 * 1024 * 1024);
        assert_eq!(parse_bytes("1024").unwrap(), 1024);
    }

    #[test]
    fn test_parse_seconds() {
        assert_eq!(parse_seconds("30m").unwrap(), 1800);
        assert_eq!(parse_seconds("2h").unwrap(), 7200);
        assert_eq!(parse_seconds("600").unwrap(), 600);
    }
}
