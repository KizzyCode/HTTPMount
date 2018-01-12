extern crate http_file;
extern crate fuse;
extern crate libc;
extern crate time;

mod httpfs;

const OK_MESSAGE: &str = "✓";

#[derive(Debug)]
struct CLI {
	http_mount: String,
	
	uri: String,
	mountpoint: String,
	
	block_size: usize,
	cache_size: usize,
	timeout: u64,
	
	no_fork: bool,
	
	licenses: bool,
	help: bool
}
impl CLI {
	fn parse() -> Self {
		let (mut cli, args) = (CLI::default(), std::env::args().fold(Vec::new(), |mut args, string| { args.push(string); args }));
		
		// Set own-path
		cli.http_mount = args[0].clone();
		
		// Parse URI
		cli.uri = CLI::parse_key_value("--uri=", &args, &|value: Option<&str>| {
			if let Some(uri) = value { uri.to_owned() }
				else { help("Error: You need to specify a URI") }
		});
		
		// Parse mountpoint
		cli.mountpoint = CLI::parse_key_value("--mountpoint=", &args, &|value: Option<&str>| {
			if let Some(mountpoint) = value { mountpoint.to_owned() }
				else { help("Error: You need to specify a mountpoint") }
		});
		
		// Parse block-size
		if let Some(block_size) = CLI::parse_key_value("--block-size=", &args, &|value: Option<&str>| value.and_then(|x| Some(x.to_owned()))) {
			if let Ok(block_size) = block_size.parse::<usize>() { cli.block_size = block_size }
				else { help(&format!("Error: Invalid block-size \"{}\"", block_size)) }
		};
		
		// Parse cache-size
		if let Some(cache_size) = CLI::parse_key_value("--cache-size=", &args, &|value: Option<&str>| value.and_then(|x| Some(x.to_owned()))) {
			if let Ok(cache_size) = cache_size.parse::<usize>() { cli.cache_size = cache_size }
				else { help(&format!("Error: Invalid cache-size \"{}\"", cache_size)) }
		};
		
		// Parse timeout
		if let Some(timeout) = CLI::parse_key_value("--timeout=", &args, &|value: Option<&str>| value.and_then(|x| Some(x.to_owned()))) {
			if let Ok(timeout) = timeout.parse::<u64>() { cli.timeout = timeout }
				else { help(&format!("Error: Invalid timeout \"{}\"", timeout)) }
		};
		
		// Parse no-fork
		cli.no_fork = CLI::parse_key_value("--no-fork", &args, &|value: Option<&str>| value.is_some());
		
		// Parse copyright
		cli.licenses = CLI::parse_key_value("--licenses", &args, &|value: Option<&str>| value.is_some());
		
		// Parse help
		cli.help = CLI::parse_key_value("--help", &args, &|value: Option<&str>| value.is_some());
		
		cli
	}
	
	fn parse_key_value<T>(key: &str, args: &[String], parser: &Fn(Option<&str>) -> T) -> T {
		for arg in args {
			if arg.starts_with(key) { return parser(Some(&arg.as_str()[key.len() ..])) }
		}
		parser(None)
	}
}
impl Default for CLI {
	fn default() -> Self {
		CLI {
			http_mount: String::new(),
			uri: String::new(), mountpoint: String::new(),
			block_size: 1 * 1024 * 1024, cache_size: 256 * 1024 * 1024, timeout: 60,
			no_fork: false,
			licenses: false, help: false
		}
	}
}



fn die(message: &str, code: i32) -> ! {
	print!("{}", message.trim());
	std::process::exit(code);
}



// Run http-mount in background
fn fork(cli: &CLI) -> ! {
	// Spawn child and set argument
	let (mut http_mount, mut args) = (std::process::Command::new(&cli.http_mount), Vec::new());
	
	args.push(format!("--uri={}", cli.uri));
	args.push(format!("--mountpoint={}", cli.mountpoint));
	args.push(format!("--block-size={}", cli.block_size));
	args.push(format!("--cache-size={}", cli.cache_size));
	args.push(format!("--timeout={}", cli.timeout));
	args.push(format!("--no-fork"));
	
	http_mount.args(&args);
	
	// Wait for state
	http_mount.stderr(std::process::Stdio::piped());
	
	let child_handle = http_mount.spawn().expect("Failed to fork process");
	let mut stderr = child_handle.stderr.expect("Child has no stderr");
	
	// Read line from stderr
	use std::io::Read;
	let mut stderr_vec = vec![0; OK_MESSAGE.len()];
	let _ = stderr.read_exact(&mut stderr_vec);
	
	// Check if OK
	if stderr_vec.as_slice() == OK_MESSAGE.as_bytes() { std::process::exit(0) }
	
	// Read the rest, print error and exit with 1
	let _ = stderr.read_to_end(&mut stderr_vec);
	println!("{}", String::from_utf8_lossy(&stderr_vec));
	std::process::exit(1)
}

// Mount HTTP-FS
fn fuse(cli: &CLI) {
	// Open HTTP-file
	let mut http_file = match http_file::File::open(&cli.uri, std::time::Duration::from_secs(cli.timeout)) {
		Ok(http_file) => http_file,
		Err(error) => die(&format!("Error {:?}: {:?}", error.error_type, error.description), 2)
	};
	
	// Adjust cache-size
	let chunk_count = cli.cache_size / cli.block_size;
	http_file.adjust_cache_size(if chunk_count == 0 { 1 } else { chunk_count }, cli.block_size);
	
	// Create FS
	let file_system = httpfs::HttpFS::new(http_file, std::time::Duration::from_secs(cli.timeout));
	
	// Create FUSE-session
	use std::error::Error;
	let mut fuse_session = match fuse::Session::new(file_system, &std::path::Path::new(&cli.mountpoint), &[]) {
		Ok(fuse_session) => fuse_session,
		Err(error) => die(&format!("Failed to initialize FUSE-filesystem: {}", error.description()), 3)
	};
	
	eprintln!("{}", OK_MESSAGE);
	fuse_session.run().expect("Failed to spawn FUSE-session");
}



fn help(msg: &str) -> ! {
	// Build message
	let mut message = msg.to_owned() + "\n";
	message += include_str!("help.txt");
	
	// Replace `%PROGRAM_NAME%`
	let program_name = std::env::args().next().unwrap_or("<program_name>".to_owned());
	message = message.replace("%PROGRAM_NAME%", &program_name);
	
	die(&message, if msg != "" { 1 } else { 0 })
}

fn licenses() -> ! {
	die(include_str!("licenses.txt"), 0)
}



fn main() {
	// Parse CLI-args
	let cli = CLI::parse();
	
	// Check for `--licenses` or `--help`
	if cli.licenses { licenses() }
	if cli.help { help("") }
	
	// Check if we should fork
	if cli.no_fork { fuse(&cli) }
		else { fork(&cli) }
}