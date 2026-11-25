//! Everything the user can ask the program to do.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    MoveDown,
    MoveUp,
    MoveTop,
    MoveBottom,
    PageUp,
    PageDown,
    /// Open the add-task prompt.
    StartAdd,
    /// Toggle the selected task's done flag.
    ToggleDone,
    /// Ask for confirmation before deleting the selected task.
    StartDelete,
    /// Confirm the pending delete.
    ConfirmDelete,
    /// Cancel the add prompt or the delete confirmation.
    Cancel,
    /// Commit the add prompt.
    CommitAdd,
    AddInput(char),
    AddBackspace,
    ToggleHelp,
    Quit,
    /// Sent by the app when the list height changes, so paging matches.
    SetPageSize(usize),
}
