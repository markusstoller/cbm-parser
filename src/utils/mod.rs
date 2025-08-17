pub mod helper {
    use crate::disk::cbm::PRG;
    use rusqlite::{Connection, params};

    pub(crate) trait Database {
        fn new(file_name: &str) -> Self;
        fn close(&mut self) -> bool;
        fn add_item(&mut self, element: Element) -> bool;
        fn insert_file_into_db(
            stmt: &mut rusqlite::Statement,
            parent_file: &str,
            file: &PRG,
        ) -> bool;
        fn write_to_db(&mut self);
    }

    pub(crate) struct Element {
        pub(crate) prg: PRG,
        pub(crate) parent: String,
    }

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

        /// Closes the current operation by performing necessary cleanup tasks.
        ///
        /// This method ensures that any pending operations or data are finalized and appropriately saved.
        /// Specifically, it writes the current state to the database using the `write_to_db` method.
        /// After successfully completing this operation, the method returns `true` to indicate
        /// the close operation was successful.
        ///
        /// # Returns
        ///
        /// * `true` - Always returns `true`, indicating the operation was performed successfully.
        ///
        /// # Example
        ///
        /// ```rust
        /// let mut manager = Manager::new();
        /// let close_result = manager.close();
        /// assert!(close_result);
        /// ```
        fn close(&mut self) -> bool {
            self.write_to_db();
            if let Some(conn) = self.conn.take() {
                // Take ownership and close
                conn.flush_prepared_statement_cache();
                conn.cache_flush().unwrap();
                conn.close().unwrap_or_else(|_| {
                    eprintln!("Failed to close database connection");
                });
            }
            true

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
        fn add_item(&mut self, element: Element) -> bool {
            self.queue.push(element);
            if self.queue.len() > 1000 {
                println!("writing to db");
                self.write_to_db();
            }
            true
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
        fn insert_file_into_db(
            stmt: &mut rusqlite::Statement,
            parent_file: &str,
            file: &PRG,
        ) -> bool {
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
        fn write_to_db(&mut self) {
            if let Some(ref mut conn) = self.conn {
                if let Ok(tx) = conn.transaction() {
                    {
                        let mut stmt = tx
                            .prepare(
                                "INSERT INTO files (parent, filename, track, sector, length, md5, sha1, sha256)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                            )
                            .expect("failed to prepare insert statement");

                        for elem in &self.queue {
                            Self::insert_file_into_db(&mut stmt, &elem.parent, &elem.prg);
                        }
                        self.queue.clear();
                    }
                    tx.commit()
                        .unwrap_or_else(|_| println!("failed to commit transaction"));
                }
            }
        }
    }
}
