use crate::disk::cbm::{D64, DiskParser};
use crate::processor::archive::{FileParser, Regular, SevenZip, Zip};
use crate::utils::helper::{Database, Element, SqliteHandler};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};

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
pub(crate) fn create_parsers() -> BTreeMap<&'static str, Box<dyn FileParser>> {
    let mut parsers: BTreeMap<&str, Box<dyn FileParser>> = BTreeMap::new();
    parsers.insert("d64", Box::new(Regular::new()));
    parsers.insert("zip", Box::new(Zip::new()));
    parsers.insert("7z", Box::new(SevenZip::new()));
    parsers
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

/// Context object that groups related parameters for file processing
pub(crate) struct FileProcessingContext {
    files_parsed: Arc<AtomicI32>,
    files_attempted: Arc<AtomicI32>,
    helper: Arc<Mutex<SqliteHandler>>,
    parsers: BTreeMap<&'static str, Box<dyn FileParser>>,
}

impl FileProcessingContext {
    pub(crate) fn new(
        files_parsed: Arc<AtomicI32>,
        files_attempted: Arc<AtomicI32>,
        sqlite: Arc<Mutex<SqliteHandler>>,
    ) -> Self {
        Self {
            files_parsed,
            files_attempted,
            helper: sqlite,
            parsers: create_parsers(),
        }
    }

    /// Increments the `files_attempted` counter by 1.
    ///
    /// This method atomically increases the value of the `files_attempted` field,
    /// which is expected to be an `AtomicUsize`, using a relaxed memory ordering.
    /// Relaxed ordering is used here because this increment operation does not
    /// require synchronization with other threads or a strict ordering guarantee.
    ///
    /// # Usage
    /// Call this method whenever a file processing attempt occurs to keep the
    /// `files_attempted` counter up to date.
    ///
    /// # Example
    /// ```
    /// let tracker = Tracker::new();
    /// tracker.increment_attempts();
    /// println!("Files attempted: {}", tracker.files_attempted.load(Ordering::Relaxed));
    /// ```
    ///
    /// # Note
    /// Ensure that the concurrent usage of `files_attempted` aligns with the intended
    /// memory consistency model, as this uses relaxed ordering.
    fn increment_attempts(&self) {
        self.files_attempted.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the `files_parsed` counter by 1.
    ///
    /// This function atomically increases the value of the `files_parsed` field,
    /// which is an `AtomicUsize`, using the `fetch_add` method with relaxed memory ordering.
    ///
    /// # Memory Ordering
    /// The `Ordering::Relaxed` parameter ensures that the increment operation
    /// does not impose synchronization or ordering constraints beyond the atomicity
    /// of the operation itself. This is a good choice when precise ordering of operations
    /// across threads is not required, but atomic updates are still necessary.
    ///
    /// # Use Case
    /// Typically used in scenarios where a counter is maintained to track the number
    /// of files processed or parsed, and multiple threads may concurrently update the counter.
    ///
    /// # Example
    /// ```rust
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    ///
    /// struct Parser {
    ///     files_parsed: AtomicUsize,
    /// }
    ///
    /// impl Parser {
    ///     fn increment_parsed(&self) {
    ///         self.files_parsed.fetch_add(1, Ordering::Relaxed);
    ///     }
    /// }
    ///
    /// let parser = Parser {
    ///     files_parsed: AtomicUsize::new(0),
    /// };
    /// parser.increment_parsed();
    /// assert_eq!(parser.files_parsed.load(Ordering::Relaxed), 1);
    /// ```
    fn increment_parsed(&self) {
        self.files_parsed.fetch_add(1, Ordering::Relaxed);
    }

    /// Processes a file based on its extension and applies the appropriate parsing handler.
    ///
    /// # Parameters
    /// - `file_path`: A string slice that specifies the path to the file to be processed.
    /// - `extension`: A string slice that specifies the extension of the file, used to identify the appropriate parser.
    ///
    /// # Behavior
    /// 1. Converts the file extension to lowercase and checks if a corresponding parser is available in `self.parsers`.
    /// 2. If a parser for the given file extension is found:
    ///    - Increments the processing attempt count using `self.increment_attempts()`.
    ///    - Invokes the parser's `parse` method, passing the file path and a closure that handles the processing results.
    ///    - If parsing fails, logs the parser error along with the file name.
    /// 3. If no parser for the given extension is found, logs a message indicating the file was not processed due to an unknown extension.
    ///
    /// # Logging
    /// - Logs errors encountered during parsing.
    /// - Logs a message when a file cannot be processed because of an unsupported extension.
    ///
    /// # Example
    /// ```rust
    /// let processor = FileProcessor::new();
    /// processor.process_file("example.json", "json");
    /// ```
    ///
    /// # Notes
    /// - This function assumes that `self.parsers` is a map containing parser handlers,
    ///   where each handler implements a `parse` method.
    /// - The closure `|buffer, parent_file| self.handle_file_processing_result(buffer, parent_file)`
    ///   is used to manage the file processing result during parsing.
    ///
    /// # Error Handling
    /// - Errors encountered while parsing are not propagated but are logged to the console.
    pub(crate) fn process_file(&self, file_path: &str, extension: &str) {
        if let Some(handler) = self.parsers.get(extension.to_lowercase().as_str()) {
            self.increment_attempts();

            let result = handler.parse(file_path, &|buffer: &[u8], parent_file: &str| {
                self.handle_file_processing_result(buffer, parent_file)
            });

            if let Err(e) = result {
                println!("Parser error for file {} - error: {}", file_path, e);
            }
        } else {
            println!("file not processed {} - unknown extension", file_path);
        }
    }
    /// Handles the result of processing a file buffer.
    ///
    /// This function processes the provided buffer as a `.d64` disk image using the `process_d64_image`
    /// function. Based on the result, it either increments the parsed file count or logs an error
    /// message if processing fails.
    ///
    /// # Parameters
    ///
    /// - `buffer`: A reference to a byte slice representing the file data to be processed.
    /// - `parent_file`: A string slice representing the name or path of the parent file associated with the buffer.
    ///
    /// # Behavior
    ///
    /// - If `process_d64_image` returns an `Ok` result, the function calls `self.increment_parsed()`
    ///   to update the count of parsed files.
    /// - If `process_d64_image` returns an error, the function logs the error to the console, along
    ///   with the name of the parent file which failed processing.
    ///
    /// # Example
    ///
    /// ```
    /// self.handle_file_processing_result(&file_buffer, "example_file.d64");
    /// ```
    ///
    /// # Errors
    ///
    /// Any errors returned by `process_d64_image` will not be propagated but will instead be logged
    /// to the console with an associated error message and the file name.
    fn handle_file_processing_result(&self, buffer: &[u8], parent_file: &str) {
        match process_d64_image(buffer, parent_file, &self.helper) {
            Ok(_) => self.increment_parsed(),
            Err(error) => println!("file not processed {} - error: {}", parent_file, error),
        }
    }
}
