/// Represents information about a track in a storage system or disk.
///
/// The `TrackInfo` struct stores details about a specific track and sector,
/// which can be used for various disk/location indexing purposes.
///
/// # Fields
///
/// * `track` - The number representing the track.
/// * `sector` - The number representing the sector within the track.
///
/// # Visibility
///
/// This struct is visible only within the current crate due to the `pub(crate)` visibility modifier.
pub(crate) struct TrackInfo {
    pub(crate) track: u8,
    pub(crate) sector: u8,
}

/// Represents a sector on a track in a storage device or disk.
///
/// A `Sector` is defined by its track number, sector number, and the data it contains.
/// This structure can be used to model low-level disk structures or data storage mechanisms.
///
/// # Fields
///
/// - `track` (`u8`): The
#[derive(Clone)]
pub(crate) struct Sector {
    pub(crate) track: u8,
    pub(crate) sector: u8,
    pub(crate) data: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct HASHES {
    pub(crate) md5: String,
    pub(crate) sha1: String,
    pub(crate) sha256: String,
}
#[derive(Clone)]
pub(crate) struct PRG {
    pub(crate) data: Vec<u8>,
    pub(crate) name: String,
    pub(crate) directory: bool,
    pub(crate) track: u8,
    pub(crate) sector: u8,
    pub(crate) hashes: HASHES,
}

/// The `DiskParser` trait defines an interface for parsing and manipulating `.D64` disk image files.
/// Implementing this trait provides methods for parsing sectors, loading files, and interacting
/// with the tracks and sectors of the disk.
pub(crate) trait DiskParser {
    /// Sets the debug output mode for the current instance.
    ///
    /// # Parameters
    /// - `debug_output`: A boolean value indicating whether to enable (`true`) or disable (`false`) debug output.
    ///
    /// # Returns
    /// - A mutable reference to the current instance (`&mut Self`), allowing method chaining.
    ///
    /// # Example
    /// ```
    /// let mut instance = Example::new();
    /// instance.set_debug_output(true)
    ///        .perform_action();
    /// ```
    ///
    /// Use this method to toggle debug output for logging or diagnostic purposes.
    fn set_debug_output(&mut self, debug_output: bool) -> &mut Self;

    /// Parses a file located at the given file path and updates internal state based on its contents.
    ///
    /// # Parameters
    /// - `path`: A string slice that holds the path to the file to be parsed.
    ///           This should point to a valid file location on the filesystem.
    ///
    /// # Returns
    /// - `true` if the file was successfully parsed and the internal state was updated.
    /// - `false` if the parsing failed (e.g., due to the file not existing, being inaccessible,
    ///   or containing invalid data).
    ///
    /// # Examples
    /// ```
    /// let mut parser = FileParser::new();
    /// let success = parser.parse_file("config.txt");
    /// assert!(success);
    /// ```
    ///
    /// # Errors
    /// This function does not provide detailed error information. If detailed error handling
    /// is required, consider implementing a more descriptive error reporting mechanism.
    fn parse_file(&mut self, path: &str) -> Result<bool, String>;

    /// Parses data from the provided buffer and updates the internal state of the struct.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A slice of bytes (`&[u8]`) containing the data to be parsed.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if parsing was successful, `false` otherwise.
    ///
    /// # Remarks
    ///
    /// This function processes the given buffer and extracts meaningful information
    /// to update the struct's internal properties or state. If the buffer doesn't
    /// contain valid or sufficient data for parsing, the function will return `false`.
    ///
    /// # Example
    ///
    /// ```
    /// let mut obj = MyStruct::new();
    /// let buffer: &[u8] = &[0x01, 0x02, 0x03];
    /// let result = obj.parse_from_buffer(buffer);
    /// assert!(result);
    /// ```
    ///
    /// # Safety
    ///
    /// Ensure the buffer provided is valid and has the expected format to avoid unexpected behavior.
    fn parse_from_buffer(&mut self, buffer: &[u8]) -> Result<bool, String>;

    /// Retrieves the total number of sectors.
    ///
    /// This method returns the count of sectors associated with the implementing object.
    /// The exact meaning of a "sector" may depend on the particular implementation and its context.
    ///
    /// # Returns
    /// - `usize`: The total number of sectors.
    ///
    /// # Examples
    /// ```
    /// let sector_count = my_object.get_sector_count();
    fn get_sector_count(&self) -> usize;

    /// Retrieves a sector based on the given sector identifier.
    ///
    /// # Parameters
    /// - `sector`: An `i32` value representing the identifier of the sector to retrieve.
    ///
    /// # Returns
    /// - `Some(Sector)` if a sector corresponding to the given identifier is found.
    /// - `None` if no sector matches the given identifier.
    ///
    /// # Examples
    /// ```
    /// let map = Map::new();
    /// if let Some(sector) = map.get_sector(5) {
    ///     println!("Sector found: {:?}", sector);
    /// } else {
    ///     println!("Sector not found.");
    /// }
    /// ```
    fn get_sector(&self, sector: i32) -> Option<Sector>;

    /// Creates a new instance of the implementing type.
    ///
    /// # Returns
    /// A new instance of `Self`.
    ///
    /// # Example
    /// ```rust
    /// let instance = ImplementingType::new();
    /// ```
    fn new() -> Self;

    /// Retrieves the track number associated with the given sector.
    ///
    /// # Parameters
    /// - `sector`: An integer representing the sector for which the track number is to be retrieved.
    ///
    /// # Returns
    /// - `Some(i32)`: The track number corresponding to the specified sector, if it exists.
    /// - `None`: If no track is associated with the specified sector.
    ///
    /// # Example
    /// ```rust
    /// let track = some
    fn get_track_info_for_raw_sector(&self, sector: i32) -> Option<TrackInfo>;

    /// Retrieves the sector information for a given track.
    ///
    /// # Arguments
    ///
    /// * `track` - A `TrackInfo` object representing the track for which sector information
    ///   is requested.
    ///
    /// # Returns
    ///
    /// Returns an `Option<i32>`:
    /// * `Some(i32)` - The sector information corresponding to the provided track.
    /// * `None` - If no sector information is available or the track does not map to a sector.
    ///
    /// # Example
    ///
    /// ```rust
    /// let track_info = TrackInfo { /* fields */ };
    /// let sector = object.get_sector_for_track_info(track_info);
    ///
    /// match sector {
    ///     Some(s) => println!("Sector: {}", s),
    ///     None => println!("No sector found for the given track."),
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// Ensure that the `TrackInfo` object provided contains valid data to correctly retrieve the sector.
    fn get_sector_for_track_info(&self, track: &TrackInfo) -> Option<i32>;

    /// Recursively searches for and collects all the files within a specified directory.
    ///
    /// # Behavior
    ///
    /// - This function uses the current state to determine the root directory to search from.
    /// - It gathers all files found, including those in subdirectories, and processes or stores
    ///   them as per the implementation.
    /// - The function ignores directories, symbolic links, or system-specific hidden files
    ///   (if applicable).
    ///
    /// # Preconditions
    ///
    /// - The function assumes that the root directory is already set or initialized.
    /// - The caller must have sufficient permissions to read from the directory and its subdirectories.
    ///
    /// # Side Effects
    ///
    /// - This function may modify internal state by populating a collection or structure with
    ///   file paths.
    ///
    /// # Notes
    ///
    /// - Ensure that you handle potential errors such as inaccessible directories,
    ///   or missing files within your implementation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mut file_handler = FileHandler::new("/path/to/directory");
    /// file_handler.find_all_files();
    /// ```
    fn find_all_files(&mut self) -> Option<Vec<PRG>>;

    /// Retrieves all files currently stored or managed in the application.
    ///
    /// This function fetches and returns a vector of `PRG` objects, which represent the files
    /// in the system. If no files are available, it returns `None`.
    ///
    /// # Returns
    /// - `Some(Vec<PRG>)`: A vector containing all `PRG` objects if files exist.
    /// - `None`: If there are no files available.
    ///
    /// # Examples
    /// ```rust
    /// let mut app = Application::new();
    /// if let Some(files) = app.get_all_files() {
    ///     for file in files {
    ///         println!("{:?}", file);
    ///     }
    /// } else {
    ///     println!("No files found.");
    /// }
    /// ```
    fn get_all_files(&mut self) -> Option<Vec<PRG>>;
}
