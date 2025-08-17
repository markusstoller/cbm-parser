pub mod disk;
mod utils;

use async_channel::*;
use clap::Parser;
use disk::cbm::{D64, DiskParser};
use std::ffi::OsStr;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use tokio::*;
use utils::helper::Element;
use utils::helper::{Database, SqliteHandler};
// Add to Cargo.toml

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

/// Processes a `.d64` file and stores information about its contained files into a utils.
///
/// This function reads the provided `.d64` file (a Commodore 64 disk image),
/// retrieves all files stored within it, and inserts their metadata into a
/// utils. If the `.d64` file cannot be loaded, or no files are found, the
/// function will return `false`. Otherwise, it returns `true`.
///
/// # Arguments
///
/// * `parent_file` - A string slice that holds the path to the `.d64` file to be processed.
/// * `conn` - A mutable reference to a utils `Connection` object used for storing file metadata.
///
/// # Returns
///
/// * `true` if the file was successfully processed and its metadata stored in the utils.
/// * `false` if the file couldn't be loaded or no files were found within the disk image.
///
/// # Behavior
///
/// 1. Creates a new instance of the `D64` parser.
/// 2. Disables debug output and attempts to parse the `.d64` file provided in `parent_file`.
/// 3. If the `.d64` file is loaded successfully, retrieves all contained files.
/// 4. For each file:
///     - Skips if the file is marked as a directory.
///     - Outputs details such as filename, track, sector, length, and SHA-256 hash.
///     - Inserts the file's metadata (e.g., parent file name, track, sector, length, hashes) into the utils.
/// 5. Commits the transaction after processing all files.
///
/// # Errors
///
/// * If the transaction fails to be created, the function will `unwrap` and panic.
/// * If the `.d64` file cannot be parsed, no metadata will be stored, and the transaction will still be committed.
/// * If the SQL insertion statements or `D64` operations fail, the application will panic.
///
/// # Logging
///
/// - Outputs a message if the `.d64` file cannot be loaded.
/// - Outputs file metadata if files are found and processed.
/// - Outputs a message if no valid files are found in the `.d64` image.
///
/// # Example
///
/// ```
/// use rusqlite::Connection;
///
/// let mut conn = Connection::open_in_memory().unwrap();
/// let result = process_d64_file("/path/to/image.d64", &mut conn);
/// if result {
///     println!("File processed successfully.");
/// } else {
///     println!("Failed to process file.");
/// }
/// ```
fn process_d64_file(parent_file: &str, helper: &Arc<Mutex<SqliteHandler>>) -> bool {
    let mut d64 = D64::new();
    if !d64.set_debug_output(false).parse_file(parent_file) {
        println!("file not loaded");
        return false;
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
        true
    } else {
        println!("no files found");
        false
    }
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
                    if extension == "d64" {
                        println!("Processing file: {}", file_path);
                        process_d64_file(file_path, &helper);
                        files_parsed.fetch_add(1, Ordering::Relaxed);
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
