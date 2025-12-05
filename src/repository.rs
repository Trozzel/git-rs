use crate::objects::Object;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a Git repository
pub struct Repository {
    pub worktree: PathBuf,
    pub gitdir: PathBuf,
}

impl Repository {
    /// Create a new Repository instance
    pub fn new(path: &Path) -> Result<Self> {
        let worktree = path.to_path_buf();
        let gitdir = worktree.join(".git");

        if !gitdir.exists() {
            return Err(anyhow::anyhow!(
                "Not a git repository (or any of the parent directories): .git"
            ));
        }

        Ok(Self { worktree, gitdir })
    }

    /// Initialize a new Git repository
    pub fn init(path: &Path) -> Result<Self> {
        let worktree = path.to_path_buf();
        let gitdir = worktree.join(".git");

        if gitdir.exists() {
            return Err(anyhow::anyhow!("Repository already exists"));
        }

        // Create directory structure
        fs::create_dir_all(&gitdir).context("Failed to create .git directory")?;
        fs::create_dir_all(gitdir.join("objects"))
            .context("Failed to create objects directory")?;
        fs::create_dir_all(gitdir.join("refs/heads"))
            .context("Failed to create refs/heads directory")?;
        fs::create_dir_all(gitdir.join("refs/tags"))
            .context("Failed to create refs/tags directory")?;

        // Create HEAD file
        fs::write(gitdir.join("HEAD"), "ref: refs/heads/main\n")
            .context("Failed to create HEAD file")?;

        // Create description file
        fs::write(
            gitdir.join("description"),
            "Unnamed repository; edit this file 'description' to name the repository.\n",
        )
        .context("Failed to create description file")?;

        // Create config file
        let config = "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n";
        fs::write(gitdir.join("config"), config).context("Failed to create config file")?;

        println!("Initialized empty Git repository in {}", gitdir.display());

        Ok(Self { worktree, gitdir })
    }

    /// Get the path for an object file
    fn object_path(&self, hash: &str) -> Result<PathBuf> {
        if hash.len() < 2 {
            return Err(anyhow::anyhow!("Invalid hash: too short"));
        }
        let (dir, file) = hash.split_at(2);
        Ok(self.gitdir.join("objects").join(dir).join(file))
    }

    /// Write an object to the repository
    pub fn write_object(&self, object: &Object) -> Result<String> {
        let hash = object.hash();
        let path = self.object_path(&hash)?;

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create object directory")?;
        }

        // Don't overwrite if object already exists
        if !path.exists() {
            let compressed = object.compress()?;
            fs::write(&path, compressed).context("Failed to write object file")?;
        }

        Ok(hash)
    }

    /// Read an object from the repository
    pub fn read_object(&self, hash: &str) -> Result<Object> {
        let path = self.object_path(hash)?;

        if !path.exists() {
            return Err(anyhow::anyhow!("Object {} not found", hash));
        }

        let compressed = fs::read(&path).context("Failed to read object file")?;
        Object::decompress(&compressed)
    }

    /// Check if an object exists in the repository
    pub fn object_exists(&self, hash: &str) -> bool {
        self.object_path(hash).map(|p| p.exists()).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::ObjectType;
    use tempfile::TempDir;

    #[test]
    fn test_repository_init() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        assert!(repo.gitdir.exists());
        assert!(repo.gitdir.join("objects").exists());
        assert!(repo.gitdir.join("refs/heads").exists());
        assert!(repo.gitdir.join("refs/tags").exists());
        assert!(repo.gitdir.join("HEAD").exists());
        assert!(repo.gitdir.join("config").exists());
    }

    #[test]
    fn test_write_and_read_object() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        let data = b"test content".to_vec();
        let object = Object::new(ObjectType::Blob, data.clone());
        let hash = repo.write_object(&object).unwrap();

        let read_object = repo.read_object(&hash).unwrap();
        assert_eq!(read_object.obj_type, ObjectType::Blob);
        assert_eq!(read_object.data, data);
    }

    #[test]
    fn test_object_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        let data = b"test content".to_vec();
        let object = Object::new(ObjectType::Blob, data);
        let hash = repo.write_object(&object).unwrap();

        assert!(repo.object_exists(&hash));
        assert!(!repo.object_exists("0000000000000000000000000000000000000000"));
    }
}
