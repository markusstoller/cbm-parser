pub mod helper {
    use crate::disk::cbm::PRG;
    use rusqlite::{Connection, params};

    /// The `Database` trait defines a set of methods required for implementing a database.
    /// This trait is designed to work with functionalities like creating a new database instance,
    /// adding items, inserting files, and writing to the database while providing flexibility for implementation.
    pub(crate) trait Database {
        /// Creates a new instance of the struct, initializing it with the provided file name.
        ///
        /// # Parameters
        /// - `file_name`: A string slice that holds the name of the file to be used for initialization.
        ///
        /// # Returns
        /// - A new instance of the struct.
        ///
        /// # Example
        /// ```rust
        /// let instance = StructName::new("example.txt");
        /// ```
        fn new(file_name: &str) -> Self;
        /// Closes the current resource or connection associated with the implementation.
        ///
        /// This function is typically used to release or finalize any resources that the
        /// implementation might be using. It is expected to ensure that any necessary cleanup
        /// is performed before returning.
        ///
        /// # Returns
        ///
        /// - `Ok(true)` if the resource was successfully closed.
        /// - `Ok(false)` if the resource was already closed.
        /// - `Err(String)` if there was an error during the closing process, with the error
        ///   message describing the issue.
        ///
        /// # Errors
        ///
        /// If closing fails (e.g., due to a runtime error, resource not being available, etc.),
        /// this function will return an `Err` with a description of the issue.
        ///
        /// # Example
        ///
        /// ```rust
        /// let mut resource = MyResource::new();
        /// match resource.close() {
        ///     Ok(true) => println!("Resource closed successfully."),
        ///     Ok(false) => println!("Resource was already closed."),
        ///     Err(e) => println!("Failed to close resource: {}", e),
        /// }
        /// ```
        fn close(&mut self) -> Result<bool, String>;
        /// Adds an item to the collection.
        ///
        /// # Parameters
        /// - `element`: The item of type `Element` to be added to the collection.
        ///
        /// # Usage
        /// Call this method to add a new `Element` to the current instance of the collection.
        ///
        /// # Example
        /// ```
        /// let mut collection = Collection::new();
        /// let item = Element::new();
        /// collection.add_item(item);
        /// ```
        ///
        /// # Notes
        /// - This function requires a mutable reference to the collection.
        /// - The behavior of this method depends on how the collection handles duplicate elements, if applicable.
        ///
        /// # Errors
        /// This method does not return any errors, but improper use may result in logical issues (e.g., unintended duplication of elements).
        fn add_item(&mut self, element: Element);
    }

    /// Represents an element in the application with associated attributes.
    ///
    /// The `Element` struct contains a pseudo-random generator (PRG) and a parent identifier,
    /// which can be used to define relationships or associations with other elements.
    ///
    /// # Fields
    /// * `prg` - A `PRG` instance used for generating pseudo-random values, specific to the element.
    /// * `parent` - A `String` representing the identifier of the parent element.
    ///   This could signify a relationship or hierarchy within a structure.
    ///
    /// # Visibility
    /// This struct and its fields are only visible within the crate (`pub(crate)` access level),
    /// meaning they are not exposed outside the crate.
    pub(crate) struct Element {
        pub(crate) prg: PRG,
        pub(crate) parent: String,
    }

    /// A struct representing a handler for managing SQLite operations.
    ///
    /// The `SqliteHandler` structure serves as a utility to handle SQLite database operations.
    /// It contains a queue to stage elements and manages an optional database connection.
    ///
    /// # Fields
    /// - `queue`: A vector of type `Element` that holds queued items to be processed or executed
    ///    in conjunction with SQLite operations.
    /// - `conn`: An optional `Connection` instance that represents the connection to the SQLite
    ///    database. This is `None` if no connection is currently established.
    ///
    /// # Visibility
    /// This struct is intended for internal use (via the `pub(crate)` visibility modifier) and is not
    /// exposed outside of its defining crate.
    ///
    /// # Example Usage
    /// ```rust
    /// let handler = SqliteHandler {
    ///     queue: Vec::new(),
    ///     conn: None,
    /// };
    ///
    /// // Add elements to the queue or set up SQLite connections as per use-case.
    /// ```
    ///
    /// This structure is typically used in conjunction with other components for database handling within the crate.
    pub(crate) struct SqliteHandler {
        queue: Vec<Element>,
        conn: Option<Connection>,
    }

    /// Initializes and sets up a SQLite database at the specified file path.
    ///
    /// This function performs the following steps:
    /// - Opens a connection to the SQLite database file specified by `file_name`.
    ///   - If the file does not exist, it will be created.
    ///   - If the connection fails, the function will panic with an error message.
    /// - Creates a table named `files` in the database if it doesn't already exist.
    ///   - The `files` table is defined with the following schema:
    ///     - `id`: An auto-incremented integer serving as the primary key.
    ///     - `parent`: A `TEXT` field storing the parent directory or identifier.
    ///     - `filename`: A `TEXT` field storing the file's name.
    ///     - `track`: An `INTEGER` field storing the file's track information.
    ///     - `sector`: An `INTEGER` field storing the file's sector information.
    ///     - `length`: An `INTEGER` field storing the file's length or size.
    ///     - `md5`: A `TEXT` field storing the MD5 hash of the file.
    ///     - `sha1`: A `TEXT` field storing the SHA-1 hash of the file.
    ///     - `sha256`: A `TEXT` field storing the SHA-256 hash of the file.
    ///   - If the creation of the `files` table fails, the function will panic with an error message.
    ///
    /// # Arguments
    ///
    /// - `file_name`: A string slice that specifies the path to the SQLite database file.
    ///
    /// # Returns
    ///
    /// - A `Connection` object representing the established connection to the SQLite database.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The database file cannot be opened.
    /// - The `files` table cannot be created.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let db_connection = setup_database("my_database.db");
    /// // `db_connection` is now ready to be used for database operations.
    /// ```
    fn setup_database(file_name: &str) -> Connection {
        let conn = Connection::open(file_name).expect("failed to open sqlite utils");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            parent TEXT NOT NULL,
            filename TEXT NOT NULL,
            track INTEGER NOT NULL,
            sector INTEGER NOT NULL,
            length INTEGER NOT NULL,
            md5 TEXT NOT NULL,
            sha1 TEXT NOT NULL,
            sha256 TEXT NOT NULL
        )",
            [],
        )
        .expect("failed to create files table");

        conn
    }

    /// Inserts a file record into the SQLite database.
    ///
    /// # Arguments
    ///
    /// * `stmt` - A mutable reference to a `rusqlite::Statement` that should be prepared
    ///            with the appropriate SQL INSERT statement before calling this function.
    /// * `parent_file` - A string slice representing the name or path of the parent file
    ///                   associated with the file being inserted.
    /// * `file` - A reference to a `PRG` object that contains information about the file to be inserted.
    ///            It is expected to have the following fields:
    ///   - `name`: The name of the file.
    ///   - `track`: The track number associated with the file.
    ///   - `sector`: The sector number where the file is located.
    ///   - `data`: The byte content of the file (used for its length).
    ///   - `hashes.md5`: The MD5 hash of the file data.
    ///   - `hashes.sha1`: The SHA1 hash of the file data.
    ///   - `hashes.sha256`: The SHA256 hash of the file data.
    ///
    /// # Returns
    ///
    /// * Returns `true` if the insertion succeeds, or `false` if there is an error during
    ///   the execution of the SQL statement. If an error occurs, it prints an error message.
    ///
    /// # Errors
    ///
    /// If the `stmt.execute` function fails, the error is logged to the standard error
    /// output and the function returns `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let db_conn = rusqlite::Connection::open("example.db").unwrap();
    /// let mut stmt = db_conn
    ///     .prepare("INSERT INTO files (parent_file, name, track, sector, data_length, md5, sha1, sha256) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")
    ///     .unwrap();
    ///
    /// let parent_file = "parent.txt";
    /// let file = PRG {
    ///     name: "file.txt".to_string(),
    ///     track: 1,
    ///     sector: 2,
    ///     data: vec![0u8; 100],
    ///     hashes: Hashes {
    ///         md5: "fake_md5_hash".to_string(),
    ///         sha1: "fake_sha1_hash".to_string(),
    ///         sha256: "fake_sha256_hash".to_string(),
    ///     },
    /// };
    ///
    /// let success = insert_file_into_db(&mut stmt, parent_file, &file);
    /// assert!(success);
    /// ```
    fn insert_file_into_db(stmt: &mut rusqlite::Statement, parent_file: &str, file: &PRG) -> bool {
        if let Err(e) = stmt.execute(params![
            parent_file,
            file.name,
            file.track as i64,
            file.sector as i64,
            file.data.len() as i64,
            file.hashes.md5,
            file.hashes.sha1,
            file.hashes.sha256
        ]) {
            eprintln!("failed to insert row into sqlite: {}", e);
            return false;
        }

        true
    }

    /// This method, `write_to_db`, writes queued file data into the database.
    ///
    /// # Description
    /// - It begins a database transaction using `self.conn`.
    /// - Prepares an SQL INSERT statement to add file information into the `files` table. The table
    ///   includes fields: `parent`, `filename`, `track`, `sector`, `length`, `md5`, `sha1`, and `sha256`.
    /// - Iterates over the `self.queue` (a collection of file data entries), and for each entry,
    ///   calls the helper method `insert_file_into_db` to insert the data into the database.
    /// - Once all data in the queue is processed, clears the `self.queue` to avoid redundant processing.
    /// - Commits the transaction to apply changes to the database. If the commit fails, an error message
    ///   is printed to indicate that the commit did not succeed.
    ///
    /// # Panics
    /// - If preparing the SQL insert statement fails, the program will panic with the message
    ///   "failed to prepare insert statement".
    ///
    /// # Errors
    /// - If the transaction creation fails (returns an error), the method will exit silently without making
    ///   any changes to the database.
    /// - If `tx.commit()` fails, an error message will be printed informing the user that the transaction
    ///   could not be committed.
    ///
    /// # Usage
    /// - Ensure `self.conn` is a valid database connection before calling this method.
    /// - Populate `self.queue` with file data in the expected format (containing `parent` and `prg` fields)
    ///   before invoking this function, as it processes and clears the queue.
    fn write_to_db(sq: &mut SqliteHandler) -> Result<(), String> {
        if let Some(ref mut conn) = sq.conn {
            if let Ok(tx) = conn.transaction() {
                {
                    let mut stmt = tx
                        .prepare(
                            "INSERT INTO files (parent, filename, track, sector, length, md5, sha1, sha256)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        )
                        .expect("failed to prepare insert statement");

                    for elem in &sq.queue {
                        insert_file_into_db(&mut stmt, &elem.parent, &elem.prg);
                    }
                    sq.queue.clear();
                }
                if let Err(_) = tx.commit() {
                    return Err("failed to commit transaction".to_string());
                }

                Ok(())
            } else {
                Err("failed to create transaction".to_string())
            }
        } else {
            Err("connection not established".to_string())
        }
    }

    impl Database for SqliteHandler {
        /// Creates a new instance of `SqliteHandler` by initializing a database connection
        /// and an empty queue.
        ///
        /// # Arguments
        ///
        /// * `file_name` - A string slice that holds the name of the database file to connect to.
        ///
        /// # Returns
        ///
        /// * A new instance of `SqliteHandler` with an established database connection and an empty queue.
        ///
        /// # Example
        ///
        /// ```
        /// let handler = SqliteHandler::new("example.db");
        /// ```
        fn new(file_name: &str) -> Self {
            let conn = setup_database(file_name);

            SqliteHandler {
                conn: Some(conn),
                queue: Vec::new(),
            }
        }

        /// Closes the active connection, performs cleanup, and writes the current state to the database.
        ///
        /// This method ensures that the connection is safely closed, and underlying resources are properly flushed
        /// and released. If the `conn` field exists, it takes ownership of the connection, flushes the prepared
        /// statement cache, ensures the cache is written, and closes the connection. The current state is also
        /// serialized and saved in the database prior to closing the connection.
        ///
        /// # Returns
        ///
        /// * `Ok(true)` - If the connection was successfully closed.
        /// * `Err(String)` - If an error occurs during the process, such as an issue in closing the connection,
        ///   or if no connection exists to be closed.
        ///
        /// # Errors
        ///
        /// This function will return:
        /// * `Err("No connection to close")` if there is no active connection.
        /// * A string description of any failure that occurs during the connection closure.
        ///
        /// # Panics
        ///
        /// This function will panic if serializing the current state to the database fails.
        ///
        /// # Examples
        ///
        /// ```
        /// let mut obj = MyObject::new();
        /// if let Err(err) = obj.close() {
        ///     eprintln!("Failed to close connection: {}", err);
        /// }
        /// ```
        fn close(&mut self) -> Result<bool, String> {
            write_to_db(self).expect("Serializing failed");
            if let Some(conn) = self.conn.take() {
                // Take ownership and close
                conn.flush_prepared_statement_cache();
                conn.cache_flush().unwrap();
                if let Err(result) = conn.close() {
                    return Err(result.1.to_string());
                }
                return Ok(true);
            }
            Err("No connection to close".to_string())
        }

        /// Adds an element to the queue.
        ///
        /// This function pushes a new `element` of type `Element` into the `queue`.
        /// If the length of the queue exceeds 1000 after adding the element,
        /// the function writes the queue's contents to the database by calling `write_to_db()`.
        /// A message "writing to db" is logged to the console when this occurs.
        ///
        /// # Arguments
        ///
        /// * `element` - The element of type `Element` to be added to the queue.
        ///
        /// # Returns
        ///
        /// Returns `true` after the element is successfully added to the queue.
        ///
        /// # Side Effects
        ///
        /// When the queue length exceeds 1000, the current queue state is written to the database.
        ///
        /// # Examples
        ///
        /// ```rust
        /// let mut obj = YourStruct::new();
        /// let element = Element::new();
        /// obj.add_item(element);
        /// ```
        fn add_item(&mut self, element: Element) {
            self.queue.push(element);
            if self.queue.len() > 10000 {
                println!("serializing queue to db");
                write_to_db(self).expect("Serializing failed");
            }
        }
    }
}
