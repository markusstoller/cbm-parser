pub mod cbm {
    use endian_codec::{DecodeLE, EncodeLE, PackedSize};
    use sha1::{Digest, Sha1};

    /// The `DiskParser` trait defines an interface for parsing and manipulating `.D64` disk image files.
    /// Implementing this trait provides methods for parsing sectors, loading files, and interacting
    /// with the tracks and sectors of the disk.
    pub(crate) trait DiskParser {
        /// Parses the sectors from the provided data source.
        ///
        /// # Returns
        /// * `true` if the sectors are successfully parsed;
        /// * `false` if there is an error during parsing or if parsing fails.
        ///
        /// # Notes
        /// * This method typically alters the internal state of the object, as it works with
        ///   `&mut self`.
        /// * Ensure that
        ///
        fn parse_sectors(&mut self) -> bool;
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
        /// Parses the input and updates the state of the calling object.
        ///
        /// # Returns
        /// * `true` if the parsing is successful.
        /// * `false` otherwise.
        ///
        /// # Behavior
        /// This method modifies the internal state of the object based on the input being parsed.
        /// The parsing logic should ensure that the object remains in a consistent state
        fn parse_disk(&mut self) -> bool;
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
        fn parse_file(&mut self, path: &str) -> bool;
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
        fn parse_from_buffer(&mut self, buffer: &[u8]) -> bool;
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
        ///
        /// Loads the contents of a file specified by the given file path into the current object.
        ///
        /// # Parameters
        /// - `path`: A string slice representing the path to the file to be loaded.
        ///
        /// # Returns
        /// - `std::io::Result<()>`: Returns `Ok(())` if the file is successfully
        fn load_file(&mut self, path: &str) -> std::io::Result<()>;
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
        /// Parses the contents of a directory and returns a list of `FileEntry` objects.
        ///
        /// # Returns
        /// - `Some(Vec<FileEntry>)`: A vector containing `FileEntry` objects, each representing a file or folder
        ///   within the current directory, if the parsing is successful.
        /// - `None`: If there is an error during the parsing process or if the directory contents cannot be retrieved.
        ///
        /// # Errors
        /// This function may return `None` if:
        /// - The directory does not exist or cannot be accessed.
        /// - There is an issue reading the directory's contents.
        /// - Any encountered errors prevent successful parsing.
        ///
        /// # Example
        /// ```rust
        /// let mut handler = DirectoryHandler::new();
        /// if let Some(entries) = handler.parse_directory() {
        ///     for entry in entries {
        ///         println!("{:?}", entry);
        ///     }
        /// } else {
        ///     eprintln!("Failed to parse directory.");
        /// }
        /// ```
        ///
        /// # Notes
        /// - Ensure that the instance calling this method has been properly initialized to point to a valid directory.
        /// - The `FileEntry` type should represent individual items in the directory, including necessary metadata such as name, path, or type (file or folder).
        fn parse_directory(&mut self) -> Option<Vec<FileEntry>>;
        /// Converts a byte slice (`&[u8]`) into a hexadecimal string representation.
        ///
        /// This method takes a reference to a byte array and converts each byte into
        /// its corresponding hexadecimal representation. The resulting hexadecimal
        /// values are concatenated into a single `String`.
        ///
        /// # Arguments
        ///
        /// * `array` - A reference to a slice of `u8` values that you want to convert to a hexadecimal string.
        ///
        /// # Returns
        ///
        /// A `String` containing the hexadecimal representation of the provided byte slice.
        ///
        /// # Example
        ///
        /// ```
        /// let data = vec![15, 255, 128];
        /// let mut your_instance = YourStruct::new();
        /// let hex_string = your_instance.vec_u8to_hex_string(&data);
        /// assert_eq!(hex_string, "0fff80"); // Example output
        /// ```
        ///
        /// # Note
        ///
        /// This method requires mutable access to the struct (`&mut self`) indicating that
        /// calling it may modify the instance. Ensure that the method implementation
        /// handles this appropriately.
        fn vec_u8to_hex_string(&mut self, array: &[u8]) -> String;
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
        /// Processes a directory file and extracts file entries from it.
        ///
        /// This method takes a reference to a `PRG` file, processes its contents,
        /// and attempts to retrieve a list of `FileEntry` elements if the file is valid
        /// and contains the necessary data.
        ///
        /// # Arguments
        ///
        /// * `file` - A reference to a `PRG` file object that represents the directory file
        ///            to be processed.
        ///
        /// # Returns
        ///
        /// * `Option<Vec<FileEntry>>` - Returns `Some(Vec<FileEntry>)` containing a list of
        ///   `FileEntry` objects if processing is successful, or `None` if the file cannot
        ///   be processed or does not contain any valid entries.
        ///
        /// # Example
        ///
        /// ```rust
        /// let directory_file = PRG::new("directory.prg");
        /// let processed_entries = object.process_directory_file(&directory_file);
        ///
        /// if let Some(entries) = processed_entries {
        ///     for entry in entries {
        ///         println!("{:?}", entry);
        ///     }
        /// } else {
        ///     println!("No valid entries found or unable to process the file.");
        /// }
        /// ```
        ///
        /// # Errors
        ///
        /// This method does not explicitly handle errors but returns `None` when processing fails.
        /// Ensure that the provided file is valid and contains recognizable data for successful processing.
        ///
        /// # Notes
        ///
        /// The behavior of this method may depend on the specific implementation details of both
        /// the `PRG` and `FileEntry` types.
        fn process_directory_file(&self, file: &PRG) -> Option<Vec<FileEntry>>;
        /// Parses a directory entry from the provided byte slice and prints its details.
        ///
        /// # Arguments
        ///
        /// * `entry_bytes` - A byte slice containing the raw data of a directory entry to be parsed.
        ///
        /// # Returns
        ///
        /// * `FileEntry` - A `FileEntry` struct representation of the parsed directory entry.
        ///
        /// # Behavior
        ///
        /// This function takes the raw bytes of a directory entry, parses it into a structured
        /// `FileEntry` object, and outputs relevant details to the console or log. The function
        /// assumes that the byte slice represents a valid directory entry and requires this
        /// precondition to successfully parse the entry.
        ///
        /// # Errors
        ///
        /// If the byte slice is malformed or does not represent a valid directory entry,
        /// the returned `FileEntry` object may be incomplete or invalid, depending on the
        /// implementation of the parsing logic.
        ///
        /// # Example
        ///
        /// ```
        /// let entry_bytes = vec![/* raw bytes of a directory entry */];
        /// let file_entry = parser.parse_and_print_directory_entry(&entry_bytes);
        /// println!("{:?}", file_entry);
        /// ```
        fn parse_and_print_directory_entry(&self, entry_bytes: &[u8]) -> FileEntry;
        /// Processes a single file located at the specified sector index.
        ///
        /// This method takes a sector index as input, processes the file
        /// located at that index, and returns an optional `PRG` object. If
        /// the file cannot be processed (e.g., the sector index is invalid
        /// or the data is corrupted), it returns `None`.
        ///
        /// # Arguments
        ///
        /// * `sector_index` - An `i32` value representing the index of the sector
        ///   to process.
        ///
        /// # Returns
        ///
        /// * `Option<PRG>` - Returns `Some(PRG)` if the file is successfully processed,
        ///   or `None` if an error occurs or the file can't be processed.
        ///
        /// # Example
        ///
        /// ```rust
        /// let mut processor = FileProcessor::new();
        /// if let Some(prg) = processor.process_single_file(5) {
        ///     println!("File processed. PRG: {:?}", prg);
        /// } else {
        ///     println!("Failed to process file at sector index 5.");
        /// }
        /// ```
        fn process_single_file(&mut self, sector_index: i32) -> Option<PRG>;
        /// Collects a list of file sectors starting from a given sector.
        ///
        /// This function gathers all the sectors associated with a file,
        /// starting from the given `start_sector`, and returns them as
        /// a vector of integers. The specific implementation for determining
        /// how sectors are collected will depend on the underlying structure
        /// (e.g., filesystem or storage mapping).
        ///
        /// # Arguments
        ///
        /// * `start_sector` - An integer representing the starting sector
        ///   from which the file sectors collection begins.
        ///
        /// # Returns
        ///
        /// * `Vec<i32>` - A vector containing the collected sectors as integers.
        ///
        /// # Usage
        ///
        /// ```rust
        /// let file_sectors = storage.collect_file_sectors(5);
        /// println!("{:?}", file_sectors);
        /// ```
        ///
        /// # Errors
        ///
        /// This function may fail to collect sectors if the starting sector is invalid,
        /// corrupted, or if there is an I/O issue. Such cases should be handled in
        /// the implementation.
        ///
        /// # Examples
        ///
        /// ```rust
        /// // Assuming a file starts at sector 3
        /// let sectors = storage.collect_file_sectors(3);
        /// assert_eq!(sectors, vec![3, 4, 5, 6]);
        /// ```
        ///
        /// Note: The exact set of sectors returned depends on the structure of the
        /// storage medium or data representation.
        fn collect_file_sectors(&mut self, start_sector: i32) -> Vec<i32>;
        /// Constructs a debug string representation for a given list of file sectors.
        ///
        /// This function takes a reference to a slice of integers (`file_sectors`), where each integer
        /// represents a sector. The function processes the slice and builds a string that can be used
        /// for debugging purposes, such as logging or inspecting the structure of the file sectors.
        ///
        /// # Parameters
        /// - `file_sectors`: A reference to a slice of `i32` values representing file sectors.
        ///
        /// # Returns
        /// A `String` containing the debug representation of the provided file sectors.
        ///
        /// # Example
        /// ```
        /// let mut instance = YourStruct::new();
        /// let sectors = vec![1, 2, 3, 4];
        /// let debug_string = instance.build_sector_debug_string(&sectors);
        /// println!("{}", debug_string);
        /// // Output: (example) "Sectors: [1, 2, 3, 4]"
        /// ```
        ///
        /// # Note
        /// Ensure the list of sectors passed to this method is valid and relevant to the context in
        /// which the debug string will be used.
        fn build_sector_debug_string(&mut self, file_sectors: &[i32]) -> String;
        /// Processes the given file sectors and returns a PRG (Pseudo-Random Generator) object.
        ///
        /// # Parameters
        /// - `sectors`: A slice of i32 values representing the file sectors to be processed.
        ///
        /// # Returns
        /// - `PRG`: Returns a pseudo-random generator object derived from the provided sectors.
        ///
        /// # Remarks
        /// This function is designed to handle the processing of file sectors
        /// and generate a meaningful PRG object for further use. The implementation assumes
        /// that the provided sectors are valid and in the appropriate format.
        ///
        /// # Example
        /// ```
        /// let sectors = vec![10, 20, 30, 40];
        /// let prg = some_instance.process_file_sectors(&sectors);
        /// ```
        fn process_file_sectors(&self, sectors: &[i32]) -> PRG;
        /// Finds the parent sector of the given sector.
        ///
        /// This method takes an integer `sector` as input and attempts to find its
        /// parent sector. The parent sector is typically a sector that is hierarchically
        /// above the given sector and may contain information or structure linking to it.
        ///
        /// # Parameters
        /// - `sector` (i32): The sector for which the parent is to be determined.
        ///
        /// # Returns
        /// - `Option<i32>`:
        ///   - `Some(i32)` containing the parent sector if found.
        ///   - `None` if there is no parent sector or if the parent cannot be determined.
        ///
        /// # Examples
        /// ```
        /// let mut structure = ...; // Initialize a relevant data structure
        /// if let Some(parent) = structure.find_parent_sector(5) {
        ///     println!("Parent sector: {}", parent);
        /// } else {
        ///     println!("No parent sector found.");
        /// }
        /// ```
        ///
        /// # Notes
        /// - This method may modify internal data structures used to track sectors.
        /// - Ensure to handle the `None` case to avoid potential runtime issues.
        fn find_parent_sector(&mut self, sector: i32) -> Option<i32>;
    }

    // derive traits
    #[derive(Debug, PartialEq, Eq, PackedSize, EncodeLE, DecodeLE)]
    struct DirectoryEntry {
        attribute: u8,
        track: u8,
        sector: u8,
        data: [u8; 16],
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

    pub(crate) struct FileEntry {
        name: String,
        track: u8,
        sector: u8,
        attribute: u8,
    }
    const HEX_CHAR_LOOKUP: [char; 16] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
    ];

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
        track: u8,
        sector: u8,
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
        track: u8,
        sector: u8,
        data: Vec<u8>,
    }

    /// `D64` is a structure representing the state and components of a D64 disk image.
    /// It is intended for internal crate usage (`pub(crate)`).
    ///
    /// # Fields
    ///
    /// * `data` (`Vec<u8>`):
    ///   The raw binary data of the disk image.
    ///
    /// * `disk_size` (`usize`):
    ///   The total size of the disk in bytes.
    ///
    /// * `sectors` (`Vec<Sector>`):
    ///   A collection of `Sector` objects representing the individual sectors of the disk.
    ///
    /// * `cumulated_sectors` (`Vec<i32>`):
    ///   A vector containing cumulative sector information, which could be used
    ///   for tracking or indexing purposes.
    ///
    /// * `debug_output` (`bool`):
    ///   A flag used for enabling or disabling debug output while processing or accessing
    ///   data in the disk image.
    ///
    /// This struct may provide the foundation for examining, manipulating, or interacting
    /// with D64 disk images. The precise functionality depends on the implementation details
    /// of associated methods and the `Sector` struct.
    pub(crate) struct D64 {
        data: Vec<u8>,
        disk_size: usize,
        sectors: Vec<Sector>,
        cumulated_sectors: Vec<i32>,
        debug_output: bool,
    }

    const DIRECTORY_ENTRY_SIZE: usize = 32;
    /// A constant representing the directory track in a file storage or disk management system.
    ///
    /// The value `18` is typically associated with the directory track location
    /// on certain disk formats, especially in older file systems like those
    /// used in vintage computing systems (e.g., Commodore 1541 disk drive).
    ///
    /// This constant can be used in applications or utilities that need to
    /// interface with or emulate such systems.
    ///
    /// # Example
    /// ```rust
    /// // Usage of DIRECTORY_TRACK constant
    /// println!("Directory Track is: {}", DIRECTORY_TRACK);
    /// // Output: Directory Track is: 18
    /// ```
    ///
    /// Note:
    /// - The value `18` is specific to certain legacy systems and may not
    ///   apply universally in modern contexts.
    const DIRECTORY_TRACK: u8 = 18;
    /// Constants
    ///
    /// `DIRECTORY_SECTOR`:
    /// Represents the directory sector index or identifier in a storage system.
    /// This value typically refers to the first sector (sector 0) in a directory or filesystem structure.
    ///
    /// - Type: `i32`
    /// - Value: `0`
    const DIRECTORY_SECTOR: u8 = 0;
    /// A constant representing the size of a disk in bytes.
    ///
    /// This value is set to `174848` bytes, which may correspond to a specific
    /// disk format or size used in the application.
    ///
    /// # Usage
    /// The `DISK_SIZE` constant can be used wherever the total size of the disk
    /// needs to be referenced within the program. It is particularly useful
    /// for defining buffers, validating data sizes, or performing calculations
    /// related to storage capacity.
    ///
    /// # Example
    /// ```
    /// let buffer = [0u8; DISK_SIZE];
    /// println!("Buffer has been initialized with {} bytes.", DISK_SIZE);
    /// ```
    ///
    /// # Note
    /// The value of `DISK_SIZE` is specific to the context of this program and should
    /// not be assumed to apply universally to all disk sizes.
    const DISK_SIZE: usize = 174848;

    /// Represents the maximum valid sector number for a given application.
    ///
    /// This constant defines the upper boundary for the sector range.
    /// Any sector value above this will be considered invalid or out of range.
    ///
    /// # Value
    /// - `683` is the maximum valid sector number.
    ///
    /// # Usage
    /// Use this constant to validate sector inputs or enforce sector range constraints.
    ///
    /// # Example
    /// ```
    ///
    const MAX_VALID_SECTOR: i32 = 683;
    /// The constant `SECTOR_SIZE` represents the size of a single sector in bytes.
    ///
    /// This value is typically used in contexts where data storage or memory is
    /// organized into fixed-size units, referred to as "sectors."
    /// Many storage devices, such as hard drives or solid-state drives (
    const SECTOR_SIZE: usize = 256;
    /// A constant representing the size of the header in bytes.
    ///
    /// This constant is used to define the size of the header portion
    /// in a data structure or protocol. It indicates how many bytes
    /// are reserved for the header.
    ///
    /// # Value
    /// - `HEADER_SIZE`: 2 bytes.
    ///
    /// # Example
    /// ```
    /// const HEADER_SIZE: usize = 2;
    ///
    /// // Use HEADER_SIZE to define header-related
    const HEADER_SIZE: usize = 2;
    /// Constant `MAX_DATA_SIZE`
    ///
    /// Represents the maximum amount of data (in bytes) that can be stored in a single sector
    /// after accounting for the size of the header. The value is derived by subtracting the
    /// `HEADER_SIZE` from the total `SECTOR_SIZE`.
    ///
    /// # Constants:
    /// - `SECTOR_SIZE`: The total size of a sector in bytes.
    /// - `HEADER
    const MAX_DATA_SIZE: usize = SECTOR_SIZE - HEADER_SIZE;

    impl DiskParser for D64 {
        /// Parses the disk data into sectors and populates the `sectors` field.
        ///
        /// This function divides the disk data into smaller chunks (sectors) by iterating
        /// over the disk data in steps of `SECTOR_SIZE`. Each sector consists of:
        /// - A track identifier (1 byte)
        /// - A sector identifier (1 byte)
        /// - A block of data of maximum `MAX_DATA_SIZE` or the remaining data size (whichever is smaller)
        ///
        /// The function processes the disk data up to the end minus the `HEADER_SIZE`. For each chunk,
        /// it creates a `Sector` object containing its track, sector, and data,
        /// and adds it to the `sectors` vector in the struct.
        ///
        /// # Returns
        /// * `true` - Always returns `true` after processing the disk data.
        ///
        /// # Panics
        /// The function may panic if:
        /// - The `i + HEADER_SIZE + data_size` index goes out of bounds, which could happen if the
        ///   `disk_size` or `data` is not consistent.
        /// - The `self.data` access fails due to incorrect size.
        ///
        /// # Notes
        /// Make sure `self.disk_size` and `self.data` are correctly initialized
        /// before calling this method.
        fn parse_sectors(&mut self) -> bool {
            for i in (0..self.disk_size - HEADER_SIZE).step_by(SECTOR_SIZE) {
                if i + HEADER_SIZE >= self.disk_size {
                    break;
                }

                let remaining_size = self.disk_size - (i + HEADER_SIZE);
                let data_size = std::cmp::min(MAX_DATA_SIZE, remaining_size);

                let sector = Sector {
                    track: self.data[i],
                    sector: self.data[i + 1],
                    data: self.data[i + HEADER_SIZE..i + HEADER_SIZE + data_size].to_vec(),
                };

                self.sectors.push(sector);
            }

            true
        }

        /// Sets the debug output mode for the current instance.
        ///
        /// # Parameters
        /// - `debug_output`: A boolean value indicating whether debug output should be enabled (`true`)
        ///                   or disabled (`false`).
        ///
        /// # Example
        /// ```
        /// let mut instance = SomeStruct::new();
        /// instance.set_debug_output(true); // Enables debug output
        /// instance.set_debug_output(false); // Disables debug output
        /// ```
        ///
        /// # Notes
        /// This method modifies the `debug_output` field within the instance,
        /// which controls whether debug-related information is printed or logged.
        fn set_debug_output(&mut self, debug_output: bool) -> &mut Self {
            self.debug_output = debug_output;
            self
        }

        /// Parses the contents of the `data` field and validates it against certain conditions.
        ///
        /// # Functionality
        /// 1. Checks whether the `data` field is empty. If it is, the function returns `false`.
        /// 2. Validates if the length of `data` matches the expected `disk_size`. If it does not,
        ///    an error message is printed ("Invalid file size"), and the function returns `false`.
        /// 3. Calls the `parse_sectors` method to handle further parsing logic.
        ///
        /// # Returns
        /// - `true` if the parsing process completes successfully via `parse_sectors`.
        /// - `false` if any of the following conditions fail:
        ///   - `data` is empty.
        ///   - The length of `data` does not match `disk_size`.
        ///
        /// # Notes
        /// Ensure that `parse_sectors` is implemented correctly for this function to work seamlessly.
        fn parse_disk(&mut self) -> bool {
            if self.data.is_empty() {
                return false;
            }

            if self.data.len() != self.disk_size {
                println!("Invalid file size");
                return false;
            }

            self.parse_sectors()
        }

        /// Parses a file from the given path and processes its content.
        ///
        /// # Parameters
        /// - `path`: A string slice that holds the file path to be parsed.
        ///
        /// # Returns
        /// - Returns `true` if the file parsing logic is successfully executed based on the inner implementation,
        ///   otherwise returns `false`.
        ///
        /// # Behavior
        /// - Attempts to load the file specified by the input path via the `load_file` method.
        /// - If the file is successfully loaded (indicated by an `Ok` result), invokes the `parse` method to process the data.
        /// - Currently, always returns `false` regardless of the result.
        ///
        /// # Notes
        /// - Ensure that the `load_file` and `parse` methods are properly implemented and integrated with this function for expected behavior.
        /// - The return value is not dependent on the success of the `parse` or `load_file` operation as it currently always returns `false`.
        ///
        /// # Example
        /// ```
        /// let mut parser = MyParser::new();
        /// let result = parser.parse_file("example.txt");
        /// assert_eq!(result, false); // Due to hardcoded false return
        /// ```
        fn parse_file(&mut self, path: &str) -> bool {
            if let Ok(()) = self.load_file(path) {
                return self.parse_disk();
            }
            false
        }

        /// Parses data from a given byte buffer and updates the internal state.
        ///
        /// This function takes a slice of bytes, copies the contents
        /// into the internal `data` variable, and then calls the `parse_disk`
        /// method to handle further parsing or processing.
        ///
        /// # Arguments
        ///
        /// * `buffer` - A slice of bytes (`&[u8]`) that contains the data to be parsed.
        ///
        /// # Returns
        ///
        /// * `bool` - The function is expected to return a boolean value indicating
        ///            the success or failure of the operation. However, the current
        ///            implementation does not explicitly return a value, which may
        ///            cause a compilation error.
        ///
        /// # Note
        ///
        /// Ensure that the function has the correct return value (`true` or `false`)
        /// to match the expected return type (`bool`). The current implementation
        /// is missing an explicit return statement.
        ///
        /// # Example
        ///
        /// ```
        /// let mut obj = YourStruct::new();
        /// let buffer = vec![1, 2, 3, 4];
        /// let success = obj.parse_from_buffer(&buffer);
        /// ```
        fn parse_from_buffer(&mut self, buffer: &[u8]) -> bool {
            self.data = buffer.to_vec();
            self.parse_disk()
        }

        /// Returns the number of sectors.
        ///
        /// This method calculates and returns the total count of sectors currently present
        /// within the `self.sectors` collection.
        ///
        /// # Returns
        /// * `usize` - The number of sectors in the `self.sectors` collection.
        ///
        /// # Examples
        /// ```
        /// let instance = MyStruct { sectors: vec![Sector::new(), Sector::new()] };
        /// let count = instance.get_sector_count();
        /// assert_eq!(count, 2);
        /// ```
        fn get_sector_count(&self) -> usize {
            self.sectors.len()
        }

        ///
        /// Retrieves a specified sector from the internal sector storage.
        ///
        /// # Parameters
        /// - `sector`: An integer representing the sector index to retrieve.
        ///
        /// # Returns
        /// - `Some(Sector)`: If the `sector` index is valid and within bounds.
        /// - `None`: If the `sector` index exceeds the defined `MAX_VALID_SECTOR`.
        ///
        /// # Behavior
        /// - The function checks if the provided `sector` index is greater than
        ///   `MAX_VALID_SECTOR`. If so, it immediately returns `None`.
        /// - If the `sector` index is valid, it clones the specified sector
        ///   from the internal storage and returns it wrapped in `Some`.
        ///
        /// # Panics
        /// - May panic if `sector` is negative or if the `sector` index
        ///   causes an out-of-bounds access on the `self.sectors` array.
        ///
        /// # Example
        /// ```
        /// let sector = object.get_sector(5);
        /// if let Some(s) = sector {
        ///     println!("Retrieved sector: {:?}", s);
        /// } else {
        ///     println!("Sector is invalid or out of bounds.");
        /// }
        /// ```
        fn get_sector(&self, sector: i32) -> Option<Sector> {
            if sector > MAX_VALID_SECTOR {
                return None;
            }

            let copy = self.sectors[sector as usize].clone();
            Some(copy)
        }

        /// Creates a new instance of the struct, initializing its fields to their default values.
        ///
        /// # Returns
        /// A newly constructed instance of the struct with the following initialized fields:
        /// - `data`: An empty `Vec` for storing data.
        /// - `disk_size`: A predefined size of the disk set to `174848`.
        /// - `sectors`: An empty `Vec` representing sectors associated with the disk.
        /// - `sector_map`: A `Vec` initialized with predefined sector identifiers, which are a sequence
        ///   of numbers that likely define the layout or structure of the sectors.
        ///
        /// # Example
        /// ```
        /// let instance = YourStruct::new();
        /// assert_eq!(instance.data.len(), 0);
        /// assert_eq!(instance.disk_size, 174848);
        /// assert_eq!(instance.sectors.len(), 0);
        /// assert_eq!(instance.sector_map.len(), 35);
        /// ```
        fn new() -> Self {
            let sector_map = vec![
                21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 19, 19, 19, 19,
                19, 19, 19, 18, 18, 18, 18, 18, 18, 17, 17, 17, 17, 17,
            ];

            // Calculate cumulated_sectors before creating the struct
            let cumulated_sectors = sector_map
                .iter()
                .scan(0, |acc, &x| {
                    *acc += x;
                    Some(*acc)
                })
                .collect();

            Self {
                data: Vec::new(),
                disk_size: DISK_SIZE,
                sectors: Vec::new(),
                cumulated_sectors,
                debug_output: false,
            }
        }

        ///
        /// Loads the contents of a file into the `data` field of the structure.
        ///
        /// # Arguments
        ///
        /// * `path` - A string slice that holds the file path to be read.
        ///
        /// # Returns
        ///
        /// * `std::io::Result<()>` - Returns `Ok(())` if the file is successfully loaded
        ///   into the `data` field. Returns an `Err` variant if there is an error reading
        ///   the file.
        ///
        /// # Errors
        ///
        /// This function will return an error in the following scenarios:
        /// * The specified file path does not exist.
        /// * The program does not have enough permissions to read the file.
        /// * Any other I/O error occurs during the reading of the file.
        ///
        /// # Examples
        ///
        /// ```
        /// let mut loader = YourStruct { data: Vec::new() };
        /// match loader.load_file("example.txt") {
        ///     Ok(_) => println!("File loaded successfully."),
        ///     Err(e) => eprintln!("Error loading file: {:?}", e),
        /// }
        /// ```
        fn load_file(&mut self, path: &str) -> std::io::Result<()> {
            self.data = std::fs::read(path)?;
            Ok(())
        }

        /// Retrieves track information for a given raw sector.
        ///
        /// This function calculates the track and sector information based on a
        /// provided raw sector value. If the sector value exceeds the maximum valid
        /// sector (`MAX_VALID_SECTOR`), it returns `None`.
        ///
        /// The calculation is performed using cumulative sector boundaries stored in
        /// the `self.cumulated_sectors` array. It determines the corresponding track
        /// number and adjusts the sector value relative to the track's start.
        ///
        /// # Parameters
        /// - `sector`: The raw sector value as an integer.
        ///
        /// # Returns
        /// - `Option<TrackInfo>`:
        ///   - `Some(TrackInfo)`: Contains the resolved track number and relative
        ///     sector within that track.
        ///   - `None`: When the `sector` value is greater than `MAX_VALID_SECTOR`.
        ///
        /// # Example
        /// ```rust
        /// // Assuming a sector within bounds and properly initialized `self` object:
        /// let sector = 1200;
        /// let track_info = self.get_track_info_for_raw_sector(sector);
        ///
        /// match track_info {
        ///     Some(info) => println!("Track: {}, Sector: {}", info.track, info.sector),
        ///     None => println!("Invalid sector."),
        /// }
        /// ```
        ///
        /// # Notes
        /// - Track numbers begin at 1.
        /// - If the sector value falls in the first track (or no boundary is exceeded),
        ///   the calculation adjusts the `sector` directly.
        ///
        /// # Errors
        /// - Returns `None` directly if the `sector` exceeds `MAX_VALID_SECTOR`.
        ///
        /// # Dependencies
        /// - `TrackInfo`: A data structure that must exist and be accessible in the
        ///   same context to store the track number and relative sector.
        ///
        /// # Constraints
        /// - The `self.cumulated_sectors` array must be initialized and properly
        ///   populated with cumulative sector counts for this function to work
        ///   correctly.
        fn get_track_info_for_raw_sector(&self, sector: i32) -> Option<TrackInfo> {
            if sector > MAX_VALID_SECTOR {
                return None;
            }

            let track_number = self
                .cumulated_sectors
                .iter()
                .enumerate()
                .find(|&(_, &max_sector)| sector + 1 <= max_sector)
                .map(|(track_num, _)| (track_num + 1) as i32)
                .unwrap();

            if track_number >= 2 {
                Some(TrackInfo {
                    track: track_number as u8,
                    sector: (sector - self.cumulated_sectors[(track_number - 2) as usize]) as u8,
                })
            } else {
                Some(TrackInfo {
                    track: track_number as u8,
                    sector: sector as u8,
                })
            }
        }

        /// Retrieves the sector number for a given `TrackInfo`.
        ///
        /// This method calculates the sector number for the provided `TrackInfo` based on the track
        /// and sector values. It uses precomputed cumulative sectors to derive the correct sector
        /// number for tracks greater than 1. If the calculated sector exceeds the `MAX_VALID_SECTOR`
        /// limit, the function returns `None`.
        ///
        /// # Parameters
        /// - `track`: A `TrackInfo` struct that contains the track and sector information.
        ///
        /// # Returns
        /// - `Some(i32)` containing the calculated sector number if it is valid.
        /// - `None` if the calculated sector exceeds the maximum valid sector value (`MAX_VALID_SECTOR`).
        ///
        /// # Notes
        /// - For `track.track <= 1`, the function directly returns the `track.sector` without
        ///   any computation.
        /// - For `track.track > 1`, the raw sector is computed using the cumulative sectors and
        ///   the provided `track.sector` value.
        ///
        /// # Example
        /// ```
        /// let track_info = TrackInfo { track: 3, sector: 5 };
        /// let sector = your_struct.get_sector_for_track_info(track_info);
        /// assert_eq!(sector, Some(15)); // Example value if within valid range
        ///
        /// let invalid_track_info = TrackInfo { track: 10, sector: 9999 };
        /// let invalid_sector = your_struct.get_sector_for_track_info(invalid_track_info);
        /// assert_eq!(invalid_sector, None); // Exceeds MAX_VALID_SECTOR.
        /// ```
        fn get_sector_for_track_info(&self, track: &TrackInfo) -> Option<i32> {
            if track.track <= 1 {
                return Some(track.sector as i32);
            }

            let raw_sector =
                self.cumulated_sectors[(track.track - 2) as usize] + (track.sector as i32);

            if raw_sector > MAX_VALID_SECTOR {
                return None;
            }

            Some(raw_sector)
        }

        ///
        /// This method scans and retrieves all files stored in the system by analyzing sectors.
        /// It identifies valid file entries, tracks their parent sectors, assembles the files,
        /// and returns them in a vector.
        ///
        /// # Returns
        /// - `Option<Vec<PRG>>`: Returns `Some(Vec<PRG>)` containing all identified files if any files are found,
        ///   otherwise returns `None`.
        ///
        /// # Procedure
        /// - Iterates through all sectors in the system using their indices.
        /// - Skips sectors that are not file entries (e.g., those not located on track 0 or within sector 0).
        /// - Traces parent sectors for each file to collect all related sectors, forming a complete file structure.
        /// - Builds a `PRG` file object by assembling data from the identified sectors while maintaining the order.
        /// - Each file entry is processed and added to the `files` vector.
        ///
        /// # Side Effects
        /// - Outputs debug information, including track and sector details for EOF detection,
        ///   and the sectors used for each file, to the console using `println!`.
        /// - Prints the total number of files found.
        ///
        /// # Example
        /// ```rust
        /// if let Some(files) = system.find_all_files() {
        ///     for file in files {
        ///         println!("File name: {}, File size: {}", file.name, file.data.len());
        ///     }
        /// } else {
        ///     println!("No files found");
        /// }
        /// ```
        ///
        /// # Notes
        /// - The method depends on several helper methods like `get_sector`, `get_sector_count`,
        ///   `get_track_info_for_raw_sector`, and `find_parent_sector`.
        /// - The `process_file_sectors` method is used to finalize the processed file.
        ///
        /// # Assumptions
        /// - Sectors are structured in a way that allows reconstructing the files based on parent sector relationships.
        /// - Failures to retrieve a sector's information (e.g., `None` return from `get_sector`) are silently skipped.
        ///
        /// # Limitations
        /// - The method assumes all files are located on track 0 and excludes specific sectors (sector 0).
        /// - The file type and name default to "unknown" before processing.
        ///
        /// # Dependencies
        /// - `PRG`: Struct representing a file, containing:
        ///     - `data: Vec<u8>`: File data extracted from relevant sectors.
        ///     - `name: String`: File name (initially set to "unknown").
        ///
        fn find_all_files(&mut self) -> Option<Vec<PRG>> {
            let mut files: Vec<PRG> = Vec::new();

            for sector_index in 0..self.get_sector_count() as i32 {
                let Some(sector_info) = self.get_sector(sector_index) else {
                    continue;
                };

                // Skip if not a file entry (files are on track 0, excluding sector 0)
                if sector_info.track != 0 || sector_info.sector == 0 {
                    continue;
                }

                if let Some(file) = self.process_single_file(sector_index) {
                    files.push(file);
                }
            }

            if self.debug_output {
                println!("Total files found: {}", files.len());
            }

            if files.is_empty() { None } else { Some(files) }
        }

        /// Lists all directories by finding all files and processing those that are directories.
        ///
        /// This method retrieves all files using the `find_all_files` function. Once the files are obtained,
        /// it iterates through the list and checks if each file represents a directory. If a file is a
        /// directory, the `process_directory_file` method is called for further processing.
        ///
        /// # Behavior
        /// - If no files are found, the method does nothing.
        /// - Processes only files marked as directories.
        ///
        /// # Assumptions
        /// - The `find_all_files` method returns an `Option<Vec<File>>` where `File` is a structure that
        ///   contains a `directory` field.
        /// - The `process_directory_file` method handles the processing of directory files.
        ///
        /// # Example
        /// ```
        /// instance.list_directory();
        /// ```
        /// Make sure `instance` has already implemented the required methods and structures for this call
        /// to work.
        fn parse_directory(&mut self) -> Option<Vec<FileEntry>> {
            if let Some(files) = self.find_all_files() {
                for file in files {
                    if file.directory {
                        return self.process_directory_file(&file);
                    }
                }
            }

            None
        }

        fn vec_u8to_hex_string(&mut self, array: &[u8]) -> String {
            let mut hex_string = String::new();
            for byte in array {
                hex_string.push(HEX_CHAR_LOOKUP[(byte >> 4) as usize]);
                hex_string.push(HEX_CHAR_LOOKUP[(byte & 0xF) as usize]);
            }
            hex_string
        }

        /// Retrieves all files in the current context, associating them with corresponding directory entries.
        ///
        /// This method performs the following steps:
        /// 1. Finds all files by calling `find_all_files`.
        /// 2. Parses the directory data via `parse_directory`.
        /// 3. Iterates through the files and directory entries. If a match is found
        ///    between a file's `track` and `sector` and a directory entry's `track` and `sector`,
        ///    it assigns the corresponding directory entry's `name` to the file.
        ///
        /// # Returns
        /// - `Some(Vec<PRG>)`: A vector of `PRG` objects with names assigned from the directory if
        ///   the files and directory are successfully found and matched.
        /// - `None`: If either files or directory data is missing or cannot be processed.
        ///
        /// # Note
        /// - The method assumes that `self.find_all_files()` and `self.parse_directory()` return
        ///   usable data for processing.
        /// - Modifies the names of the files in-place based on the matching directory entries.
        ///
        /// # Example Usage
        /// ```rust
        /// let files = instance.get_all_files();
        /// if let Some(files) = files {
        ///     for file in files {
        ///         println!("File name: {}", file.name);
        ///     }
        /// } else {
        ///     println!("No files found or directory parsing failed.");
        /// }
        /// ```
        fn get_all_files(&mut self) -> Option<Vec<PRG>> {
            if let Some(mut files) = self.find_all_files() {
                if let Some(directory) = self.parse_directory() {
                    for file in files.iter_mut() {
                        for entry in &directory {
                            if entry.track == file.track && entry.sector == file.sector {
                                file.name = entry.name.clone();
                                break;
                            }
                        }
                        file.hashes.md5 =
                            self.vec_u8to_hex_string(&Sha1::digest(file.data.clone()));
                        file.hashes.sha1 =
                            self.vec_u8to_hex_string(&md5::compute(file.data.clone()).to_vec());
                        file.hashes.sha256 =
                            sha256::digest(file.data.clone()).to_uppercase().to_string();
                    }

                    return Some(files);
                }
            }

            None
        }

        /// Processes a directory file and extracts file entries.
        ///
        /// This function processes a given directory file (`PRG` format) to parse
        /// its contents and extract individual file entries into a structured format.
        /// It skips the Block Availability Map (BAM) sector and parses all other sectors
        /// of the directory file. Each sector is broken down into individual entries
        /// which are validated and collected.
        ///
        /// # Parameters
        /// - `file`: A reference to the directory file (`PRG`) to process. This file is
        ///   assumed to contain a sequence of sectors, where each sector may contain up
        ///   to a fixed number of directory entries.
        ///
        /// # Returns
        /// - `Option<Vec<FileEntry>>`: A vector of parsed `FileEntry` objects wrapped in
        ///   `Some` if any entries are successfully parsed. Returns `None` if no valid
        ///   entries are found in the directory file.
        ///
        /// # Details
        /// - Each sector can contain a fixed number of entries defined by `ENTRIES_PER_SECTOR`.
        /// - The size of each readable entry is determined by `DIRECTORY_ENTRY_SIZE`.
        /// - A sector's data is calculated as:
        ///   `SECTOR_SIZE - HEADER_SIZE`, where `SECTOR_SIZE` is the total size of one
        ///   sector and `HEADER_SIZE` is the size taken up by the sector header.
        /// - Sectors are processed sequentially, starting from the second sector
        ///   (skipping the BAM sector).
        /// - For each sector, the function calculates the offset and extracts directory
        ///   entries. Entries that go beyond the end of the file data buffer are ignored.
        ///
        /// # Notes
        /// - The `parse_and_print_directory_entry` method is used to parse individual entries.
        ///   It is assumed that this function handles entry parsing and any corresponding
        ///   logging or printing.
        /// - The function ensures no out-of-bounds reads are performed on the input `file.data`.
        ///
        /// # Constants
        /// - `ENTRIES_PER_SECTOR`: The number of directory entries per sector.
        /// - `SECTOR_SIZE`: The size of a single sector in bytes.
        /// - `HEADER_SIZE`: The size of a sector's header in bytes.
        /// - `DIRECTORY_ENTRY_SIZE`: The size of an individual directory entry in bytes.
        ///
        /// # Example
        /// ```rust
        /// let prg_file = PRG { data: vec![/* raw data */] };
        /// let directory_entries = processor.process_directory_file(&prg_file);
        ///
        /// if let Some(entries) = directory_entries {
        ///     for entry in entries {
        ///         println!("{:?}", entry);
        ///     }
        /// } else {
        ///     println!("No valid entries found.");
        /// }
        /// ```
        fn process_directory_file(&self, file: &PRG) -> Option<Vec<FileEntry>> {
            const ENTRIES_PER_SECTOR: usize = 8;
            let sector_data_size = SECTOR_SIZE - HEADER_SIZE;
            let sector_count = file.data.len() / sector_data_size;

            let mut file_entries: Vec<FileEntry> = Vec::new();
            // make sure we skip the BAM
            for sector_index in 1..sector_count {
                let sector_start = sector_index * sector_data_size;

                for entry_index in 0..ENTRIES_PER_SECTOR {
                    let entry_offset = sector_start + (entry_index * DIRECTORY_ENTRY_SIZE);

                    if entry_offset + DIRECTORY_ENTRY_SIZE > file.data.len() {
                        continue;
                    }

                    file_entries.push(self.parse_and_print_directory_entry(
                        &file.data[entry_offset..entry_offset + DIRECTORY_ENTRY_SIZE],
                    ));
                }
            }

            if file_entries.is_empty() {
                None
            } else {
                Some(file_entries)
            }
        }

        /// Parses raw bytes representing a directory entry, prints its details, and returns a `FileEntry` structure.
        ///
        /// # Arguments
        ///
        /// * `entry_bytes` - A slice of bytes representing the directory entry in the specified filesystem.
        ///
        /// # Returns
        ///
        /// * `FileEntry` - A structure containing the parsed file name, track, and sector values of the directory entry.
        ///
        /// # Example
        ///
        /// ```
        /// let entry_bytes = vec![/* raw directory entry bytes */];
        /// let file_entry = your_instance.parse_and_print_directory_entry(&entry_bytes);
        /// ```
        ///
        /// # Implementation Details
        ///
        /// * The method decodes the raw `entry_bytes` into a `DirectoryEntry` structure using the `decode_from_le_bytes` method.
        /// * Converts the entry `data` field into a `PetsciiString` representation.
        /// * Prints detailed information about the directory entry, including its attributes, track, sector, data, and name in PETSCII format.
        /// * Constructs and returns a `FileEntry` object containing the file name, track number, and sector number.
        ///
        /// # Printed Output
        ///
        /// Information printed includes:
        /// - Attribute of the directory entry (in hexadecimal format).
        /// - Track and sector numbers associated with the entry.
        /// - Raw data associated with the entry.
        /// - Directory name in a human-readable PETSCII string format.
        fn parse_and_print_directory_entry(&self, entry_bytes: &[u8]) -> FileEntry {
            let entry = DirectoryEntry::decode_from_le_bytes(entry_bytes);
            let petscii_name = petscii::PetsciiString::from(&entry.data);

            if self.debug_output {
                println!(
                    "attribute:{:2x} track:#{:2.2}:{:2.2}, {:x?} {}",
                    entry.attribute,
                    entry.track,
                    entry.sector,
                    entry.data,
                    petscii_name.to_string()
                );
            }

            FileEntry {
                name: petscii_name.to_string(),
                track: entry.track,
                sector: entry.sector,
                attribute: entry.attribute,
            }
        }

        /// Processes a single file based on the specified sector index.
        ///
        /// This function performs the following steps:
        /// 1. Collects all the sectors belonging to the file starting at the given `sector_index`.
        /// 2. Builds a debug string containing sector information for diagnostic purposes.
        /// 3. Identifies the track and sector information for the first file sector.
        /// 4. Skips processing if the file resides in the directory track (track 18, sector 0).
        /// 5. Checks for end-of-file (EOF) by analyzing track information and logs it if detected.
        /// 6. Processes the collected file sectors, converting them into a `PRG` object and generates a debug log.
        ///
        /// ### Parameters
        /// - `sector_index`: The index of the initial sector where the file starts.
        ///
        /// ### Returns
        /// - `Option<PRG>`:
        ///   - Returns `Some(PRG)` if the file was successfully processed.
        ///   - Returns `None` if the file resides in the directory track and was skipped.
        ///
        /// ### Side Effects
        /// - Logs sector details and EOF detection as debug information to the console.
        /// - Logs the sectors used by the file.
        ///
        /// ### Example
        /// ```rust
        /// let mut processor = FileProcessor::new();
        /// if let Some(file) = processor.process_single_file(5) {
        ///     println!("File processed successfully: {:?}", file);
        /// } else {
        ///     println!("File processing skipped.");
        /// }
        /// ```
        ///
        /// ### Notes
        /// - The function assumes that sectors are already valid raw sectors.
        /// - The directory track (track 18, sector 0) is ignored and will return `None`.
        ///
        /// ### Debug Output
        /// - Prints details on the sectors processed.
        /// - Notifies if EOF is detected, including track and sector information.
        fn process_single_file(&mut self, sector_index: i32) -> Option<PRG> {
            let file_sectors = self.collect_file_sectors(sector_index);
            let debug_output = self.build_sector_debug_string(&file_sectors);
            let mut file = self.process_file_sectors(&file_sectors);

            if file_sectors.is_empty() {
                return None;
            }

            let sector_info = self.get_track_info_for_raw_sector(file_sectors[0]).unwrap();
            if sector_info.track == DIRECTORY_TRACK && sector_info.sector == DIRECTORY_SECTOR {
                if self.debug_output {
                    println!("Directory");
                }

                file.directory = true;
            }

            file.track = sector_info.track;
            file.sector = sector_info.sector;

            // Get and print track information for EOF detection
            if self.debug_output {
                if let Some(track_info) = self.get_track_info_for_raw_sector(sector_index) {
                    println!(
                        "EOF detected - track #{:02}:{:02}",
                        track_info.track, track_info.sector
                    );
                }

                println!("file sectors used: {}", debug_output);
            }

            Some(file)
        }

        ///
        /// Collects and returns a list of file sectors starting from a given start sector.
        ///
        /// This method traces the sectors of a file by starting from the given `start_sector`
        /// and following the chain of parent sectors using the `find_parent_sector` method.
        /// The resulting list of sectors is returned in reverse order, beginning with the
        /// earliest sector in the chain and ending with the starting sector.
        ///
        /// # Arguments
        ///
        /// * `start_sector` - The starting sector of the file to be collected.
        ///
        /// # Returns
        ///
        /// A `Vec<i32>` containing the chain of file sectors in the proper order,
        /// where the earliest sector is the first element and the starting sector is the last.
        ///
        /// # Example
        ///
        /// ```
        /// // Assuming `self.find_parent_sector` resolves parent sectors correctly.
        /// let file_sectors = object.collect_file_sectors(10);
        /// println!("{:?}", file_sectors); // Output: [1, 5, 10] (example output based on sector chain)
        /// ```
        ///
        /// # Notes
        ///
        /// * It is assumed that `find_parent_sector` is implemented to return the parent sector
        ///   of a given sector or `None` if no parent sector exists.
        /// * The sequence of sectors is traced until a sector with no parent is found.
        ///
        fn collect_file_sectors(&mut self, start_sector: i32) -> Vec<i32> {
            let mut file_sectors = vec![start_sector];
            let mut current_sector = start_sector;

            // Trace parent sectors
            while let Some(parent_sector) = self.find_parent_sector(current_sector) {
                file_sectors.push(parent_sector);
                current_sector = parent_sector;
            }

            file_sectors.reverse();
            file_sectors
        }

        /// Builds a debug string for the provided list of file sectors.
        ///
        /// This function takes a slice of sector identifiers (`file_sectors`) and generates a string
        /// representing the corresponding track and sector information for each valid sector.
        /// Each track and sector pair is formatted as `TT:SS`, where `TT` is the track number
        /// and `SS` is the sector number, both zero-padded to two digits. The pairs are joined
        /// with a single space in the resulting string.
        ///
        /// Any sectors for which track and sector information cannot be retrieved are skipped.
        ///
        /// # Arguments
        ///
        /// * `file_sectors` - A slice of integers representing the sectors of a file to process.
        ///
        /// # Returns
        ///
        /// A `String` containing the formatted track and sector information for the valid sectors.
        ///
        /// # Example
        ///
        /// ```rust
        /// let file_sectors = vec![1, 2, 3, 4];
        /// let debug_string = build_sector_debug_string(&file_sectors);
        /// println!("{}", debug_string); // Outputs something like: "00:01 00:02 00:03"
        /// ```
        ///
        /// # Notes
        ///
        /// This function assumes the presence of a method `get_track_info_for_raw_sector`
        /// in the caller's context, which retrieves information about a given sector.
        /// If `get_track_info_for_raw_sector` returns `None` for a particular sector,
        /// that sector is omitted from the resulting string.
        fn build_sector_debug_string(&mut self, file_sectors: &[i32]) -> String {
            file_sectors
                .iter()
                .filter_map(|&sector| {
                    self.get_track_info_for_raw_sector(sector)
                        .map(|track_info| {
                            format!("{:02}:{:02}", track_info.track, track_info.sector)
                        })
                })
                .collect::<Vec<_>>()
                .join(" ")
        }

        /// Processes the given file sectors and constructs a `PRG` file structure containing the combined data from those sectors.
        ///
        /// # Parameters
        /// - `sectors`: A slice of integers representing the sector identifiers to process.
        ///
        /// # Returns
        /// A `PRG` structure with the following fields:
        /// - `data`: A `Vec<u8>` containing the combined data of all processed sectors, excluding the first two bytes of each sector's data.
        /// - `name`: A `String` initialized to `"unknown"`.
        ///
        /// # Behavior
        /// - Iterates over the provided sectors, retrieving each sector's data using the `get_sector` method (assumed to be defined in the `self` context).
        /// - For each valid sector data (i.e., when `get_sector` returns `Some(sector_data)`), appends the data starting from the third byte (skipping the first two bytes) into the `PRG`'s `data` field.
        /// - If a sector does not exist or cannot be retrieved (`get_sector` returns `None`), it is ignored.
        ///
        /// # Note
        /// - Assumes the `get_sector` method retrieves sector data as an object with a `data` field (of type `Vec<u8>`).
        /// - The resulting `PRG`'s `name` is set to `"unknown"`. Additional logic may be required if custom naming is desired.
        fn process_file_sectors(&self, sectors: &[i32]) -> PRG {
            let mut file = PRG {
                data: Vec::new(),
                name: String::from("unknown"),
                directory: false,
                track: 0,
                sector: 0,
                hashes: HASHES {
                    md5: String::from("unknown"),
                    sha1: String::from("unknown"),
                    sha256: String::from("unknown"),
                },
            };

            for &sector in sectors {
                if let Some(sector_data) = self.get_sector(sector) {
                    if sector_data.track != 0 {
                        file.data.append(&mut sector_data.data.to_vec());
                    } else {
                        if sector_data.sector > 0xFD {
                            file.data.append(&mut sector_data.data.to_vec());
                        } else {
                            file.data.append(
                                &mut sector_data.data[0..(sector_data.sector - 1) as usize]
                                    .to_vec(),
                            );
                        }
                    }
                }
            }
            file
        }

        /// Finds the parent sector for the given raw sector.
        ///
        /// This function looks for the parent sector corresponding to the provided
        /// raw sector index. If the sector number is greater than the maximum allowed
        /// (`MAX_VALID_SECTOR`), it will immediately return `None`. Otherwise, it fetches
        /// the track information for the provided raw sector and iterates over all
        /// sectors to find a matching parent sector, where the track and sector
        /// identifiers align.
        ///
        /// # Arguments
        ///
        /// * `sector` - An `i32` specifying the raw sector whose parent sector is to be found.
        ///
        /// # Returns
        ///
        /// * `Some(i32)` - Returns the raw sector index of the parent sector if a match is found.
        /// * `None` - If no parent sector is found or if the provided sector is invalid.
        ///
        /// # Side Effects
        ///
        /// * This function prints a debug message to the console when a parent sector is found,
        ///   displaying the track, sector, and raw sector index.
        ///
        /// # Errors
        ///
        /// * The function utilizes helper methods like `get_track_info_for_raw_sector` and
        ///   `get_sector` which may return `None`. In such cases, `None` is propagated
        ///   as a result.
        ///
        /// # Example
        ///
        /// ```rust
        /// let mut disk = Disk::new();
        /// let raw_sector = 42;
        /// if let Some(parent) = disk.find_parent_sector(raw_sector) {
        ///     println!("Parent sector index: {parent}");
        /// } else {
        ///     println!("No parent sector found.");
        /// }
        /// ```
        fn find_parent_sector(&mut self, sector: i32) -> Option<i32> {
            if sector > MAX_VALID_SECTOR {
                return None;
            }

            let track_info = self.get_track_info_for_raw_sector(sector)?;

            for i in 0..self.get_sector_count() as i32 {
                let raw_sector = self.get_sector(i)?;

                if raw_sector.track == track_info.track as u8
                    && raw_sector.sector == track_info.sector as u8
                {
                    return Some(i);
                }
            }
            None
        }
    }
}
