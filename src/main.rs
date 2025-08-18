pub mod disk;
pub mod processor;
pub mod utils;

use async_channel::*;
use clap::Parser;
use processor::collector::*;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::atomic::{AtomicI32};
use std::sync::{Arc, Mutex};
use tokio::*;
use utils::helper::{Database, SqliteHandler, ProgressState};
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



/// Asynchronous consumer task that processes directory entries received through a channel.
///
/// This function continuously listens for `DirEntry` objects sent to the provided `Receiver`,
/// and processes them if they are determined to be files. File paths along with their extensions
/// are extracted and passed to the context for processing.
///
/// # Arguments
///
/// * `rx` - An asynchronous `Receiver` that sends `DirEntry` items, representing directory entries.
/// * `files_parsed` - A shared atomic counter tracking the number of successfully parsed files.
/// * `files_attempted` - A shared atomic counter tracking the number of attempted file parsing operations.
/// * `sqlite` - A thread-safe, shared `SqliteHandler` wrapped in a `Mutex` for database interaction.
///
/// # Behavior
///
/// * Checks if the received `DirEntry` represents a file. If it is not a file, it skips further processing.
/// * Extracts the file path string and file extension from the directory entry if available.
/// * Passes the file path and extension to `process_file` using the provided file-processing context.
///
/// # Note
///
/// * This function runs in an infinite loop, continuously consuming directory entries from the receiver
///   until the channel is closed.
/// * The function is intended to be part of a multi-threaded or asynchronous file-processing pipeline.
/// * Proper error handling is required for related components (e.g., `process_file`) to ensure robustness.
///
/// # Example Usage
///
/// ```rust
/// use tokio::sync::{mpsc, Mutex};
/// use std::sync::{Arc, atomic::{AtomicI32, Ordering}};
///
/// let (tx, rx) = mpsc::channel(100);
/// let files_parsed = Arc::new(AtomicI32::new(0));
/// let files_attempted = Arc::new(AtomicI32::new(0));
/// let sqlite_handler = Arc::new(Mutex::new(SqliteHandler::new()));
///
/// tokio::spawn(consumer(rx, files_parsed.clone(), files_attempted.clone(), sqlite_handler.clone()));
///
/// // Send directory entries to the channel using `tx`...
/// ```
async fn consumer(
    rx: Receiver<DirEntry>,
    files_parsed: Arc<AtomicI32>,
    files_attempted: Arc<AtomicI32>,
    sqlite: Arc<Mutex<SqliteHandler>>,
) {
    let context = FileProcessingContext::new(files_parsed, files_attempted, sqlite);

    while let Ok(path) = rx.recv().await {
        if !path.file_type().is_file() {
            continue;
        }

        if let Some(file_path) = path.path().to_str() {
            if let Some(extension) = get_extension_from_filename(file_path) {
                context.process_file(file_path, extension);
            }
        }
    }
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

    // Ensure proper shutdown of the database connection
    sqlite
        .lock()
        .unwrap()
        .close()
        .expect("Failed to close database");

    progress.terminate_and_report();
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
    directory: &str,
    progress: &ProgressState,
    tx: &Sender<DirEntry>,
    supported_parsers: BTreeMap<&str, Box<dyn FileParser>>,
) {
    // Process files in the directory
    for entry in WalkDir::new(directory)
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
        progress.increment_total_files();
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
    let mut progress = ProgressState::new();
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

    // Create parsers, indexed by file extension
    let supported_parsers = create_parsers();

    // Spawn progress reporter
    progress.spawn_reporter();

    // Process files in the directory
    process_directory(&args.path, &progress, &tx, supported_parsers).await;

    // close the channel, indicating that no more files will be sent
    tx.close();

    // Wait for all workers to complete and ensure proper shutdown of the database
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
