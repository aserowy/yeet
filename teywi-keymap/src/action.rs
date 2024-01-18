#[derive(Clone, Debug)]
pub enum Action {
    Mode(Mode),
    Refresh,
    Quit,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Mode {
    Normal,
    Command,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}
