use crate::objects::{Object, ObjectType};
use crate::repository::Repository;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Initialize a new Git repository
pub fn init(path: &Path) -> Result<()> {
    Repository::init(path)?;
    Ok(())
}

/// Hash a file and optionally write it to the repository
pub fn hash_object(path: &Path, write: bool, repo_path: Option<&Path>) -> Result<String> {
    let data = fs::read(path).context("Failed to read file")?;
    let object = Object::new(ObjectType::Blob, data);
    let hash = object.hash();

    if write {
        let repo_path = repo_path.unwrap_or_else(|| Path::new("."));
        let repo = Repository::new(repo_path)?;
        repo.write_object(&object)?;
    }

    Ok(hash)
}

/// Print the contents of an object
pub fn cat_file(hash: &str, repo_path: Option<&Path>) -> Result<()> {
    let repo_path = repo_path.unwrap_or_else(|| Path::new("."));
    let repo = Repository::new(repo_path)?;
    let object = repo.read_object(hash)?;

    // All object types are printed the same way
    print!("{}", String::from_utf8_lossy(&object.data));

    Ok(())
}

/// Show the type of an object
pub fn cat_file_type(hash: &str, repo_path: Option<&Path>) -> Result<String> {
    let repo_path = repo_path.unwrap_or_else(|| Path::new("."));
    let repo = Repository::new(repo_path)?;
    let object = repo.read_object(hash)?;
    Ok(object.obj_type.to_string())
}

/// Show the size of an object
pub fn cat_file_size(hash: &str, repo_path: Option<&Path>) -> Result<usize> {
    let repo_path = repo_path.unwrap_or_else(|| Path::new("."));
    let repo = Repository::new(repo_path)?;
    let object = repo.read_object(hash)?;
    Ok(object.data.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_init() {
        let temp_dir = TempDir::new().unwrap();
        init(temp_dir.path()).unwrap();
        assert!(temp_dir.path().join(".git").exists());
    }

    #[test]
    fn test_hash_object() {
        let temp_dir = TempDir::new().unwrap();
        init(temp_dir.path()).unwrap();

        let test_file = temp_dir.path().join("test.txt");
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"hello world").unwrap();

        let hash = hash_object(&test_file, true, Some(temp_dir.path())).unwrap();
        assert_eq!(hash, "95d09f2b10159347eece71399a7e2e907ea3df4f");

        // Verify object was written
        let repo = Repository::new(temp_dir.path()).unwrap();
        assert!(repo.object_exists(&hash));
    }

    #[test]
    fn test_cat_file_type() {
        let temp_dir = TempDir::new().unwrap();
        init(temp_dir.path()).unwrap();

        let test_file = temp_dir.path().join("test.txt");
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"test content").unwrap();

        let hash = hash_object(&test_file, true, Some(temp_dir.path())).unwrap();
        let obj_type = cat_file_type(&hash, Some(temp_dir.path())).unwrap();
        assert_eq!(obj_type, "blob");
    }

    #[test]
    fn test_cat_file_size() {
        let temp_dir = TempDir::new().unwrap();
        init(temp_dir.path()).unwrap();

        let test_file = temp_dir.path().join("test.txt");
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"test content").unwrap();

        let hash = hash_object(&test_file, true, Some(temp_dir.path())).unwrap();
        let size = cat_file_size(&hash, Some(temp_dir.path())).unwrap();
        assert_eq!(size, 12); // "test content" is 12 bytes
    }
}
