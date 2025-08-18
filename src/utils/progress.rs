use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;
use tokio::spawn;
use tokio::task::JoinHandle;

/// Shared state for tracking progress across all workers
pub(crate) struct ProgressState {
    pub(crate) files_parsed: Arc<AtomicI32>,
    pub(crate) files_attempted: Arc<AtomicI32>,
    total_files: Arc<AtomicI32>,
    handle: Option<JoinHandle<()>>,
}

impl ProgressState {
    pub(crate) fn new() -> Self {
        Self {
            files_parsed: Arc::new(AtomicI32::new(0)),
            files_attempted: Arc::new(AtomicI32::new(0)),
            total_files: Arc::new(AtomicI32::new(0)),
            handle: None,
        }
    }

    /// Increments the count of files parsed by 1.
    ///
    /// This function increases the value of the `files_parsed` counter, which is
    /// internally stored as an `AtomicUsize`. The increment operation is performed
    /// using `Ordering::Relaxed`, meaning it does not impose strict memory ordering
    /// constraints. This is suitable if the exact ordering of memory operations
    /// is not critical to the program's correctness.
    ///
    /// # Usage
    /// This method is typically called to track the progress of file parsing operations
    /// in a concurrent or multi-threaded environment where multiple threads may need
    /// to update the counter.
    ///
    /// # Notes
    /// - Calling this method multiple times will increment the counter by the respective
    ///   number of invocations.
    /// - Since `Ordering::Relaxed` is used, you should ensure that higher-order
    ///   synchronization is handled elsewhere if required.
    ///
    /// # Example
    /// ```rust
    /// // Assuming `self` is an instance that contains `files_parsed`:
    /// self.increment_files_parsed();
    /// ```
    pub(crate) fn increment_files_parsed(&self) {
        self.files_parsed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the counter that tracks the number of files attempted.
    ///
    /// This method increments the `files_attempted` atomic counter by 1
    /// using a relaxed memory ordering. Relaxed ordering is used here
    /// as the operation does not require synchronization with other
    /// memory accesses.
    ///
    /// # Note
    /// Ensure that relaxed memory ordering is appropriate for your use
    /// case, as it does not provide guarantees for memory visibility
    /// or ordering across threads.
    pub(crate) fn increment_files_attempted(&self) {
        self.files_attempted.fetch_add(1, Ordering::Relaxed);
    }

    /// Increments the total number of files counter by 1.
    ///
    /// This function increments the value of `total_files` atomically using a relaxed ordering.
    /// It is designed to be used in situations where the ordering of operations across threads
    /// is not critical but there is a need to update the counter safely in a concurrent environment.
    ///
    /// # Notes
    /// - `Ordering::Relaxed` ensures that the operation is performed atomically, but it does not
    ///   enforce any particular memory ordering constraints.
    /// - Use with caution in multithreaded contexts, as relaxed ordering might not be suitable
    ///   for all use cases.
    ///
    /// # Examples
    /// ```
    /// use std::sync::Arc;
    /// use my_crate::MyStruct;
    ///
    /// let my_struct = Arc::new(MyStruct::new());
    /// let clone = my_struct.clone();
    /// clone.increment_total_files();
    /// ```
    pub(crate) fn increment_total_files(&self) {
        self.total_files.fetch_add(1, Ordering::Relaxed);
    }

    /// Prints the final count of files parsed to the standard output.
    ///
    /// This method retrieves the current count of files parsed from the `files_parsed`
    /// atomic variable using relaxed memory ordering, and then formats it as a string
    /// to display a message in the format: "<count> files parsed".
    ///
    /// # Example
    ///
    /// ```
    /// let parser = Parser::new();
    /// // After parsing files:
    /// parser.print_final_count(); // Output example: "42 files parsed"
    /// ```
    ///
    /// # Note
    /// - The `files_parsed` variable is expected to be an atomic counter
    ///   (e.g., `AtomicUsize`) for thread-safe updates from multiple threads.
    /// - Relaxed ordering is sufficient here as the operation does not
    ///   depend on synchronization with other threads.
    pub(crate) fn print_final_count(&self) {
        println!("{} files parsed", self.files_parsed.load(Ordering::Relaxed));
    }

    /// Spawns a background task to periodically report progress of file processing.
    ///
    /// This method creates an asynchronous task that runs in the background,
    /// printing the progress of processed files to the console every second.
    /// It compares the current count of processed files (`files_attempted`)
    /// against the total number of files (`total_files`) and prints the
    /// updated progress only when there is a change in the processed count.
    ///
    /// The method uses atomic counters (`files_attempted` and `total_files`)
    /// for thread-safe tracking of progress. The task runs in an infinite loop,
    /// sleeping for 1 second between iterations using `tokio::time::sleep`.
    ///
    /// # Fields Updated
    /// - `self.handle`: Stores the handle of the spawned asynchronous task.
    ///   This can be used to manage or ensure proper cleanup of the task later.
    ///
    /// # Panics
    /// - This function does not handle any specific panics directly. Any I/O
    ///   or runtime errors occurring during the execution of the task could
    ///   propagate as a runtime panic.
    ///
    /// # Notes
    /// - Ensure the `self.files_attempted` and `self.total_files` atomic counters
    ///   are properly initialized and updated from other parts of the system
    ///   before calling this method.
    /// - The function is designed to be safe and lightweight, using
    ///   `Ordering::Relaxed` for atomic operations to minimize performance impact.
    ///
    /// # Example
    /// ```rust
    /// let mut reporter = Reporter::new();
    /// reporter.spawn_reporter();
    /// // Continue processing files and updating `files_attempted` externally.
    /// ```
    ///
    /// # Dependencies
    /// - Requires the `tokio` runtime to be initialized for asynchronous execution.
    /// - Output is logged using `println!`, which writes progress updates to the console.
    ///
    /// # Limitations
    /// - The progress message (`println!`) is hard-coded; any advanced reporting
    ///   requires modifying the task logic.
    pub(crate) fn spawn_reporter(&mut self) {
        let files_attempted = self.files_attempted.clone();
        let total_files = self.total_files.clone();

        self.handle = Some(spawn(async move {
            let mut last_printed = -1; // sentinel to force initial print
            loop {
                let attempted = files_attempted.load(Ordering::Relaxed);
                let total = total_files.load(Ordering::Relaxed);

                if attempted != last_printed {
                    println!("Progress: {} / {} files processed", attempted, total);
                    last_printed = attempted;
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }));
    }

    /// Terminates the reporter task if it is currently running.
    ///
    /// This function checks if a task handle exists (`self.handle`) and
    /// aborts the associated task. The reporter task will no longer execute
    /// once this function is called.
    ///
    /// # Notes
    /// - Aborting a task is a forceful operation that stops the task's execution
    ///   immediately, potentially leaving shared resources in an inconsistent state.
    ///   Use this function cautiously.
    /// - If no handle is found (`self.handle` is `None`), the function simply returns
    ///   without performing any actions.
    ///
    /// # Example
    /// ```
    /// let reporter = Reporter::new(); // Assume Reporter struct is implemented.
    /// reporter.terminate_reporter(); // Terminates the reporter if running.
    /// ```
    pub(crate) fn terminate_reporter(&self) {
        if let Some(handle) = &self.handle {
            handle.abort();
        }
    }

    /// Terminates the reporter and provides a report of the final count.
    ///
    /// This function is responsible for gracefully stopping the reporter functionality
    /// by invoking the `terminate_reporter` method. Once the reporter has been successfully
    /// terminated, it will print the final count generated by the system using the
    /// `print_final_count` method. This function is intended to ensure proper cleanup
    /// and finalization of the reporting process before concluding execution.
    ///
    /// # Usage
    /// This method is typically called when the reporting process needs to be stopped,
    /// and a summary of the overall count needs to be output for review or logging purposes.
    ///
    /// # Panics
    /// The method may panic if:
    /// - The `terminate_reporter` or `print_final_count` methods encounter any runtime issues.
    ///
    /// # Example
    /// ```
    /// let reporter = Reporter::new();
    /// reporter.terminate_and_report();
    /// ```
    pub(crate) fn terminate_and_report(&self) {
        self.terminate_reporter();
        self.print_final_count();
    }
}
