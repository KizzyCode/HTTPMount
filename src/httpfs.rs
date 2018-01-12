use std;
use super::http_file::ErrorType as HttpErrorType;

use super::fuse;
use super::libc;
use super::time;


const TTL_1S: time::Timespec = time::Timespec{ sec: 1, nsec: 0 };
const TIME_0: time::Timespec = time::Timespec{ sec: 0, nsec: 0 };


pub struct HttpFS {
	http_file: super::http_file::File,
	timeout: std::time::Duration,
	
	attr_root: fuse::FileAttr,
	attr_file: fuse::FileAttr,
	buffer: Vec<u8>
}
impl HttpFS {
	pub fn new(http_file: super::http_file::File, timeout: std::time::Duration) -> Self {
		// Create the attributes of the root-directory
		let attr_root = fuse::FileAttr {
			ino: 1, size: 0, blocks: 0,
			atime: TIME_0, mtime: TIME_0, ctime: TIME_0, crtime: TIME_0,
			kind: fuse::FileType::Directory,
			perm: 0o755, nlink: 2, uid: 501, gid: 0,
			rdev: 0, flags: 0
		};
		
		// Create the attributes of the file
		let mut blocks = http_file.size() / 4096;
		if blocks % 4096 != 0 { blocks += 1 }
		
		let attr_file = fuse::FileAttr {
			ino: 2, size: http_file.size(), blocks,
			atime: TIME_0, mtime: TIME_0, ctime: TIME_0, crtime: TIME_0,
			kind: fuse::FileType::RegularFile,
			perm: 0o644, nlink: 1, uid: 501, gid: 0,
			rdev: 0, flags: 0
		};
		
		HttpFS{ http_file, timeout, attr_root, attr_file, buffer: Vec::new() }
	}
}
impl super::fuse::Filesystem for HttpFS {
	fn lookup(&mut self, _request: &fuse::Request, parent: u64, name: &std::ffi::OsStr, reply: fuse::ReplyEntry) {
		if parent == self.attr_root.ino && name.to_str() == Some(self.http_file.name()) { reply.entry(&TTL_1S, &self.attr_file, 0) }
			else { reply.error(libc::ENOENT) }
	}
	
	fn getattr(&mut self, _request: &fuse::Request, inode: u64, reply: fuse::ReplyAttr) {
		match inode {
			1 => reply.attr(&TTL_1S, &self.attr_root),
			2 => reply.attr(&TTL_1S, &self.attr_file),
			_ => reply.error(libc::ENOENT)
		}
	}
	
	fn read(&mut self, _req: &fuse::Request, inode: u64, _file_handle: u64, offset: i64, size: u32, reply: fuse::ReplyData) {
		if inode == self.attr_file.ino {
			// Reallocate buffer
			if self.buffer.len() < size as usize { self.buffer = vec![0u8; size as usize] }
			
			// Read data
			match self.http_file.read_at(&mut self.buffer[.. size as usize], offset as u64, self.timeout) {
				Ok(bytes_read) => reply.data(&self.buffer[..bytes_read]),
				Err(error) => {
					match error.error_type {
						HttpErrorType::IOAccessError => reply.error(libc::EACCES),
						HttpErrorType::IOReadWriteError => reply.error(libc::EIO),
						HttpErrorType::GenericIOError(error) => reply.error(error.raw_os_error().unwrap_or(libc::EIO)),
						_ => reply.error(libc::EIO)
					}
				}
			}
		} else {
			reply.error(libc::ENOENT)
		}
	}
	
	fn readdir(&mut self, _request: &fuse::Request, inode: u64, _file_handle: u64, offset: i64, mut reply: fuse::ReplyDirectory) {
		if inode == self.attr_root.ino {
			if offset == 0 {
				reply.add(self.attr_root.ino, 0, fuse::FileType::Directory, ".");
				reply.add(self.attr_root.ino, 1, fuse::FileType::Directory, "..");
				reply.add(self.attr_file.ino, 2, self.attr_file.kind, &self.http_file.name());
			}
			reply.ok()
		} else {
			reply.error(libc::ENOENT)
		}
	}
}