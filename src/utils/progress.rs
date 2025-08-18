use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;
use tokio::spawn;

/// Shared state for tracking progress across all workers
pub(crate) struct ProgressState {
    pub(crate) files_parsed: Arc<AtomicI32>,
    pub(crate) files_attempted: Arc<AtomicI32>,
    total_files: Arc<AtomicI32>,
}

impl ProgressState {
    pub(crate) fn new() -> Self {
        Self {
            files_parsed: Arc::new(AtomicI32::new(0)),
            files_attempted: Arc::new(AtomicI32::new(0)),
            total_files: Arc::new(AtomicI32::new(0)),
        }
    }

    /// Marks the completion of file processing by setting the `total_files` atomic variable to -1.
    ///
    /// This function is typically called to indicate that no more files remain to be processed.
    /// The use of `Ordering::Relaxed` ensures that this operation is performed without enforcing
    /// any ordering constraints with other atomic operations, as strict synchronization is not
    /// required in this context.
    ///
    /// # Effects
    /// - Sets the `total_files` atomic variable to -1, signaling that file processing is complete.
    ///
    /// # Panics
    /// This function does not panic.
    ///
    /// # Usage
    /// Call this function when the file processing workflow has completed.
    pub(crate) fn signal_completion(&self) {
        self.total_files.store(-1, Ordering::Relaxed);
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

    /// Spawns an asynchronous task to continuously report progress of a file processing operation.
    ///
    /// This function takes a reference to a `ProgressState` struct that tracks the number of files
    /// attempted and the total number of files to be processed. It creates an asynchronous task that
    /// periodically prints the progress (every second) to the console in the format:
    ///
    /// `Progress: X / Y files processed`
    ///
    /// where `X` is the number of files attempted so far, and `Y` is the total number of files.
    ///
    /// ### Arguments
    /// - `progress`: A reference to a `ProgressState` structure. This structure contains counters for
    ///   `files_attempted` and `total_files`. These counters are shared between threads and updated atomically.
    ///
    /// ### Behavior
    /// - The function clones the atomic counters for `files_attempted` and `total_files` to be used in the spawned task.
    /// - Inside the task:
    ///   - It periodically checks the current progress (attempted and total files).
    ///   - If the number of attempted files has changed since the last iteration, it prints the current progress to the console.
    ///   - The loop exits when `total_files` is set to `-1`, indicating that the task should terminate.
    ///
    /// ### Note
    /// - The task runs asynchronously in a loop and uses `tokio::time::sleep` to wait 1 second between progress checks.
    ///
    /// ### Example Usage
    /// ```rust
    /// let progress_state = ProgressState {
    ///     files_attempted: Arc::new(AtomicI32::new(0)),
    ///     total_files: Arc::new(AtomicI32::new(100)) // Example total files count
    /// };
    ///
    /// spawn_progress_reporter(&progress_state);
    ///
    /// // Simulate file processing
    /// for i in 0..100 {
    ///     progress_state.files_attempted.store(i + 1, Ordering::Relaxed); // Update attempted count
    ///     tokio::time::sleep(Duration::from_millis(50)).await; // Simulated delay
    /// }
    ///
    /// progress_state.total_files.store(-1, Ordering::Relaxed); // Signal completion
    /// ```
    ///
    /// ### Requirements
    /// - This function requires the use of the Tokio runtime to execute asynchronous tasks.
    /// - Ensure proper synchronization (using atomic counters) for shared progress state between threads.
    ///
    /// ### Limitations
    /// - Progress is printed to the console every second and will not update more frequently even if progress changes rapidly.
    /// - The function assumes that `total_files` is set to `-1` explicitly to indicate task completion.
    pub(crate) fn spawn_reporter(&self) {
        let files_attempted = self.files_attempted.clone();
        let total_files = self.total_files.clone();

        spawn(async move {
            let mut last_printed = -1; // sentinel to force initial print
            loop {
                let attempted = files_attempted.load(Ordering::Relaxed);
                let total = total_files.load(Ordering::Relaxed);

                if attempted != last_printed {
                    println!("Progress: {} / {} files processed", attempted, total);
                    last_printed = attempted;
                }

                if total == -1 {
                    break;
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }
}
