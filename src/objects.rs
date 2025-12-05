use anyhow::{Context, Result};
use sha1::{Digest, Sha1};
use std::fmt;
use std::io::{Read, Write};

/// Git object types
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl ObjectType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "blob" => Ok(ObjectType::Blob),
            "tree" => Ok(ObjectType::Tree),
            "commit" => Ok(ObjectType::Commit),
            "tag" => Ok(ObjectType::Tag),
            _ => Err(anyhow::anyhow!("Unknown object type: {}", s)),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ObjectType::Blob => "blob",
            ObjectType::Tree => "tree",
            ObjectType::Commit => "commit",
            ObjectType::Tag => "tag",
        }
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a Git object
#[derive(Debug, Clone)]
pub struct Object {
    pub obj_type: ObjectType,
    pub data: Vec<u8>,
}

impl Object {
    /// Create a new Git object
    pub fn new(obj_type: ObjectType, data: Vec<u8>) -> Self {
        Self { obj_type, data }
    }

    /// Serialize the object for storage (includes header)
    pub fn serialize(&self) -> Vec<u8> {
        let header = format!("{} {}\0", self.obj_type.as_str(), self.data.len());
        let mut result = header.into_bytes();
        result.extend_from_slice(&self.data);
        result
    }

    /// Deserialize an object from storage format
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        // Find the null byte that separates header from content
        let null_pos = data
            .iter()
            .position(|&b| b == 0)
            .context("Invalid object format: no null byte")?;

        let header = std::str::from_utf8(&data[..null_pos])
            .context("Invalid UTF-8 in object header")?;

        let parts: Vec<&str> = header.split(' ').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid object header format"));
        }

        let obj_type = ObjectType::from_str(parts[0])?;
        let size: usize = parts[1]
            .parse()
            .context("Invalid size in object header")?;

        let content = &data[null_pos + 1..];
        if content.len() != size {
            return Err(anyhow::anyhow!(
                "Object size mismatch: expected {}, got {}",
                size,
                content.len()
            ));
        }

        Ok(Self::new(obj_type, content.to_vec()))
    }

    /// Calculate the SHA-1 hash of this object
    pub fn hash(&self) -> String {
        let serialized = self.serialize();
        let mut hasher = Sha1::new();
        hasher.update(&serialized);
        hex::encode(hasher.finalize())
    }

    /// Compress the object data using zlib
    pub fn compress(&self) -> Result<Vec<u8>> {
        let serialized = self.serialize();
        let compression = flate2::Compression::default();
        let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), compression);
        encoder
            .write_all(&serialized)
            .context("Failed to compress object")?;
        encoder.finish().context("Failed to finish compression")
    }

    /// Decompress object data from zlib format
    pub fn decompress(compressed: &[u8]) -> Result<Self> {
        let mut decoder = flate2::read::ZlibDecoder::new(compressed);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .context("Failed to decompress object")?;
        Self::deserialize(&decompressed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_type_from_str() {
        assert_eq!(ObjectType::from_str("blob").unwrap(), ObjectType::Blob);
        assert_eq!(ObjectType::from_str("tree").unwrap(), ObjectType::Tree);
        assert_eq!(ObjectType::from_str("commit").unwrap(), ObjectType::Commit);
        assert_eq!(ObjectType::from_str("tag").unwrap(), ObjectType::Tag);
        assert!(ObjectType::from_str("invalid").is_err());
    }

    #[test]
    fn test_object_serialize_deserialize() {
        let data = b"hello world".to_vec();
        let obj = Object::new(ObjectType::Blob, data.clone());
        let serialized = obj.serialize();
        let deserialized = Object::deserialize(&serialized).unwrap();
        
        assert_eq!(deserialized.obj_type, ObjectType::Blob);
        assert_eq!(deserialized.data, data);
    }

    #[test]
    fn test_object_hash() {
        let data = b"hello world".to_vec();
        let obj = Object::new(ObjectType::Blob, data);
        let hash = obj.hash();
        
        // This is the actual SHA-1 hash for "blob 11\0hello world"
        assert_eq!(hash, "95d09f2b10159347eece71399a7e2e907ea3df4f");
    }

    #[test]
    fn test_object_compress_decompress() {
        let data = b"hello world".to_vec();
        let obj = Object::new(ObjectType::Blob, data.clone());
        let compressed = obj.compress().unwrap();
        let decompressed = Object::decompress(&compressed).unwrap();
        
        assert_eq!(decompressed.obj_type, ObjectType::Blob);
        assert_eq!(decompressed.data, data);
    }
}
