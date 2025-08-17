pub mod disk;
pub mod processor;
pub mod utils;

use async_channel::*;
use clap::Parser;
use disk::cbm::{D64, DiskParser};
use processor::collector::FileParser;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use tokio::*;
use utils::helper::Element;
use utils::helper::{Database, SqliteHandler};

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
    helper: Arc<Mutex<SqliteHandler>>,
) {
    let parsers = create_parsers();

    while let Ok(path) = rx.recv().await {
        if !path.file_type().unwrap().is_file() {
            continue;
        }

        if let Some(file_path) = path.path().to_str() {
            let Some(extension) = get_extension_from_filename(file_path) else {
                continue;
            };

            if let Some(handler) = parsers.get(extension.to_lowercase().as_str()) {
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
    sqlite
        .lock()
        .unwrap()
        .close()
        .expect("Failed to close database");

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
