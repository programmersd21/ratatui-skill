//! Everything the user can ask the program to do.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    MoveDown,
    MoveUp,
    MoveTop,
    MoveBottom,
    PageUp,
    PageDown,
    /// Open the selected directory.
    Enter,
    Parent,
    StartFilter,
    CommitFilter,
    ClearFilter,
    FilterInput(char),
    FilterBackspace,
    ToggleHidden,
    CycleSort,
    ToggleHelp,
    HelpScrollUp,
    HelpScrollDown,
    HelpPageUp,
    HelpPageDown,
    Quit,
    /// Sent by the app when the list height changes, so paging matches.
    SetPageSize(usize),
}
