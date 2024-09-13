use super::{
    git_commit_author::GitCommitAuthor,
    git_folders::{GitFolders, GIT_FOLDER},
    git_project::GitProject,
};
use crate::errors::git_object_error::GitObjectError;
use core::fmt;
use flate2::bufread::ZlibDecoder;
use serde::{Deserialize, Serialize};
use std::{io::Read, path::Path};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GitCommit {
    hash: String,
    tree_hash: String,
    parent_hashes: Vec<String>,
    author: GitCommitAuthor,
    committer: GitCommitAuthor,
    message: String,
}

impl GitCommit {
    pub fn new(
        hash: &str,
        tree_hash: &str,
        parent_hashes: &[String],
        author: GitCommitAuthor,
        committer: GitCommitAuthor,
        message: &str,
    ) -> GitCommit {
        GitCommit {
            hash: hash.to_string(),
            tree_hash: tree_hash.to_string(),
            parent_hashes: parent_hashes.to_vec(),
            author,
            committer,
            message: message.to_string(),
        }
    }

    pub fn from_hash(project: &GitProject, commit_hash: &str) -> Result<GitCommit, GitObjectError> {
        let objects_folder_path = Path::new(project.get_directory())
            .join(GIT_FOLDER)
            .join(GitFolders::OBJECTS.to_string());

        let commit_folder = &commit_hash[..2];
        let commit_file = &commit_hash[2..];
        let commit_file = objects_folder_path.join(commit_folder).join(commit_file);

        let buf = std::fs::read(commit_file).map_err(|_| GitObjectError::FileReadError)?;
        GitCommit::from_encoded_data(commit_hash.to_string(), &buf)
    }

    pub fn from_encoded_data(
        commit_hash: String,
        encoded_file_content: &[u8],
    ) -> Result<GitCommit, GitObjectError> {
        let mut zlib = ZlibDecoder::new(encoded_file_content);
        let mut decoded_file_content = String::new();

        zlib.read_to_string(&mut decoded_file_content)
            .map_err(|_| GitObjectError::DecompressionError)?;

        let mut lines = decoded_file_content.lines();

        let tree_line = lines.next().ok_or(GitObjectError::InvalidCommitFile)?;
        let tree_line = tree_line
            .split("\0")
            .nth(1)
            .ok_or(GitObjectError::InvalidCommitFile)?;
        let tree_hash = tree_line
            .strip_prefix("tree ")
            .ok_or(GitObjectError::InvalidCommitFile)?;

        let parent_hashes = lines
            .clone()
            .take_while(|line| line.starts_with("parent "))
            .map(|line| line.strip_prefix("parent ").unwrap().to_string())
            .collect::<Vec<String>>();

        let mut lines = lines.skip_while(|line| line.starts_with("parent "));
        let author_line = lines.next().ok_or(GitObjectError::InvalidCommitFile)?;
        let author = GitCommitAuthor::from_string(author_line)?;

        let committer_line = lines.next().ok_or(GitObjectError::InvalidCommitFile)?;
        let committer = GitCommitAuthor::from_string(committer_line)?;

        lines.next(); // skip empty line
        let message = lines.collect::<Vec<&str>>().join("\n");

        Ok(GitCommit::new(
            commit_hash.as_str(),
            tree_hash,
            parent_hashes.as_slice(),
            author,
            committer,
            message.as_str(),
        ))
    }

    pub fn get_hash(&self) -> &String {
        &self.hash
    }

    pub fn get_tree_hash(&self) -> &String {
        &self.tree_hash
    }

    pub fn get_parent_hashes(&self) -> &Vec<String> {
        &self.parent_hashes
    }

    pub fn get_author(&self) -> &GitCommitAuthor {
        &self.author
    }

    pub fn get_committer(&self) -> &GitCommitAuthor {
        &self.committer
    }

    pub fn get_message(&self) -> &String {
        &self.message
    }

    pub fn get_parent_commits(
        &self,
        project: &GitProject,
    ) -> Result<Vec<GitCommit>, GitObjectError> {
        self.parent_hashes
            .iter()
            .map(|parent_hash| GitCommit::from_hash(project, parent_hash))
            .collect()
    }
}

impl fmt::Display for GitCommit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parent_hashes = self
            .parent_hashes
            .iter()
            .map(|parent_hash| format!("parent {}\n", parent_hash))
            .collect::<Vec<String>>()
            .join("");

        let content = format!(
            "tree {}\n{}{}\n{}\n\n{}",
            self.tree_hash,
            parent_hashes,
            self.author.to_string(true),
            self.committer.to_string(false),
            self.message
        );

        write!(f, "commit {}\0{}", content.len(), content)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    use crate::git::git_user::GitUser;
    use flate2::write::ZlibEncoder;

    fn mock_git_commit_author() -> GitCommitAuthor {
        GitCommitAuthor::new(
            GitUser {
                name: "Test User".to_string(),
                email: "test@example.com".to_string(),
            },
            1234567890,
            "+0000".to_string(),
        )
    }

    fn mock_git_commit() -> GitCommit {
        let author = mock_git_commit_author();
        GitCommit::new(
            "hash",
            "tree_hash",
            &vec!["parent_hash1".to_string(), "parent_hash2".to_string()],
            author.clone(),
            author.clone(),
            "commit message",
        )
    }

    fn create_encoded_commit_file(
        author: GitCommitAuthor,
        commiter: GitCommitAuthor,
        tree: Option<&str>,
        parent_commits: Vec<&str>,
        message: &str,
    ) -> Result<Vec<u8>, GitObjectError> {
        let tree_line = match tree {
            Some(tree) => format!("tree {}\n", tree),
            None => "".to_string(),
        };
        let parent_lines = parent_commits
            .iter()
            .map(|parent_commit| format!("parent {}\n", parent_commit))
            .collect::<Vec<String>>()
            .join("");
        let author_line = format!(
            "author {} <{}> {} {}\n",
            author.get_user().name,
            author.get_user().email,
            author.date_seconds,
            author.timezone
        );
        let commiter_line = format!(
            "commiter {} <{}> {} {}\n",
            commiter.get_user().name,
            commiter.get_user().email,
            commiter.date_seconds,
            commiter.timezone
        );

        let file_content = format!(
            "{}{}{}{}\n{}",
            tree_line, parent_lines, author_line, commiter_line, message
        );
        let file_content_to_encode = format!("commit {}\0{}", file_content.len(), file_content);

        let mut zlib = flate2::bufread::ZlibEncoder::new(
            file_content_to_encode.as_bytes(),
            flate2::Compression::default(),
        );
        let mut encoded_file_content = Vec::new();
        zlib.read_to_end(&mut encoded_file_content)
            .map_err(|_| GitObjectError::DecompressionError)?;

        Ok(encoded_file_content)
    }

    #[test]
    fn test_from_string() {
        let commiter = mock_git_commit_author();

        let commit_hash = "ae575432e84a11c11b8dc3e357806f65c50f4619".to_string();
        let encoded_file_content = create_encoded_commit_file(
            commiter.clone(),
            commiter.clone(),
            Some("50c8353444afbef3172c999ef6cff8d31309ac3e"),
            Vec::new(),
            "test commit",
        )
        .unwrap();

        let git_commit =
            GitCommit::from_encoded_data(commit_hash.clone(), &encoded_file_content).unwrap();
        assert_eq!(*git_commit.get_hash(), commit_hash);
        assert_eq!(*git_commit.get_parent_hashes(), Vec::<String>::new());
        assert_eq!(
            git_commit.get_tree_hash(),
            "50c8353444afbef3172c999ef6cff8d31309ac3e"
        );
        assert_eq!(git_commit.get_message(), "test commit");
        assert_eq!(*git_commit.get_author(), commiter);
        assert_eq!(*git_commit.get_committer(), commiter);
    }

    #[test]
    fn test_from_string_invalid() {
        let commit_hash = "50c8353444afbef3172c999ef6cff8d31309ac3e";
        let encoded_file_content = "invalid content".as_bytes();

        let git_commit = GitCommit::from_encoded_data(commit_hash.to_string(), encoded_file_content);
        assert!(git_commit.is_err());
    }

    #[test]
    fn test_to_string_no_parent() {
        let commiter = mock_git_commit_author();

        let commit_hash = "ae575432e84a11c11b8dc3e357806f65c50f4619".to_string();
        let encoded_file_content = create_encoded_commit_file(
            commiter.clone(),
            commiter.clone(),
            Some("50c8353444afbef3172c999ef6cff8d31309ac3e"),
            Vec::new(),
            "test commit",
        );

        let git_commit =
            GitCommit::from_encoded_data(commit_hash.clone(), encoded_file_content.as_ref().unwrap())
                .unwrap();

        let mut zlib = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        zlib.write_all(git_commit.to_string().as_bytes()).unwrap();

        let encoded_git_commit = zlib.finish().unwrap();
        assert_eq!(encoded_git_commit, encoded_file_content.unwrap());
    }

    #[test]
    fn test_to_string_with_parents() {
        let commiter = mock_git_commit_author();

        let commit_hash = "ae575432e84a11c11b8dc3e357806f65c50f4619".to_string();
        let parent_commit_hash = Vec::from([
            "50c8353444afbef3172c999ef6cff8d31309ac3e",
            "50c8353444afbef3172c999ef6cff8d31309ac33",
        ]);
        let encoded_file_content = create_encoded_commit_file(
            commiter.clone(),
            commiter.clone(),
            Some("50c8353444afbef3172c999ef6cff8d31309ac3e"),
            parent_commit_hash.clone(),
            "test commit",
        );

        let git_commit =
            GitCommit::from_encoded_data(commit_hash.clone(), encoded_file_content.as_ref().unwrap())
                .unwrap();

        let mut zlib = ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        zlib.write_all(git_commit.to_string().as_bytes()).unwrap();

        let encoded_git_commit = zlib.finish().unwrap();
        assert_eq!(encoded_git_commit, encoded_file_content.unwrap());
    }

    #[test]
    fn test_from_string_with_parents() {
        let commiter = mock_git_commit_author();

        let commit_hash = "ae575432e84a11c11b8dc3e357806f65c50f4619".to_string();
        let parent_commit_hash = Vec::from([
            "50c8353444afbef3172c999ef6cff8d31309ac3e",
            "50c8353444afbef3172c999ef6cff8d31309ac33",
        ]);
        let encoded_file_content = create_encoded_commit_file(
            commiter.clone(),
            commiter.clone(),
            Some(&commit_hash),
            parent_commit_hash.clone(),
            "test commit",
        );

        let git_commit =
            GitCommit::from_encoded_data(commit_hash.clone(), encoded_file_content.as_ref().unwrap())
                .unwrap();
        assert_eq!(git_commit.hash, commit_hash);
        assert_eq!(git_commit.parent_hashes, parent_commit_hash);
        assert_eq!(git_commit.tree_hash, commit_hash);
        assert_eq!(git_commit.message, "test commit");
        assert_eq!(git_commit.author, commiter);
    }

    #[test]
    fn test_serialize_git_commit() {
        let git_commit = mock_git_commit();
        let serialized = serde_json::to_string(&git_commit).unwrap();
        let expected = r#"{"hash":"hash","tree_hash":"tree_hash","parent_hashes":["parent_hash1","parent_hash2"],"author":{"user":{"name":"Test User","email":"test@example.com"},"date_seconds":1234567890,"timezone":"+0000"},"committer":{"user":{"name":"Test User","email":"test@example.com"},"date_seconds":1234567890,"timezone":"+0000"},"message":"commit message"}"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_deserialize_git_commit() {
        let json_str = r#"{"hash":"hash","tree_hash":"tree_hash","parent_hashes":["parent_hash1","parent_hash2"],"author":{"user":{"name":"Test User","email":"test@example.com"},"date_seconds":1234567890,"timezone":"+0000"},"committer":{"user":{"name":"Test User","email":"test@example.com"},"date_seconds":1234567890,"timezone":"+0000"},"message":"commit message"}"#;
        let deserialized: GitCommit = serde_json::from_str(json_str).unwrap();
        let expected = mock_git_commit();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn test_deserialize_invalid_json() {
        let invalid_json_str = r#"{"hash":"hash","tree_hash":"tree_hash","parent_hashes":["parent_hash1","parent_hash2"],"author":{"user":{"name":"Test User","email":"test@example.com"},"date_seconds":1234567890,"timezone":"+0000"},"committer":{"user":{"name":"Test User","email":"test@example.com"},"date_seconds":1234567890,"timezone":"+0000"},"message":"commit message""#; // Missing closing brace
        let result: Result<GitCommit, serde_json::Error> = serde_json::from_str(invalid_json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_new() {
        let name = "name".to_string();
        let email = "email".to_string();
        let git_user = GitUser::new(name.clone(), email.clone());
        let timezone = "timezone".to_string();
        let git_commit_author = GitCommitAuthor::new(git_user, 1, timezone.clone());
        let hash = "hash";
        let parent_hash = Vec::new();
        let message = "message";
        let git_commit = GitCommit::new(
            hash,
            hash,
            parent_hash.as_slice(),
            git_commit_author.clone(),
            git_commit_author.clone(),
            message,
        );
        assert_eq!(git_commit.hash, hash);
        assert_eq!(git_commit.tree_hash, hash);
        assert_eq!(git_commit.parent_hashes, parent_hash);
        assert_eq!(git_commit.author.get_user().name, name);
        assert_eq!(git_commit.author.get_user().email, email);
        assert_eq!(git_commit.committer.get_user().name, name);
        assert_eq!(git_commit.committer.get_user().email, email);
        assert_eq!(git_commit.message, message);
    }
}
