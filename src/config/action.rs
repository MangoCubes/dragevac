/// A single [`Action`] item appears as a box in the program window
/// Whenever you drag an item into the box, the [`Action::command`] is executed.
pub struct Action {
    /// List of MIME types to accept. If the dropped item matches one of the MIME type, the command
    /// will be executed. If not, nothing will happen.
    pub accept: Vec<String>,
    /// Command that will be executed when an item is dropped into it. Instead of putting the whole
    /// command, you should put each part of the command (separated by a space) as an element in the
    /// array, except that part that usually goes into the quote, which should go in as whole. For
    /// example, "bash -c 'ls -lah'" becomes ["bash", "-c", "ls -lah"].
    ///
    /// If %ITEMS is present, it will be replaced with the items you dropped into the box,
    /// concatenated by [`Action::concat`]. If you need to enter %ITEMS literally, enter %%ITEMS.
    pub command: Vec<String>,
    /// If multiple items are dropped into this area and %ITEMS is present, then they will be
    /// concatenated with this string between them.
    pub concat: String,
}
