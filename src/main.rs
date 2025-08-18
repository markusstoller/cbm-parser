pub mod disk;
pub mod processor;
pub mod utils;

use async_channel::*;
use clap::Parser;
use disk::cbm::{D64, DiskParser};
use processor::collector::FileParser;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use tokio::time::Duration;
use tokio::*;
use utils::helper::{Database, Element, SqliteHandler};
use walkdir::{DirEntry, WalkDir};

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
fn process_d64_image(
    buffer: &[u8],
    parent_file: &str,
    helper: &Arc<Mutex<SqliteHandler>>,
) -> Result<bool, String> {
    let mut d64 = D64::new();
    if let Err(result) = d64.set_debug_output(false).parse_from_buffer(buffer) {
        return Err(result);
    }

    if let Some(files) = d64.get_all_files() {
        for file in files {
            if file.directory {
                continue;
            }
            /*
            {
                println!(
                    "filename: {:16} track: #{:2.2}:{:2.2} length: {:10} sha256: {}",
                    file.name,
                    file.track,
                    file.sector,
                    file.data.len(),
                    file.hashes.sha256
                );
            }

             */
            helper.lock().unwrap().add_item(Element {
                prg: file,
                parent: parent_file.to_string(),
            });
        }
        Ok(true)
    } else {
        Err("no files found".to_string())
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

/// Creates a collection of file parsers and associates them with file extensions.
///
/// This function initializes a `BTreeMap` to store file parsers, each identified
/// by a specific file extension. The parsers implement the `FileParser` trait
/// and are dynamically dispatched. The following associations are created:
///
/// - `"d64"`: Maps to an instance of `processor::collector::Regular`.
/// - `"zip"`: Maps to an instance of `processor::collector::Zip`.
/// - `"7z"`: Maps to an instance of `processor::collector::SevenZip`.
///
/// Each parser instance is encapsulated in a `Box` to allow for dynamic dispatch,
/// and stored in the map keyed by its corresponding file extension.
///
/// # Returns
///
/// A `BTreeMap` where keys are file extensions (`&'static str`) and values are
/// boxed instances of types implementing the `FileParser` trait.
///
/// # Example
///
/// ```ignore
/// let parsers = create_parsers();
/// if let Some(parser) = parsers.get("zip") {
///     // Use the parser for .zip files
/// }
/// ```
///
/// # Dependencies
///
/// Ensure that the modules `processor::collector::Regular`, `processor::collector::Zip`,
/// and `processor::collector::SevenZip` are correctly defined and implement
/// the `FileParser` trait.
///
/// # Panics
///
/// This function does not panic under normal circumstances.
///
/// # Notes
///
/// - The ordering of the `BTreeMap` ensures that keys are sorted, which can be
///   beneficial if iteration over the map in sorted order is required.
/// - Additional file parsers can be added to this function by extending the
///   `insert` statements with new file extensions and corresponding parser instances.
fn create_parsers() -> BTreeMap<&'static str, Box<dyn FileParser>> {
    let mut parsers: BTreeMap<&str, Box<dyn FileParser>> = BTreeMap::new();
    parsers.insert("d64", Box::new(processor::collector::Regular::new()));
    parsers.insert("zip", Box::new(processor::collector::Zip::new()));
    parsers.insert("7z", Box::new(processor::collector::SevenZip::new()));
    parsers
}

/// Asynchronous function that consumes file entries, processes them using appropriate parsers,
/// and updates a counter for successfully parsed files.
///
/// # Arguments
///
/// * `rx` - A [Receiver] channel used to receive file entries (`DirEntry`) produced by another asynchronous task.
/// * `files_parsed` - An `Arc<AtomicI32>` representing a thread-safe counter of the number of
///                    files successfully processed and parsed.
/// * `helper` - An `Arc<Mutex<SqliteHandler>>` that provides a thread-safe SQLite resource
///              handler used during file processing.
///
/// # Behavior
///
/// - The function continuously receives directory entries (`DirEntry`) from the given channel `rx`.
/// - If the received directory entry is not a file, it is ignored.
/// - For each file, it determines the file extension and checks if a corresponding parser exists
///   in the set of parsers created by `create_parsers()`.
/// - If a suitable parser is found, the parser processes the file. During processing:
///   - Each file is passed to the `process_d64_image` function for further handling.
///   - If the processing is successful, the `files_parsed` atomic counter is incremented.
///   - If an error occurs during processing, the file path and the error details are logged.
/// - If no suitable parser exists for the file's extension, a warning message is logged, indicating
///   that the file is not processed due to an unknown extension.
///
/// # Notes
///
/// - The function assumes that `rx.recv()` continuously provides new `DirEntry` items to process
///   until the channel is closed.
/// - Any issue with file parsing or SQLite processing is logged and does not terminate the
///   application.
/// - The function is designed to handle files with supported extensions only. Unknown file
///   extensions are skipped.
///
/// # Panics
///
/// - If `path.file_type()` fails unexpectedly, the function will panic.
/// - If `path.path().to_str()` returns `None`, indicating an invalid UTF-8 string, the function
///   will panic.
/// - If the parser (`selected_parser.parse`) unexpectedly fails, the function will panic with a message
///   "should not happen".
///
/// # Example Usage
///
/// ```
/// // Assuming `rx` is a Receiver<DirEntry>,
/// // `files_parsed` is an Arc<AtomicI32>,
/// // and `helper` is an initialized Arc<Mutex<SqliteHandler>>:
///
/// tokio::spawn(async move {
///     consumer(rx, files_parsed, helper).await;
/// });
/// ```
///
/// This example spawns the consumer function to process file entries as they become available.
async fn consumer(
    rx: Receiver<DirEntry>,
    files_parsed: Arc<AtomicI32>,
    files_attempted: Arc<AtomicI32>,
    helper: Arc<Mutex<SqliteHandler>>,
) {
    let parsers = create_parsers();

    while let Ok(path) = rx.recv().await {
        if !path.file_type().is_file() {
            continue;
        }

        if let Some(file_path) = path.path().to_str() {
            let Some(extension) = get_extension_from_filename(file_path) else {
                continue;
            };

            if let Some(handler) = parsers.get(extension.to_lowercase().as_str()) {
                // Count this file as attempted, regardless of success
                files_attempted.fetch_add(1, Ordering::Relaxed);
                handler
                    .parse(file_path, &|buffer: &[u8], parent_file: &str| {
                        if let Err(result) = process_d64_image(buffer, parent_file, &helper) {
                            println!("file not processed {} - error: {}", parent_file, result);
                            return;
                        }
                        files_parsed.fetch_add(1, Ordering::Relaxed);
                    })
                    .expect("should not happen");
            } else {
                println!("file not processed {} - unknown extension", &file_path);
            }
        }
    }
}

/// Shared state for tracking progress across all workers
struct ProgressState {
    files_parsed: Arc<AtomicI32>,
    files_attempted: Arc<AtomicI32>,
    total_files: Arc<AtomicI32>,
}

impl ProgressState {
    fn new() -> Self {
        Self {
            files_parsed: Arc::new(AtomicI32::new(0)),
            files_attempted: Arc::new(AtomicI32::new(0)),
            total_files: Arc::new(AtomicI32::new(0)),
        }
    }

    fn signal_completion(&self) {
        self.total_files.store(-1, Ordering::Relaxed);
    }

    fn print_final_count(&self) {
        println!("{} files parsed", self.files_parsed.load(Ordering::Relaxed));
    }
}

/// Spawns an asynchronous task to continuously report progress of a file processing operation.
///
/// This function takes a reference to a `ProgressState` struct that tracks the number of files
/// attempted and the total number of files to be processed. It creates an asynchronous task that
/// periodically prints the progress (every second) to the console in the format:
///
/// `Progress: X / Y files processed`
///
/// where `X` is the number of files attempted so far, and `Y` is the total number of files.
///
/// ### Arguments
/// - `progress`: A reference to a `ProgressState` structure. This structure contains counters for
///   `files_attempted` and `total_files`. These counters are shared between threads and updated atomically.
///
/// ### Behavior
/// - The function clones the atomic counters for `files_attempted` and `total_files` to be used in the spawned task.
/// - Inside the task:
///   - It periodically checks the current progress (attempted and total files).
///   - If the number of attempted files has changed since the last iteration, it prints the current progress to the console.
///   - The loop exits when `total_files` is set to `-1`, indicating that the task should terminate.
///
/// ### Note
/// - The task runs asynchronously in a loop and uses `tokio::time::sleep` to wait 1 second between progress checks.
///
/// ### Example Usage
/// ```rust
/// let progress_state = ProgressState {
///     files_attempted: Arc::new(AtomicI32::new(0)),
///     total_files: Arc::new(AtomicI32::new(100)) // Example total files count
/// };
///
/// spawn_progress_reporter(&progress_state);
///
/// // Simulate file processing
/// for i in 0..100 {
///     progress_state.files_attempted.store(i + 1, Ordering::Relaxed); // Update attempted count
///     tokio::time::sleep(Duration::from_millis(50)).await; // Simulated delay
/// }
///
/// progress_state.total_files.store(-1, Ordering::Relaxed); // Signal completion
/// ```
///
/// ### Requirements
/// - This function requires the use of the Tokio runtime to execute asynchronous tasks.
/// - Ensure proper synchronization (using atomic counters) for shared progress state between threads.
///
/// ### Limitations
/// - Progress is printed to the console every second and will not update more frequently even if progress changes rapidly.
/// - The function assumes that `total_files` is set to `-1` explicitly to indicate task completion.
fn spawn_progress_reporter(progress: &ProgressState) {
    let files_attempted = progress.files_attempted.clone();
    let total_files = progress.total_files.clone();

    spawn(async move {
        let mut last_printed = -1; // sentinel to force initial print
        loop {
            let attempted = files_attempted.load(Ordering::Relaxed);
            let total = total_files.load(Ordering::Relaxed);

            if attempted != last_printed {
                println!("Progress: {} / {} files processed", attempted, total);
                last_printed = attempted;
            }

            if total == -1 {
                break;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
}

/// Cleans up worker tasks and ensures proper shutdown of the database.
///
/// # Arguments
///
/// * `children` - A vector of asynchronous task join handles representing worker threads to be awaited and cleaned up.
/// * `progress` - A reference to a `ProgressState` object used to signal completion and display final progress updates.
/// * `sqlite` - A reference-counted and thread-safe mutex wrapping a `SqliteHandler`, which is responsible for managing the database connection.
///
/// # Behavior
///
/// 1. Waits for all worker threads to complete. If any thread fails, logs the error to standard error output.
/// 2. Signals the progress state to indicate that all workers have completed.
/// 3. Closes the database connection safely, ensuring the connection is properly shut down. If this operation fails, it panics with an error message.
/// 4. Prints the final progress count after all operations are complete.
///
/// # Panics
///
/// Panics if the database connection cannot be safely closed.
///
/// # Errors
///
/// Errors during worker thread completion (via `JoinHandle`) are logged to standard error but do not propagate.
///
/// # Example
///
/// ```no_run
/// use std::sync::{Arc, Mutex};
/// use tokio::task;
///
/// struct ProgressState;
/// impl ProgressState {
///     fn signal_completion(&self) { /* implementation */ }
///     fn print_final_count(&self) { /* implementation */ }
/// }
///
/// struct SqliteHandler;
/// impl SqliteHandler {
///     fn close(&self) -> Result<(), &'static str> {
///         // Close database connection
///         Ok(())
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let workers: Vec<task::JoinHandle<()>> = vec![];
///     let progress = ProgressState;
///     let sqlite = Arc::new(Mutex::new(SqliteHandler));
///
///     cleanup_workers_and_db(workers, &progress, sqlite).await;
/// }
/// ```
async fn cleanup_workers_and_db(
    children: Vec<task::JoinHandle<()>>,
    progress: &ProgressState,
    sqlite: Arc<Mutex<SqliteHandler>>,
) {
    // Wait for all workers to complete
    for child in children {
        if let Err(e) = child.await {
            eprintln!("Worker thread failed: {:?}", e);
        }
    }

    progress.signal_completion();

    // Ensure proper shutdown of the database connection
    sqlite
        .lock()
        .unwrap()
        .close()
        .expect("Failed to close database");

    progress.print_final_count();
}

/// Asynchronously processes a directory by walking through its files, filtering them based on supported parsers,
/// and sending each valid file entry to a provided channel for further processing.
///
/// # Parameters
/// - `args`: A reference to an `Args` struct containing configuration details, including the target directory path to process.
/// - `progress`: A shared `ProgressState` object used to track the total number of processed files.
/// - `tx`: A reference to the `Sender` channel used to send valid file entries (`DirEntry`) to worker tasks for further processing.
/// - `supported_parsers`: A map of file extensions (`&str`) to their corresponding file parsers (`Box<dyn FileParser>`),
///   used to determine which files are supported for processing.
///
/// # Behavior
/// - Iterates through all entries in the directory specified in `args`.
/// - Filters entries to include only files with extensions that are supported by the provided `supported_parsers`.
///   The extensions are matched case-insensitively.
/// - For each valid file, increments the `ProgressState`'s `total_files` counter.
/// - Sends each valid file entry (`DirEntry`) to the provided channel. If sending fails due to a disconnected receiver,
///   the loop exits early and prints an error to `stderr`.
///
/// # Errors
/// - If the sender channel (`tx`) is closed or disconnected, sending a file entry will fail, and no further files will be processed.
///
/// # Notes
/// - This function uses the `WalkDir` crate to recursively traverse the directory structure.
/// - Directory entries without valid UTF-8 paths or unsupported extensions are ignored.
/// - The file extension matching is performed in a case-insensitive manner.
async fn process_directory(
    args: &&Args,
    progress: &ProgressState,
    tx: &Sender<DirEntry>,
    supported_parsers: BTreeMap<&str, Box<dyn FileParser>>,
) {
    // Process files in the directory
    for entry in WalkDir::new(&args.path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            if let Some(path_str) = entry.path().to_str() {
                if let Some(ext) = get_extension_from_filename(path_str) {
                    return supported_parsers.contains_key(ext.to_lowercase().as_str());
                }
            }
            false
        })
    {
        progress.total_files.fetch_add(1, Ordering::Relaxed);
        if let Err(_) = tx.send(entry).await {
            eprintln!("Failed to send file path to worker");
            break;
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
/// 1. Creates shared atomic and mutex-controlled resources for progress tracking and database access.
/// 2. Sets up an unbounded channel for sending file paths from the producer to consumers.
/// 3. Spawns worker threads and a progress reporter.
/// 4. Iterates through supported files in the directory, sending paths to workers.
/// 5. Waits for all workers to complete and ensures proper database shutdown.
///
/// # Errors
///
/// This function logs errors related to failed channel sends and worker thread execution.
async fn producer(args: &Args) {
    let progress = ProgressState::new();
    let sqlite = Arc::new(Mutex::new(SqliteHandler::new(&args.dbfile)));
    let (tx, rx) = unbounded();
    let mut children = Vec::new();

    // Spawn worker threads
    for _ in 0..args.threads {
        let rx_thread = rx.clone();
        children.push(spawn(consumer(
            rx_thread,
            progress.files_parsed.clone(),
            progress.files_attempted.clone(),
            sqlite.clone(),
        )));
    }

    let supported_parsers = create_parsers();
    spawn_progress_reporter(&progress);
    process_directory(&args, &progress, &tx, supported_parsers).await;
    tx.close();
    cleanup_workers_and_db(children, &progress, sqlite).await;
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
