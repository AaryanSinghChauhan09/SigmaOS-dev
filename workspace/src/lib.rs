#![forbid(unsafe_code)]
//! SigmaOS-workspace — Workspace management for development environments.

/// Simple workspace management for SigmaOS development.
pub struct Workspace {
    pub name: String,
    pub path: std::path::PathBuf,
}

impl Workspace {
    pub fn new(name: impl Into<String>, path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_creation() {
        let ws = Workspace::new("test-workspace", "/tmp/test");
        assert_eq!(ws.name(), "test-workspace");
        assert_eq!(ws.path(), std::path::Path::new("/tmp/test"));
    }
}
