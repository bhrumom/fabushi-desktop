use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use uuid::Uuid;

const COMPARISON_BUFFER_BYTES: usize = 64 * 1024;

fn invalid_existing_file() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "Content-addressed attachment is not a regular file with the expected bytes.",
    )
}

/// Only absence permits a new publication. An invalid entry is never repaired.
fn verify_existing_file(path: &Path, bytes: &[u8]) -> io::Result<bool> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if !metadata.file_type().is_file() || metadata.len() != bytes.len() as u64 {
        return Err(invalid_existing_file());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x0400 != 0 {
            return Err(invalid_existing_file());
        }
    }

    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // A replaced leaf must not redirect this read or block it on a FIFO.
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // FILE_FLAG_OPEN_REPARSE_POINT opens the entry rather than its target.
        options.custom_flags(0x0020_0000);
    }
    let mut file = options.open(path)?;
    let opened = file.metadata()?;
    if !opened.file_type().is_file() || opened.len() != bytes.len() as u64 {
        return Err(invalid_existing_file());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.dev() != opened.dev() || metadata.ino() != opened.ino() {
            return Err(invalid_existing_file());
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        // Reject all reparse points, not only symbolic-link tags.
        if opened.file_attributes() & 0x0400 != 0 {
            return Err(invalid_existing_file());
        }
    }

    compare_file_contents(&mut file, bytes)?;
    Ok(true)
}

fn compare_file_contents(file: &mut File, bytes: &[u8]) -> io::Result<()> {
    let mut buffer = [0_u8; COMPARISON_BUFFER_BYTES];
    for expected in bytes.chunks(buffer.len()) {
        let actual = &mut buffer[..expected.len()];
        file.read_exact(actual)?;
        if actual != expected {
            return Err(invalid_existing_file());
        }
    }
    let mut trailing = [0_u8; 1];
    if file.read(&mut trailing)? != 0 {
        return Err(invalid_existing_file());
    }
    Ok(())
}

struct StagedContentFile {
    path: PathBuf,
    file: Option<File>,
    cleanup_needed: bool,
}

impl StagedContentFile {
    fn create(path: PathBuf) -> io::Result<Self> {
        // Preserve fs::write's default creation permissions. Own the path only
        // after create_new succeeds, so a collision cannot clean up foreign data.
        let file = OpenOptions::new().write(true).create_new(true).open(&path)?;
        Ok(Self { path, file: Some(file), cleanup_needed: true })
    }

    fn cleanup(&mut self) -> io::Result<()> {
        drop(self.file.take());
        if self.cleanup_needed {
            fs::remove_file(&self.path)?;
            self.cleanup_needed = false;
        }
        Ok(())
    }
}

impl Drop for StagedContentFile {
    fn drop(&mut self) {
        // Interruption or a cleanup failure may leave this temporary sibling.
        // There is deliberately no cleanup of final or other temporary paths.
        let _ = self.cleanup();
    }
}

fn stage_content_file(path: &Path, bytes: &[u8]) -> io::Result<StagedContentFile> {
    // A short independent name avoids extending a valid long resource filename
    // beyond the filesystem's component limit.
    let temporary = path.with_file_name(format!(".attachment-{}.tmp", Uuid::new_v4().simple()));
    let mut staged = StagedContentFile::create(temporary)?;
    let file = staged.file.as_mut().expect("staged attachment owns its file");
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(staged.file.take());
    Ok(staged)
}

fn publish_content_file(
    path: &Path,
    staged: &StagedContentFile,
    bytes: &[u8],
) -> io::Result<()> {
    match fs::hard_link(&staged.path, path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            if verify_existing_file(path, bytes)? {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Content-addressed attachment disappeared during publication.",
                ))
            }
        }
        // No replacing rename or direct-final-write fallback is safe here.
        Err(error) => Err(error),
    }
}

fn sync_parent_directory(_path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        let parent = _path.parent().filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

/// Publish within the existing owner's stable directory. This does not grant
/// parent-directory authority or protect later consumers from file replacement.
pub(super) fn write_content_addressed_file(
    dir: &Path,
    path: &Path,
    bytes: &[u8],
) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    if verify_existing_file(path, bytes)? {
        // Also retry a prior publication whose directory sync failed. Do not
        // rewrite or require a writable handle to an intact legacy object.
        return sync_parent_directory(path);
    }

    let mut staged = stage_content_file(path, bytes)?;
    let publication = publish_content_file(path, &staged, bytes)
        .and_then(|()| sync_parent_directory(path));
    // A failure here leaves the complete final object intact for a verified
    // retry. Preserve that original error if cleanup also fails; otherwise a
    // cleanup failure is an ordinary error, with Drop as best-effort fallback.
    let cleanup = staged.cleanup();
    publication.and(cleanup)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "fabushi-content-publication-{}", Uuid::new_v4().simple(),
            ));
            fs::create_dir_all(&path).expect("test directory");
            Self(path)
        }

        fn target(&self) -> PathBuf {
            self.0.join("object.bin")
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn staged_bytes_are_complete_before_the_final_name_is_published() {
        let directory = TestDirectory::new();
        let target = directory.target();
        let bytes = vec![0x5a; COMPARISON_BUFFER_BYTES + 17];
        let mut staged = stage_content_file(&target, &bytes).expect("stage and sync bytes");
        let temporary = staged.path.clone();

        assert_eq!(fs::read(&temporary).expect("staged bytes"), bytes);
        assert!(!target.exists(), "complete staging has not published a final name");
        assert!(staged.file.is_none(), "the staged writer is closed before publication");

        publish_content_file(&target, &staged, &bytes).expect("publish complete bytes");
        sync_parent_directory(&target).expect("sync publication");
        staged.cleanup().expect("remove owned stage");
        assert!(!temporary.exists());
        assert_eq!(fs::read(&target).expect("final bytes"), bytes);
    }

    #[test]
    fn concurrent_staged_publishers_converge_on_one_complete_file() {
        let directory = TestDirectory::new();
        let target = directory.target();
        let bytes = Arc::new(vec![0x6b; COMPARISON_BUFFER_BYTES * 2 + 31]);
        let ready = Arc::new(Barrier::new(4));
        let mut workers = Vec::new();
        for _ in 0..4 {
            let target = target.clone();
            let bytes = bytes.clone();
            let ready = ready.clone();
            workers.push(thread::spawn(move || -> io::Result<()> {
                let staged = stage_content_file(&target, &bytes);
                // Even a staging error reaches the barrier before it propagates.
                // Every successful candidate is complete before any publication.
                ready.wait();
                let mut staged = staged?;
                let publication = publish_content_file(&target, &staged, &bytes)
                    .and_then(|()| sync_parent_directory(&target));
                let cleanup = staged.cleanup();
                publication.and(cleanup)
            }));
        }
        let results: Vec<_> = workers.into_iter().map(|worker| worker.join()).collect();
        for result in results {
            result.expect("publisher thread").expect("complete publication");
        }
        assert_eq!(fs::read(&target).expect("single winner"), *bytes);
        assert_eq!(fs::read_dir(&directory.0).expect("remaining entries").count(), 1);
    }

    #[test]
    fn publication_rejects_a_corrupt_winner_created_after_staging() {
        let directory = TestDirectory::new();
        let target = directory.target();
        let bytes = b"complete expected bytes";
        let mut staged = stage_content_file(&target, bytes).expect("stage bytes");
        let temporary = staged.path.clone();
        let corrupt = vec![0x7c; bytes.len()];
        fs::write(&target, &corrupt).expect("seed competing corrupt winner");

        let error = publish_content_file(&target, &staged, bytes).expect_err("reject winner");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        staged.cleanup().expect("cleanup rejected candidate");
        assert!(!temporary.exists());
        assert_eq!(fs::read(&target).expect("retained winner"), corrupt);
    }

    #[test]
    fn failed_publication_cleans_only_its_owned_stage() {
        let directory = TestDirectory::new();
        let target = directory.target();
        let bytes = b"candidate bytes";
        let mut staged = stage_content_file(&target, bytes).expect("stage bytes");
        let temporary = staged.path.clone();
        let foreign = directory.0.join(".attachment-foreign.tmp");
        fs::write(&foreign, b"another operation").expect("foreign stage");
        fs::create_dir(&target).expect("directory winner");
        let marker = target.join("keep.txt");
        fs::write(&marker, b"existing directory content").expect("marker");

        assert!(publish_content_file(&target, &staged, bytes).is_err());
        staged.cleanup().expect("cleanup owned candidate");
        assert!(!temporary.exists());
        assert_eq!(fs::read(&foreign).expect("foreign stage retained"), b"another operation");
        assert_eq!(fs::read(&marker).expect("winner retained"), b"existing directory content");
    }

    #[test]
    fn create_new_collision_never_owns_or_removes_the_existing_path() {
        let directory = TestDirectory::new();
        let foreign = directory.0.join(".attachment-existing.tmp");
        fs::write(&foreign, b"not owned by this call").expect("foreign stage");
        let result = StagedContentFile::create(foreign.clone());
        assert!(matches!(&result, Err(error) if error.kind() == io::ErrorKind::AlreadyExists));
        drop(result);
        assert_eq!(fs::read(&foreign).expect("foreign stage retained"), b"not owned by this call");
    }

    #[test]
    fn retry_after_publication_before_acknowledgement_reuses_the_final_file() {
        let directory = TestDirectory::new();
        let target = directory.target();
        let bytes = b"complete publication awaiting acknowledgement";
        let staged = stage_content_file(&target, bytes).expect("stage bytes");
        let temporary = staged.path.clone();
        publish_content_file(&target, &staged, bytes).expect("publish before acknowledgement");
        let published_metadata = fs::metadata(&target).expect("published metadata");
        // Stop at the real link boundary, before the writer's directory sync and
        // caller acknowledgement. This is not a process-kill or power-loss test.
        drop(staged);
        assert!(!temporary.exists());

        write_content_addressed_file(&directory.0, &target, bytes).expect("verify and sync retry");
        assert_eq!(fs::read(&target).expect("retry winner"), bytes);
        let retried_metadata = fs::metadata(&target).expect("retry metadata");
        assert_eq!(published_metadata.modified().unwrap(), retried_metadata.modified().unwrap());
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_eq!(published_metadata.ino(), retried_metadata.ino());
        }
        assert_eq!(fs::read_dir(&directory.0).expect("remaining entries").count(), 1);
    }

    #[test]
    fn cleanup_failure_leaves_the_complete_final_file_retryable() {
        let directory = TestDirectory::new();
        let target = directory.target();
        let bytes = b"complete winner despite cleanup failure";
        let mut staged = stage_content_file(&target, bytes).expect("stage bytes");
        publish_content_file(&target, &staged, bytes).expect("publish bytes");
        // Deterministically fail the same cleanup operation used by the writer,
        // without depending on user privileges, permissions, or scheduling.
        fs::remove_file(&staged.path).expect("remove owned staging name");
        let error = staged.cleanup().expect_err("surface cleanup failure");
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        drop(staged);

        assert_eq!(fs::read(&target).expect("intact final object"), bytes);
        write_content_addressed_file(&directory.0, &target, bytes).expect("retry complete winner");
        assert_eq!(fs::read_dir(&directory.0).expect("remaining entries").count(), 1);
    }

    #[test]
    fn bounded_comparison_rejects_changes_after_the_first_buffer() {
        let directory = TestDirectory::new();
        let target = directory.target();
        let bytes = vec![0x4d; COMPARISON_BUFFER_BYTES * 2 + 13];
        fs::write(&target, &bytes).expect("complete legacy object");
        assert!(verify_existing_file(&target, &bytes).expect("verify all chunks"));

        let mut corrupt = bytes.clone();
        *corrupt.last_mut().expect("nonempty bytes") ^= 1;
        fs::write(&target, &corrupt).expect("seed corrupt final chunk");
        let error = verify_existing_file(&target, &bytes).expect_err("reject later difference");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(fs::read(&target).expect("corrupt object retained"), corrupt);

        fs::write(&target, &bytes).expect("restore exact comparison fixture");
        let mut opened = File::open(&target).expect("open exact-length object");
        assert_eq!(opened.metadata().unwrap().len(), bytes.len() as u64);
        OpenOptions::new().append(true).open(&target).unwrap()
            .write_all(b"extra byte after the length check").unwrap();
        let error = compare_file_contents(&mut opened, &bytes).expect_err("require EOF after expected bytes");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);

        fs::write(&target, &bytes).expect("restore truncation fixture");
        let mut opened = File::open(&target).expect("open pre-truncation object");
        assert_eq!(opened.metadata().unwrap().len(), bytes.len() as u64);
        OpenOptions::new().write(true).open(&target).unwrap()
            .set_len(bytes.len() as u64 - 1).unwrap();
        let error = compare_file_contents(&mut opened, &bytes).expect_err("require complete expected bytes");
        assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn dropping_an_unpublished_stage_does_not_create_a_final_object() {
        let directory = TestDirectory::new();
        let target = directory.target();
        let foreign = directory.0.join(".attachment-other.tmp");
        fs::write(&foreign, b"other stage").expect("foreign stage");
        let staged = stage_content_file(&target, b"unpublished bytes").expect("stage bytes");
        let temporary = staged.path.clone();
        drop(staged);

        assert!(!target.exists());
        assert!(!temporary.exists());
        assert_eq!(fs::read(&foreign).expect("foreign stage retained"), b"other stage");
    }
}
