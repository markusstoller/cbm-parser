pub mod disk;
mod utils;

use async_channel::*;
use clap::Parser;
use disk::cbm::{D64, DiskParser};
use std::ffi::OsStr;
use std::fs;
use std::fs::DirEntry;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use tokio::*;
use utils::helper::Element;
use utils::helper::{Database, SqliteHandler};
use zip::ZipArchive;

/// Simple d64 parser
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the file to be processed or directory containing files to be processed
    #[arg(short, long, default_value = "d64_files.sqlite")]
    dbfile: String,

    /// Path to the file to be processed or directory containing files to be processed
    #[arg(short, long)]
    path: String,

    /// Number of threads to use
    #[arg(short, long, default_value_t = 4)]
    threads: u8,

    /// Number of threads to use
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

/// Processes a D64 disk image buffer, extracts files, and adds entries to a database via a helper.
///
/// This function takes a buffer representation of a D64 disk image, parses it using the `D64`
/// instance, and processes each file found in the image. For each file that is not a directory,
/// an entry is added to the database using the provided `SqliteHandler`.
///
/// If the image cannot be loaded or no files are found, appropriate messages are logged, and
/// the function returns `false`.
///
/// # Parameters
/// - `buffer`: A slice of bytes representing the D64 image to be processed.
/// - `parent_file`: A string slice specifying the name of the parent file from which the D64
///   image was derived.
/// - `helper`: A reference-counted, thread-safe wrapper (`Arc<Mutex<SqliteHandler>`) allowing
///   safe concurrent access to the database handler.
///
/// # Returns
/// - `true` if the D64 image was successfully loaded and files were processed.
/// - `false` if the image could not be loaded or no files were found.
///
/// # Behavior
/// - Initializes a `D64` object and disables debug output.
/// - Attempts to parse the `buffer` as a D64 image.
/// - Logs an error if the image cannot be loaded.
/// - Iterates through all files in the image, excluding directories, and for each file:
///   - Creates an `Element` object with file data and the parent file name.
///   - Adds it to the database using the `SqliteHandler`.
/// - Logs an error if no files are found.
///
/// ```rust
/// // Example usage:
/// let buffer = std::fs::read("disk_image.d64").unwrap();
/// let parent_file = "parent_file.d64";
/// let helper = Arc::new(Mutex::new(SqliteHandler::new("database.db")));
///
/// if process_d64_image(&buffer, parent_file, &helper) {
///     println!("D64 image processed successfully");
/// } else {
///     println!("Failed to process D64 image");
/// }
/// ```
fn process_d64_image(buffer: &[u8], parent_file: &str, helper: &Arc<Mutex<SqliteHandler>>) -> Result<bool, String> {
    let mut d64 = D64::new();
    if !d64.set_debug_output(false).parse_from_buffer(buffer) {
        return Err("file not loaded".to_string());
    }

    if let Some(files) = d64.get_all_files() {
        for file in files {
            if file.directory {
                continue;
            }
            helper.lock().unwrap().add_item(Element {
                prg: file,
                parent: parent_file.to_string(),
            });
            /*
            println!(
                "filename: {:16} track: #{:2.2}:{:2.2} length: {:10} sha256: {}",
                file.name,
                file.track,
                file.sector,
                file.data.len(),
                file.hashes.sha256
            );

             */
        }
        return Ok(true);
    } else {
        Err("no files found".to_string())
    }
}

/// Processes a `.d64` file by reading its contents and passing it to a handler.
///
/// # Arguments
///
/// * `parent_file` - A string slice that holds the file path to the `.d64` file to be processed.
/// * `helper` - An `Arc` wrapped `Mutex` containing a `SqliteHandler` instance,
///   which is used as a helper to manage database operations.
///
/// # Returns
///
/// * A `bool` indicating whether the file was successfully processed (`true` for success, `false` for failure).
///
/// # Behavior
///
/// 1. Attempts to read the contents of the file specified by `parent_file`.
/// 2. If the file is successfully read, the contents are passed to the `process_d64_image` function
///    along with `parent_file` and `helper` for further processing.
/// 3. If the file cannot be read, logs "file not loaded" to the console and returns `false`.
///
/// # Example
///
/// ```rust
/// let handler = Arc::new(Mutex::new(SqliteHandler::new("database.db")));
/// let process_result = process_d64_file("example.d64", &handler);
/// println!("Processing result: {}", process_result);
/// ```
fn process_d64_file(parent_file: &str, helper: &Arc<Mutex<SqliteHandler>>) -> i32 {
    let buffer = fs::read(parent_file);
    if buffer.is_ok() {
        if let Err(result) = process_d64_image(buffer.unwrap().as_slice(), parent_file, helper) {
            println!("file not processed - error: {}", result);
        } else {
            return 1;
        }
    } else {
        println!("file not loaded");
    }
    0
}

/// Extracts the file extension from a given filename if it exists.
///
/// # Arguments
///
/// * `filename` - A string slice that represents the filename from which the extension is to be extracted.
///
/// # Returns
///
/// This function returns an `Option<&str>`:
/// * `Some(&str)` containing the file extension if it exists.
/// * `None` if the file has no extension or the extension could not be determined.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use std::ffi::OsStr;
///
/// let filename = "example.txt";
/// let extension = get_extension_from_filename(filename);
/// assert_eq!(extension, Some("txt"));
///
/// let filename_without_extension = "example";
/// let extension = get_extension_from_filename(filename_without_extension);
/// assert_eq!(extension, None);
/// ```
///
/// # Notes
///
/// This function uses `std::path::Path::extension()` to determine the extension,
/// which excludes the leading dot (`.`) from the result. The function also converts
/// the extension from an `OsStr` to a `&str` for ease of use.
fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

/// Processes a `.zip` file: opens the archive, finds all `.d64` entries, parses them,
/// and forwards each to the database via the existing D64 buffer parser.
///
/// Returns true if at least one `.d64` entry was successfully processed.
fn process_zip_file(zip_path: &str, helper: &Arc<Mutex<SqliteHandler>>) -> i32 {
    let mut files_processed_ok: i32 = 0;

    let file = match File::open(zip_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open zip file {}: {}", zip_path, e);
            return 0;
        }
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to read zip archive {}: {}", zip_path, e);
            return 0;
        }
    };

    for i in 0..archive.len() {
        let mut entry = match archive.by_index(i) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to access zip entry {} in {}: {}", i, zip_path, e);
                continue;
            }
        };
        if !entry.is_file() {
            continue;
        }
        let name = entry.name().to_string();
        if let Some(ext) = get_extension_from_filename(&name) {
            if ext.eq_ignore_ascii_case("d64") {
                let mut buf = Vec::new();
                if let Err(e) = entry.read_to_end(&mut buf) {
                    eprintln!("Failed to read entry {} in {}: {}", name, zip_path, e);
                    continue;
                }
                let parent = format!("{}!{}", zip_path, name);
                if let Err(result) = process_d64_image(&buf, &parent, helper) {
                    eprintln!("Failed to process entry {} in {} - Error {}", name, zip_path, result);
                } else {
                    files_processed_ok += 1;
                }
            }
        }
    }

    files_processed_ok
}

/// Processes a `.7z` archive file, extracting its contents and handling `.d64` files within.
///
/// # Parameters
/// - `archive_path`: A string slice representing the file path to the `.7z` archive.
/// - `helper`: An `Arc<Mutex<SqliteHandler>>` instance used for database-related operations.
///
/// # Returns
/// - A `bool` indicating success or failure. This function currently always returns `false`.
///
/// # Panics
/// The function will panic if:
/// - The `.7z` archive cannot be opened using the `sevenz_rust2::ArchiveReader`.
/// - An error occurs while reading or processing the entries in the archive.
///
/// # Details
/// 1. The function attempts to open and read entries from the `.7z` archive specified by `archive_path`.
/// 2. For each entry in the archive:
///    - If the file extension is `.d64` (case-insensitive), the entry's data is read into a buffer.
///    - The `process_d64_image` function is called to handle the buffer.
///      - If the processing fails, an error is printed to `stderr`.
/// 3. The function currently always returns `false`, regardless of success or failure.
///
/// # Notes
/// - Entries without a `.d64` extension are ignored.
/// - The `get_extension_from_filename` function is assumed to extract the file extension from the entry name.
/// - The `process_d64_image` function is responsible for handling `.d64` file data and utilizes the provided `SqliteHandler`.
/// - The archive password is currently hardcoded as `"pass"`.
///
/// # Example
/// ```rust
/// use std::sync::{Arc, Mutex};
///
/// let sqlite_handler = Arc::new(Mutex::new(SqliteHandler::new()));
/// let archive_path = "path/to/archive.7z";
///
/// let result = process_7z_file(archive_path, &sqlite_handler);
/// println!("Processing result: {}", result);
/// ```
///
/// # Limitations
/// - This function does not provide robust error handling.
/// - The return value does not reflect the success of the archive processing; it always returns `false`.
/// - The archive password is hardcoded and cannot be customized.
fn process_7z_file(archive_path: &str, helper: &Arc<Mutex<SqliteHandler>>) -> i32 {
    // Try to decompress the 7z archive into the temp directory.
    let mut sz = sevenz_rust2::ArchiveReader::open(archive_path, "pass".into()).unwrap();
    let mut file_buffer: Vec<u8> = Vec::new();
    let mut files_processed_ok: i32 = 0;

    sz.for_each_entries(|entry, reader| {
        file_buffer.clear();
        let name = entry.name().to_string();
        if let Some(ext) = get_extension_from_filename(&name) {
            if ext.eq_ignore_ascii_case("d64") {
                let mut buf = [0u8; 1024];
                loop {
                    let read_size = reader.read(&mut buf)?;
                    if read_size == 0 {
                        break;
                    }
                    file_buffer.append(&mut buf[..read_size].to_vec());
                }

                let parent = format!("{}!{}", archive_path, name);
                if let Err(result) = process_d64_image(&file_buffer, &parent, helper) {
                    eprintln!("Failed to process entry {} in {} - Error {}", name, archive_path, result);
                } else {
                    files_processed_ok += 1;
                }
            }
        }
        return Ok(true);
    })
    .unwrap();

    files_processed_ok
}

/// Asynchronous worker function that processes file paths received through a channel
/// and updates a counter for parsed files.
///
/// # Arguments
///
/// * `rx` - An `async_channel::Receiver<String>` through which file paths are received for processing.
/// * `files_parsed` - An `Arc<AtomicI32>` that keeps track of the number of successfully parsed files.
///
/// # Behavior
///
/// This function continuously listens for file paths sent through the provided `rx` receiver.
/// For each path received:
/// 1. It logs the received path to standard output.
/// 2. Clones the path to avoid ownership issues during multi-threaded processing.
/// 3. Spawns a blocking task using `tokio::task::spawn_blocking`, which sets up a utils
///    connection (`setup_database`) and processes the file using the `process_d64_file` function.
/// 4. Awaits the result of the blocking task.
/// 5. If the result of `process_d64_file` is `true`, increments the `files_parsed` counter atomically.
///
/// # Notes
///
/// * The `setup_database` and `process_d64_file` functions must be properly implemented and thread-safe
///   as they are executed within a blocking task.
/// * The `Ordering::Relaxed` model is used for atomic increments, which is sufficient if the exact
///   timing of updates does not need to enforce stricter synchronization guarantees.
///
/// # Usage Example
///
/// ```rust
/// use async_channel::{Receiver, Sender};
/// use std::sync::{Arc};
/// use std::sync::atomic::{AtomicI32, Ordering};
///
/// #[tokio::main]
/// async fn main() {
///     let (tx, rx): (Sender<String>, Receiver<String>) = async_channel::unbounded();
///     let files_parsed = Arc::new(AtomicI32::new(0));
///
///     tokio::spawn(worker(rx, files_parsed.clone()));
///
///     tx.send(String::from("file1.d64")).await.unwrap();
///     tx.send(String::from("file2.d64")).await.unwrap();
///
///     // Wait and observe the output and increment of files_parsed count.
/// }
/// ```
///
/// # Errors
///
/// Any errors occurring during the reception of messages from the channel or errors during
/// file processing and utils setup are not explicitly handled in this function,
/// aside from unwrapping the result of the spawned blocking task. To handle errors,
/// consider introducing error handling mechanisms or logging for production-safe execution.
///
/// # Considerations
///
/// * Be cautious of potential blocking operations within `setup_database` and `process_d64_file`
///   as they could impact performance if not properly optimized.
/// * Ensure appropriate logging or error handling to debug any potential issues during execution.
async fn consumer(
    rx: Receiver<DirEntry>,
    files_parsed: Arc<AtomicI32>,
    helper: Arc<Mutex<SqliteHandler>>,
) {
    while let Ok(path) = rx.recv().await {
        if path.file_type().unwrap().is_file() {
            if let Some(file_path) = path.path().to_str() {
                if let Some(extension) = get_extension_from_filename(file_path) {
                    // todo - replace by btreemap
                    if extension.eq_ignore_ascii_case("d64") {
                        println!("Processing d64: {}", file_path);
                        files_parsed
                            .fetch_add(process_d64_file(file_path, &helper), Ordering::Relaxed);
                    } else if extension.eq_ignore_ascii_case("zip") {
                        println!("Processing zip: {}", file_path);
                        files_parsed
                            .fetch_add(process_zip_file(file_path, &helper), Ordering::Relaxed);
                    } else if extension.eq_ignore_ascii_case("7z") {
                        println!("Processing 7z: {}", file_path);
                        files_parsed
                            .fetch_add(process_7z_file(file_path, &helper), Ordering::Relaxed);
                    }
                };
            }
        }
    }
}

/// Asynchronous producer function that orchestrates workers to process file paths.
///
/// This function is responsible for initializing and managing worker threads, producing
/// file paths to distribute among consumer workers, and ensuring proper synchronization
/// and shutdown when all work is completed.
///
/// # Arguments
///
/// * `args` - A reference to an `Args` struct which contains:
///   * `path`: The directory path containing the files to process.
///   * `dbfile`: The path to the SQLite database file.
///   * `threads`: The number of worker threads to spawn.
///
/// # Workflow
///
/// 1. Creates shared atomic and mutex-controlled resources:
///    - `files_parsed`: An `AtomicI32` to track the number of processed files.
///    - `sqlite`: A `Mutex` holding a `SqliteHandler` to manage database operations.
/// 2. Sets up an unbounded channel (`tx`, `rx`) for sending file paths from the producer to consumers.
///    - Spawns `args.threads` worker threads that run the `consumer` function, each receiving a copy
///      of the channel's receiver (`rx`) and shared resources.
/// 3. Iterates through the files in the directory specified by `args.path`:
///    - Successfully resolved file paths are sent through the channel to workers.
///    - If sending fails, the error is logged, and the loop breaks.
/// 4. Closes the sending side of the channel to signal workers that no more data will be sent.
/// 5. Waits for all worker threads to complete their execution:
///    - Logs any errors encountered during thread execution.
/// 6. Ensures proper shutdown of the database connection by locking and closing `sqlite`.
/// 7. Outputs the total count of files processed as recorded in the `files_parsed` counter.
///
/// # Errors
///
/// This function logs errors related to:
/// * Failed attempts to send file paths to the worker channel.
/// * Worker threads that encounter errors during execution.
///
/// # Remarks
///
/// * The `files_parsed` atomic counter is accessed using the `Relaxed` ordering mode.
/// * This function ensures thread-safe shared resource access with `Arc` and proper synchronization mechanisms.
/// * Error handling is minimal and focuses mainly on logging issues rather than fully resolving them.
async fn producer(args: &Args) {
    let files_parsed: Arc<AtomicI32> = Arc::new(AtomicI32::new(0));
    let sqlite = Arc::new(Mutex::new(SqliteHandler::new(&args.dbfile)));

    let (tx, rx) = unbounded();
    let mut children = Vec::new();

    for _ in 0..args.threads {
        let rx_thread = rx.clone();
        children.push(spawn(consumer(
            rx_thread,
            files_parsed.clone(),
            sqlite.clone(),
        )));
    }

    if let Ok(paths) = fs::read_dir(&args.path) {
        for path in paths.filter_map(Result::ok) {
            if let Err(_) = tx.send(path).await {
                eprintln!("Failed to send file path to worker");
                break;
            }
        }
    }

    tx.close();

    for child in children {
        if let Err(e) = child.await {
            eprintln!("Worker thread failed: {:?}", e);
        }
    }

    // Ensure proper shutdown of the database connection/write out remaining data.
    sqlite.lock().unwrap().close();

    println!(
        "{} files parsed",
        files_parsed.load(Ordering::Relaxed).to_string()
    );
}

/// The asynchronous main function is the entry point of the application.
///
/// This function runs using the Tokio runtime, which allows for asynchronous, non-blocking tasks.
///
/// * Functionality:
/// - Prints the program title and version to the console (`simple d64 parser v0.1\n`).
/// - Parses command-line arguments into an `Args` structure using the `Args::parse()` method.
/// - Prints the parsed arguments to the console for debugging or informational purposes.
/// - Delegates further processing or work to the `work` asynchronous function, passing in the parsed arguments.
///
/// # Remarks
/// The function uses the `#[tokio::main]` attribute to enable asynchronous execution.
/// Ensure the `Args` structure and the `work` function are defined elsewhere in the code.
///
/// # Example
/// Invoking this application from the command line with arguments will automatically:
/// 1. Parse the given arguments.
/// 2. Perform further processing as defined by the `work` function.
///
/// ```bash
/// $ cargo run -- <your-args>
/// simple d64 parser v0.1
/// args: <parsed-args>
/// ```
#[tokio::main]
async fn main() {
    println!("simple d64 parser v0.1\n");
    let args = Args::parse();
    println!("args: {:?}", args);

    producer(&args).await;
}
