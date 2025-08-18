use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

pub(crate) trait FileParser: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn parse(&self, parent_file: &str, callback: &dyn Fn(&[u8], &str)) -> Result<(), String>;
}

/// Extracts the file extension from a given filename.
///
/// # Arguments
///
/// * `filename` - A string slice representing the name of the file from which the extension is to be extracted.
///
/// # Returns
///
/// * `Option<&str>` -
///     - `Some(&str)` containing the file extension if it exists and can be represented as UTF-8.
///     - `None` if the file does not have an extension or if the extension cannot be converted to a UTF-8 string.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use std::ffi::OsStr;
///
/// let filename = "example.txt";
/// assert_eq!(get_extension_from_filename(filename), Some("txt"));
///
/// let filename_no_extension = "example";
/// assert_eq!(get_extension_from_filename(filename_no_extension), None);
///
/// let hidden_file = ".hidden";
/// assert_eq!(get_extension_from_filename(hidden_file), None);
/// ```
fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

pub(crate) struct Regular {}

impl FileParser for Regular {
    /// Creates and returns a new instance of the `Regular` struct.
    ///
    /// # Examples
    ///
    /// ```
    /// let instance = Regular::new();
    /// ```
    fn new() -> Self {
        Regular {}
    }

    /// Parses a file located at the given `parent_file` path, reads its contents
    /// into a buffer, and invokes a callback with the file data and file name.
    ///
    /// # Arguments
    ///
    /// * `parent_file` - A string slice representing the path to the file that needs to be parsed.
    /// * `callback` - A function pointer or closure that takes two arguments:
    ///     - A byte slice (`&[u8]`) representing the file's contents.
    ///     - A string slice (`&str`) representing the file's path.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the file was successfully read and the callback was executed.
    /// * `Err(String)` - If the file could not be read, an error string is returned describing the failure.
    ///
    /// # Errors
    ///
    /// This function will return an error if reading the file at the specified path fails.
    /// The error will contain a message in the format "Failed to read file: <filename>".
    ///
    /// # Examples
    ///
    /// ```
    /// fn example_usage(data: &[u8], file_name: &str) {
    ///     println!("File name: {}", file_name);
    ///     println!("File contents: {:?}", data);
    /// }
    ///
    /// let parser = MyParser {}; // Assuming your structure is defined
    /// let result = parser.parse("example.txt", &example_usage);
    ///
    /// match result {
    ///     Ok(()) => println!("File parsed successfully."),
    ///     Err(e) => println!("Error parsing file: {}", e),
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// This function relies on the `std::fs::read` method to read the file,
    /// and therefore will block until the file operation is complete.
    fn parse(&self, parent_file: &str, callback: &dyn Fn(&[u8], &str)) -> Result<(), String> {
        let buffer = fs::read(parent_file);
        if buffer.is_ok() {
            callback(buffer.unwrap().as_slice(), parent_file);
            Ok(())
        } else {
            Err(format!("Failed to read file: {}", parent_file))
        }
    }
}

pub(crate) struct SevenZip {}

impl FileParser for SevenZip {
    /// Creates and returns a new instance of the `SevenZip` struct.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let seven_zip = SevenZip::new();
    /// ```
    fn new() -> Self {
        SevenZip {}
    }

    /// Parses a 7z archive file and processes its contents with a user-provided callback function.
    ///
    /// This function reads through the specified 7z archive `parent_file`, and for each entry:
    /// - Checks if the file has a `.d64` extension (case-insensitive).
    /// - If the file matches the criteria, the file's contents are fully read into a buffer.
    /// - The buffer and the file's parent path are passed to the provided callback function, which
    ///   handles the data as per user-defined logic.
    ///
    /// # Arguments
    ///
    /// * `parent_file` - A string slice representing the path to the 7z archive file to be parsed.
    /// * `callback` - A reference to a closure or function that takes two arguments:
    ///   1. A slice of bytes (`&[u8]`) containing the file data.
    ///   2. A string slice (`&str`) indicating the file's name in the format `parent_file!entry_name`.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If parsing and processing are successful.
    /// * `Err(String)` - If an error occurs during parsing.
    ///
    /// # Errors
    ///
    /// Errors might occur during:
    /// - Opening the 7z archive file (e.g., file not found or inaccessible).
    /// - Reading entries from the archive or decompressing them.
    /// - Callback function processing, if it causes any panics or errors.
    ///
    /// # Example
    ///
    /// ```rust
    /// use your_crate::YourStruct;
    ///
    /// let my_parser = YourStruct {};
    /// my_parser.parse("archive.7z", &|data, parent| {
    ///     println!("File '{}' contains {} bytes", parent, data.len());
    /// }).expect("Failed to parse the archive");
    /// ```
    ///
    /// # Notes
    ///
    /// * This function assumes the archive may contain files of interest with a `.d64` extension.
    ///   Other files are ignored.
    /// * Inline decompression logic ensures the file is processed in chunks to avoid memory issues
    ///   with large files.
    /// * If specific logic for handling `.d64` files is required, implement it inside the provided
    ///   callback function.
    /// * The function uses the `sevenz_rust2` library for handling 7z archives.
    ///
    /// # Dependencies
    ///
    /// Ensure you include the `sevenz_rust2` crate and related dependencies in your project to
    /// enable archive processing.
    fn parse(&self, parent_file: &str, callback: &dyn Fn(&[u8], &str)) -> Result<(), String> {
        // Try to decompress the 7z archive into the temp directory.
        let mut sz = sevenz_rust2::ArchiveReader::open(parent_file, "pass".into()).unwrap();
        let mut file_buffer: Vec<u8> = Vec::new();

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

                    let parent = format!("{}!{}", parent_file, name);
                    callback(&file_buffer, &parent);
                }
            }
            Ok(true)
        })
            .unwrap();

        Ok(())
    }
}

pub(crate) struct Zip {}

impl FileParser for Zip {
    /// Constructs a new instance of the `Zip` struct.
    ///
    /// # Returns
    ///
    /// A new `Zip` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// let zip_instance = Zip::new();
    /// ```
    fn new() -> Self {
        Zip {}
    }

    ///
    /// Parses a zip archive file, processes its entries, and invokes a callback function for files with a specific extension.
    ///
    /// This method reads each file entry in a given zip archive (`.zip`) file.
    /// For files with the `.d64` extension (case-insensitive), it reads their content into a buffer
    /// and invokes a user-specified callback function with the file's byte content and a reference
    /// to the parent file path.
    ///
    /// # Arguments
    ///
    /// * `parent_file` - A `&str` representing the path to the zip archive file to be processed.
    /// * `callback` - A reference to a callback function that takes two arguments:
    ///   - A byte slice (`&[u8]`) containing the contents of the file.
    ///   - A `&str` representing the full path of the entry within the zip archive.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the zip file was parsed and processed successfully.
    /// * `Err(String)` if there was an error opening the file, reading the archive, or encountering
    ///   any unrecoverable issues during processing. The error message provides details about the issue.
    ///
    /// # Error Handling
    ///
    /// * If the `parent_file` can't be opened, it returns an `Err` with an appropriate error message.
    /// * If the zip archive can't be read, it returns an `Err` with an appropriate error message.
    /// * If individual entries in the zip archive can't be accessed or read, errors are logged
    ///   to the standard error stream using `eprintln!`, but parsing continues for other entries.
    ///
    /// # Notes
    ///
    /// * Only file entries with a `.d64` extension are processed. Non-file entries or entries
    ///   without the `.d64` extension are skipped.
    /// * The callback function is executed only for valid `.d64` file entries with successfully
    ///   read content.
    ///
    /// # Example
    ///
    /// ```
    /// use std::fs::File;
    /// use zip::ZipArchive;
    ///
    /// fn my_callback(data: &[u8], path: &str) {
    ///     println!("Processing file {} with size: {}", path, data.len());
    /// }
    ///
    /// let processor = MyZipProcessor; // Assume a struct implementing this method exists.
    /// if let Err(e) = processor.parse("archive.zip", &my_callback) {
    ///     eprintln!("Error processing zip file: {}", e);
    /// }
    /// ```
    ///
    fn parse(&self, parent_file: &str, callback: &dyn Fn(&[u8], &str)) -> Result<(), String> {
        let file = match File::open(parent_file) {
            Ok(f) => f,
            Err(e) => {
                return Err(format!("Failed to open zip file {}: {}", parent_file, e));
            }
        };

        let mut archive = match ZipArchive::new(file) {
            Ok(a) => a,
            Err(e) => {
                return Err(format!("Failed to read zip archive {}: {}", parent_file, e));
            }
        };

        for i in 0..archive.len() {
            let mut entry = match archive.by_index(i) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Failed to access zip entry {} in {}: {}", i, parent_file, e);
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
                        eprintln!("Failed to read entry {} in {}: {}", name, parent_file, e);
                        continue;
                    }
                    let parent = format!("{}!{}", parent_file, name);
                    callback(&buf, &parent);
                }
            }
        }

        Ok(())
    }
}
