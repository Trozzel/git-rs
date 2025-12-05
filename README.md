# git-rs

A Rust implementation of Git - simpler than libgit2 but capable of handling most everyday Git usage.

## Overview

`git-rs` is a Rust-based reimplementation of Git's core functionality. While it's simpler than `libgit2`, it provides the essential features needed for everyday Git operations.

## Features

Currently implemented:
- **Repository initialization** (`init`)
- **Object hashing** (`hash-object`)
- **Object inspection** (`cat-file`)
- Core Git object types (blob, tree, commit, tag)
- Object storage with SHA-1 hashing and zlib compression

## Building

```bash
cargo build --release
```

## Usage

### Initialize a repository

```bash
git-rs init [path]
```

Creates a new Git repository in the specified directory (defaults to current directory).

### Hash an object

```bash
git-rs hash-object [-w] <file>
```

Computes the SHA-1 hash of a file. With `-w`, writes the object to the repository's object database.

Example:
```bash
printf "hello world" > test.txt
git-rs hash-object -w test.txt
# Output: 95d09f2b10159347eece71399a7e2e907ea3df4f
```

### Inspect an object

```bash
git-rs cat-file [-t|-s|-p] <object>
```

- `-t`: Show the object type
- `-s`: Show the object size
- `-p`: Pretty-print the object content

Examples:
```bash
git-rs cat-file -t 95d09f2b10159347eece71399a7e2e907ea3df4f
# Output: blob

git-rs cat-file -s 95d09f2b10159347eece71399a7e2e907ea3df4f
# Output: 11

git-rs cat-file -p 95d09f2b10159347eece71399a7e2e907ea3df4f
# Output: hello world
```

## Architecture

The project is organized into several modules:

- `objects`: Core Git object types and serialization
- `repository`: Repository management and object storage
- `commands`: High-level command implementations

## Testing

Run the test suite:

```bash
cargo test
```

## Future Work

Planned features:
- Tree and commit objects
- Index (staging area)
- Branch management
- Merge functionality
- Remote operations
- Diff and status commands

## License

This project is open source.
