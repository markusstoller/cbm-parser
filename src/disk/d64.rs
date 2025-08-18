use crate::disk::cbm::{DiskParser, HASHES, PRG, Sector, TrackInfo};
use endian_codec::{DecodeLE, EncodeLE, PackedSize};
use sha1::{Digest, Sha1};

// derive traits
#[derive(Debug, PartialEq, Eq, PackedSize, EncodeLE, DecodeLE)]
struct DirectoryEntry {
    attribute: u8,
    track: u8,
    sector: u8,
    data: [u8; 16],
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
    type_info: Vec<TypeInfo>,
    identified_type_info: TypeInfo,
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
const DISK_SIZE_VARIANT_1: usize = 174848;
const ERROR_SIZE_VARIANT_1: usize = 683;
const DISK_SIZE_VARIANT_2: usize = 196608;
const ERROR_SIZE_VARIANT_2: usize = 768;
const DISK_SIZE_VARIANT_3: usize = 205312;
const ERROR_SIZE_VARIANT_3: usize = 802;

#[derive(Clone)]
struct TypeInfo {
    disk_size: usize,
    error_size: usize,
    error_information_present: bool,
}

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
    fn parse_file(&mut self, path: &str) -> Result<bool, String> {
        if let Ok(()) = load_file(self, path) {
            return parse_disk(self);
        }
        Err("failed to load file".to_string())
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
    fn parse_from_buffer(&mut self, buffer: &[u8]) -> Result<bool, String> {
        self.data = buffer.to_vec();
        parse_disk(self)
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
        if sector > get_max_valid_sector(self) {
            return None;
        }

        let copy = self.sectors[sector as usize].clone();
        Some(copy)
    }

    /// Creates a new instance of the struct initialized with default values.
    ///
    /// The `new` function initializes the following fields:
    ///
    /// - `sector_map`: A predefined vector representing the sizes of sectors in the system.
    /// - `type_info`: A vector of `TypeInfo` objects that provide metadata about disk variants,
    ///   including disk size, error size, and whether error information is present. There are three variants:
    ///     - Variant 1
    ///     - Variant 2
    ///     - Variant 3
    /// - `cumulated_sectors`: A vector that holds cumulative sums of sector sizes from `sector_map`.
    ///   This is calculated using an iterator with the `.scan()` function to maintain a running total.
    ///
    /// The returned struct has the following default field values:
    /// - `data`: An empty vector, intended to store specific disk data.
    /// - `disk_size`: Initialized to `DISK_SIZE_VARIANT_1`, which corresponds to the first disk type.
    /// - `sectors`: An empty vector, reserved for sector-specific data when needed.
    /// - `cumulated_sectors`: Pre-computed cumulative sector sizes from `sector_map`.
    /// - `debug_output`: A boolean flag initialized to `false`, used to toggle debugging information.
    /// - `type_info`: The predefined vector describing disk variants and relevant metadata.
    /// - `identified_type_info`: Set to the default `TypeInfo` of `Variant 1`, which corresponds to:
    ///   - Disk size: `DISK_SIZE_VARIANT_1`
    ///   - Error size: `ERROR_SIZE_VARIANT_1`
    ///   - Error information presence: `false`
    ///
    /// # Returns
    /// A new instance of the struct with all fields initialized using the predefined or calculated values.
    fn new() -> Self {
        let sector_map = vec![
            21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 19, 19, 19, 19, 19,
            19, 19, 18, 18, 18, 18, 18, 18, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17,
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
            disk_size: DISK_SIZE_VARIANT_1,
            sectors: Vec::new(),
            cumulated_sectors,
            debug_output: false,
            type_info: create_type_info(),
            identified_type_info: default_type_info(),
        }
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
        if sector > get_max_valid_sector(self) {
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

        let raw_sector = self.cumulated_sectors[(track.track - 2) as usize] + (track.sector as i32);

        if raw_sector > get_max_valid_sector(self) {
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

            if let Some(file) = process_single_file(self, sector_index) {
                files.push(file);
            }
        }

        if self.debug_output {
            println!("Total files found: {}", files.len());
        }

        if files.is_empty() { None } else { Some(files) }
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
            if let Some(directory) = parse_directory(self) {
                for file in files.iter_mut() {
                    for entry in &directory {
                        if entry.track == file.track && entry.sector == file.sector {
                            file.name = entry.name.clone();
                            break;
                        }
                    }
                    file.hashes.md5 = vec_u8to_hex_string(&Sha1::digest(file.data.clone()));
                    file.hashes.sha1 =
                        vec_u8to_hex_string(&md5::compute(file.data.clone()).to_vec());
                    file.hashes.sha256 =
                        sha256::digest(file.data.clone()).to_uppercase().to_string();
                }

                return Some(files);
            }
        }

        None
    }
}

/// Identifies the type of disk based on the associated `type_info` of the `D64` structure.
///
/// This function iterates over the cloned `type_info` of the `D64` instance and checks for
/// a matching disk type using the `check_disk_match` function. If a match is found, the
/// function immediately returns the result (a `bool`) from `check_disk_match`. If no match is
/// found during iteration, the function returns `false`.
///
/// # Parameters
/// - `self_ref: &mut D64`: A mutable reference to the `D64` instance whose `type_info` will be
///   used to identify the disk type. The mutation allows the possibility of modifying `self_ref`
///   as part of type matching, depending on the implementation of `check_disk_match`.
///
/// # Returns
/// - `bool`: Returns `true` if a matching disk type is found, otherwise `false`.
///
/// # Notes
/// - This function assumes that `self_ref.type_info` implements the `Clone` trait so it can be
///   safely cloned for iteration.
/// - The behavior of `check_disk_match` determines the conditions under which a match is considered found.
/// - The function will stop iteration as soon as a match is found, enhancing performance.
///
/// # Example (pseudocode, assuming valid usage):
/// ```rust
/// let mut d64_instance = D64::new();
/// let is_disk_type_identified = d64_instance.identify_disk_type();
/// if is_disk_type_identified {
///     println!("Disk type identified.");
/// } else {
///     println!("Failed to identify disk type.");
/// }
/// ```
fn identify_disk_type(self_ref: &mut D64) -> bool {
    let type_info_clone = self_ref.type_info.clone();
    for type_info in &type_info_clone {
        if let Some(found) = check_disk_match(self_ref, type_info) {
            return found;
        }
    }
    false
}

/// Sets up the identified type information for a given `D64` structure.
///
/// # Parameters
/// - `self_ref`: A mutable reference to the `D64` instance being configured.
/// - `type_info`: A reference to the `TypeInfo` structure containing information about the identified type.
/// - `has_error_info`: A boolean indicating whether error information is present for the identified type.
///
/// # Functionality
/// - Copies the `type_info` into the `identified_type_info` of the `D64` object.
/// - Updates the `error_information_present` field in `identified_type_info` based on the `has_error_info` parameter.
/// - Truncates the `data` of the `D64` object to match the `disk_size` specified in the provided `type_info`.
/// - Updates the `disk_size` field of the `D64` object using the `disk_size` from the provided `type_info`.
///
/// This function ensures that the `D64` object's identified type and its associated disk data are properly
/// synchronized with the provided type information and error state.
fn setup_identified_type(self_ref: &mut D64, type_info: &TypeInfo, has_error_info: bool) {
    self_ref.identified_type_info = type_info.clone();
    self_ref.identified_type_info.error_information_present = has_error_info;
    // temporarily truncate the error information
    self_ref
        .data
        .truncate(self_ref.identified_type_info.disk_size);
    self_ref.disk_size = self_ref.identified_type_info.disk_size;
}

/// Checks if the length of the disk's data matches the expected size defined in the `TypeInfo`.
///
/// This function compares the `self_ref`'s data length (`self_ref.data.len()`) with the `disk_size`
/// property of `type_info`. If the sizes match, it identifies the type using the
/// `setup_identified_type` function. Additionally, this function handles cases where the
/// data length matches the sum of `disk_size` and `error_size` and treats it as a valid
/// match with error tolerance.
///
/// # Parameters
/// - `self_ref`: A mutable reference to a `D64` instance for which the data is being checked.
/// - `type_info`: A reference to a `TypeInfo` struct containing the expected `disk_size`
///   and `error_size`.
///
/// # Returns
/// - `Some(true)` if `self_ref.data.len()` matches either the `disk_size` or `disk_size + error_size`.
/// - `None` if no match is found.
///
/// # Function Flow
/// 1. Retrieve the length of `self_ref.data`.
/// 2. Compare the data length to `type_info.disk_size`.
///    - If equal, call `setup_identified_type` with error flag as `false` and return `Some(true)`.
/// 3. Compare the data length to `type_info.disk_size + type_info.error_size`.
///    - If equal, call `setup_identified_type` with error flag as `true` and return `Some(true)`.
/// 4. If no match is found in either case, return `None`.
///
/// # Side Effects
/// Calls the `setup_identified_type` function, which modifies the state of `self_ref` to associate
/// it with the provided type information.
///
/// # Example
/// ```rust
/// let mut d64_instance = D64::new();
/// let type_info = TypeInfo { disk_size: 1024, error_size: 16 };
///
/// match check_disk_match(&mut d64_instance, &type_info) {
///     Some(true) => println!("Disk size matches type information."),
///     None => println!("Disk size does not match."),
/// }
/// ```
fn check_disk_match(self_ref: &mut D64, type_info: &TypeInfo) -> Option<bool> {
    let data_len = self_ref.data.len();

    if data_len == type_info.disk_size {
        setup_identified_type(self_ref, type_info, false);
        return Some(true);
    }

    if data_len == type_info.disk_size + type_info.error_size {
        setup_identified_type(self_ref, type_info, true);
        return Some(true);
    }

    None
}

/// Returns the maximum valid sector for the current D64 instance based on the associated
/// identified type information's error size.
///
/// # Parameters
///
/// * `self_ref` - A mutable reference to a `D64` instance.
///
/// # Returns
///
/// * `i32` - The maximum valid sector as an integer, derived by casting the
///   `error_size` from the `identified_type_info` field to an `i32`.
///
/// # Note
///
/// This function assumes that `identified_type_info.error_size` exists and holds a
/// meaningful value. Ensure this value is properly initialized before calling this method.
///
/// # Example
///
/// ```rust
/// let mut d64_instance = D64::new();
/// let max_sector = d64_instance.get_max_valid_sector();
/// println!("Max valid sector: {}", max_sector);
/// ```
fn get_max_valid_sector(self_ref: &D64) -> i32 {
    self_ref.identified_type_info.error_size as i32
}

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
fn parse_sectors(self_ref: &mut D64) -> bool {
    for i in (0..self_ref.disk_size - HEADER_SIZE).step_by(SECTOR_SIZE) {
        if i + HEADER_SIZE >= self_ref.disk_size {
            break;
        }

        let remaining_size = self_ref.disk_size - (i + HEADER_SIZE);
        let data_size = std::cmp::min(MAX_DATA_SIZE, remaining_size);

        let sector = Sector {
            track: self_ref.data[i],
            sector: self_ref.data[i + 1],
            data: self_ref.data[i + HEADER_SIZE..i + HEADER_SIZE + data_size].to_vec(),
        };

        self_ref.sectors.push(sector);
    }

    true
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
fn parse_disk(self_ref: &mut D64) -> Result<bool, String> {
    if self_ref.data.is_empty() {
        return Err("disk is empty".to_string());
    }

    if identify_disk_type(self_ref) == false {
        return Err("disk type is not identified".to_string());
    }

    parse_sectors(self_ref);
    Ok(true)
}
/// Converts a slice of `u8` values into a hexadecimal string representation.
///
/// This function takes in a slice of bytes (`&[u8]`) and converts each byte
/// into its corresponding two-character hexadecimal string representation.
/// It utilizes a lookup table, `HEX_CHAR_LOOKUP`, to map the lower and upper
/// nibble (4 bits) of each byte to their respective hexadecimal character.
///
/// # Arguments
///
/// * `array` - A slice of `u8` values representing the input byte array.
///
/// # Returns
///
/// A `String` containing the hexadecimal representation of the given byte slice.
///
/// # Example
///
/// ```rust
/// // Assuming HEX_CHAR_LOOKUP is defined as:
/// // const HEX_CHAR_LOOKUP: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7',
/// //                                    '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];
/// let byte_array = [0xDE, 0xAD, 0xBE, 0xEF];
/// let hex_string = vec_u8to_hex_string(&byte_array);
/// assert_eq!(hex_string, "DEADBEEF");
/// ```
///
/// # Notes
///
/// * Ensure that `HEX_CHAR_LOOKUP` is properly defined to include valid
///   hexadecimal characters (0-9 and A-F).
/// * The function assumes that the input slice is valid and does not perform
///   any additional checks for null or invalid data.
///
/// # Performance
///
/// This function iterates through each byte in the array and appends two hexadecimal
/// characters to the resulting `String`. It avoids allocations other than for the
/// final `String` result.
fn vec_u8to_hex_string(array: &[u8]) -> String {
    let mut hex_string = String::new();
    for byte in array {
        hex_string.push(HEX_CHAR_LOOKUP[(byte >> 4) as usize]);
        hex_string.push(HEX_CHAR_LOOKUP[(byte & 0xF) as usize]);
    }
    hex_string
}

/// Creates a default `TypeInfo` instance with preset configuration values.
///
/// # Description
/// This function returns a `TypeInfo` struct initialized with predefined constants
/// for its `disk_size` and `error_size` fields. Additionally, the `error_information_present`
/// field is set to `false` by default.
///
/// # Returns
/// * A `TypeInfo` struct with the following values:
///   - `disk_size`: Set to `DISK_SIZE_VARIANT_1` (a predefined constant).
///   - `error_size`: Set to `ERROR_SIZE_VARIANT_1` (a predefined constant).
///   - `error_information_present`: Set to `false`.
///
/// # Example
/// ```rust
/// let info = default_type_info();
/// assert_eq!(info.disk_size, DISK_SIZE_VARIANT_1);
/// assert_eq!(info.error_size, ERROR_SIZE_VARIANT_1);
/// assert_eq!(info.error_information_present, false);
/// ```
fn default_type_info() -> TypeInfo {
    TypeInfo {
        disk_size: DISK_SIZE_VARIANT_1,
        error_size: ERROR_SIZE_VARIANT_1,
        error_information_present: false,
    }
}

/// Creates and returns a `Vec` of `TypeInfo` structs with predefined values.
///
/// This function initializes a collection of `TypeInfo` instances,
/// each with specific values for `disk_size`, `error_size`, and
/// `error_information_present`. The `disk_size` and `error_size`
/// fields use constant variants (e.g., `DISK_SIZE_VARIANT_1`,
/// `ERROR_SIZE_VARIANT_1`, etc.), and the `error_information_present`
/// field is set to `false` for all instances.
///
/// # Returns
///
/// A vector of `TypeInfo` structs, where:
/// - The first instance has values from `DISK_SIZE_VARIANT_1` and `ERROR_SIZE_VARIANT_1`.
/// - The second instance has values from `DISK_SIZE_VARIANT_2` and `ERROR_SIZE_VARIANT_2`.
/// - The third instance has values from `DISK_SIZE_VARIANT_3` and `ERROR_SIZE_VARIANT_3`.
///
/// # Example
///
/// ```rust
/// let type_infos = create_type_info();
/// assert_eq!(type_infos.len(), 3);
/// assert_eq!(type_infos[0].error_information_present, false);
/// ```
///
/// This example demonstrates how to use the function and check the size of the returned vector,
/// as well as validate the `error_information_present` field for any instance.
fn create_type_info() -> Vec<TypeInfo> {
    vec![
        TypeInfo {
            disk_size: DISK_SIZE_VARIANT_1,
            error_size: ERROR_SIZE_VARIANT_1,
            error_information_present: false,
        },
        TypeInfo {
            disk_size: DISK_SIZE_VARIANT_2,
            error_size: ERROR_SIZE_VARIANT_2,
            error_information_present: false,
        },
        TypeInfo {
            disk_size: DISK_SIZE_VARIANT_3,
            error_size: ERROR_SIZE_VARIANT_3,
            error_information_present: false,
        },
    ]
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
fn load_file(self_ref: &mut D64, path: &str) -> std::io::Result<()> {
    self_ref.data = std::fs::read(path)?;
    Ok(())
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
fn collect_file_sectors(self_ref: &mut D64, start_sector: i32) -> Vec<i32> {
    let mut file_sectors = vec![start_sector];
    let mut current_sector = start_sector;

    // Trace parent sectors
    while let Some(parent_sector) = find_parent_sector(self_ref, current_sector) {
        file_sectors.push(parent_sector);
        current_sector = parent_sector;
    }

    file_sectors.reverse();
    file_sectors
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
fn find_parent_sector(self_ref: &mut D64, sector: i32) -> Option<i32> {
    if sector > get_max_valid_sector(self_ref) {
        return None;
    }

    let track_info = self_ref.get_track_info_for_raw_sector(sector)?;

    for i in 0..self_ref.get_sector_count() as i32 {
        let raw_sector = self_ref.get_sector(i)?;

        if raw_sector.track == track_info.track && raw_sector.sector == track_info.sector {
            return Some(i);
        }
    }
    None
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
fn process_file_sectors(self_ref: &D64, sectors: &[i32]) -> PRG {
    let mut prg_file = PRG {
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

    for &sector_id in sectors {
        if let Some(sector_data) = self_ref.get_sector(sector_id) {
            prg_file
                .data
                .extend_from_slice(extract_sector_data(&sector_data));
        }
    }

    prg_file
}

/// Extracts the relevant data from a sector based on track and sector values.
///
/// # Parameters
/// - `sector`: The sector containing track, sector, and data information
///
/// # Returns
/// A slice of the sector's data that should be included in the final file
fn extract_sector_data(sector: &Sector) -> &[u8] {
    if sector.track != 0 {
        return &sector.data;
    }

    if sector.sector > 0xFD {
        &sector.data
    } else {
        let end_index = sector.sector.saturating_sub(1) as usize;
        &sector.data[..end_index.min(sector.data.len())]
    }
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
fn process_directory_file(self_ref: &D64, file: &PRG) -> Option<Vec<FileEntry>> {
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

            file_entries.push(parse_and_print_directory_entry(
                self_ref,
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
fn parse_and_print_directory_entry(self_ref: &D64, entry_bytes: &[u8]) -> FileEntry {
    let entry = DirectoryEntry::decode_from_le_bytes(entry_bytes);
    let petscii_name = petscii::PetsciiString::from(&entry.data);

    if self_ref.debug_output {
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
fn process_single_file(self_ref: &mut D64, sector_index: i32) -> Option<PRG> {
    let file_sectors = collect_file_sectors(self_ref, sector_index);
    let debug_output = build_sector_debug_string(self_ref, &file_sectors);
    let mut file = process_file_sectors(self_ref, &file_sectors);

    if file_sectors.is_empty() {
        return None;
    }

    let sector_info = self_ref
        .get_track_info_for_raw_sector(file_sectors[0])
        .unwrap();
    if sector_info.track == DIRECTORY_TRACK && sector_info.sector == DIRECTORY_SECTOR {
        if self_ref.debug_output {
            println!("Directory");
        }

        file.directory = true;
    }

    file.track = sector_info.track;
    file.sector = sector_info.sector;

    // Get and print track information for EOF detection
    if self_ref.debug_output {
        if let Some(track_info) = self_ref.get_track_info_for_raw_sector(sector_index) {
            println!(
                "EOF detected - track #{:02}:{:02}",
                track_info.track, track_info.sector
            );
        }

        println!("file sectors used: {}", debug_output);
    }

    Some(file)
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
fn build_sector_debug_string(self_ref: &mut D64, file_sectors: &[i32]) -> String {
    file_sectors
        .iter()
        .filter_map(|&sector| {
            self_ref
                .get_track_info_for_raw_sector(sector)
                .map(|track_info| format!("{:02}:{:02}", track_info.track, track_info.sector))
        })
        .collect::<Vec<_>>()
        .join(" ")
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
fn parse_directory(self_ref: &mut D64) -> Option<Vec<FileEntry>> {
    if let Some(files) = self_ref.find_all_files() {
        for file in files {
            if file.directory {
                return process_directory_file(self_ref, &file);
            }
        }
    }

    None
}
