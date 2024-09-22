#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Emulator {
    Alacritty,
    Foot,
    Ghostty,
    Hyper,
    Iterm2,
    Kitty,
    Neovim,
    Tabby,
    Unknown,
    Urxvt,
    VsCode,
    WezTerm,
    WindowsTerminal,
}

impl Default for Emulator {
    fn default() -> Self {
        if is_env_set("NVIM") {
            return Self::Neovim;
        }

        if is_env_set("WT_Session") {
            return Self::WindowsTerminal;
        }

        let (term, term_program) = get_term_env();
        let emulator = match term.as_str() {
            "alacritty" => Self::Alacritty,
            "foot" => Self::Foot,
            "foot-extra" => Self::Foot,
            "rxvt-unicode-256color" => Self::Urxvt,
            "xterm-ghostty" => Self::Ghostty,
            "xterm-kitty" => Self::Kitty,
            _ => Self::Unknown,
        };

        if emulator != Self::Unknown {
            return emulator;
        }

        let emulator = match term_program.as_str() {
            "Hyper" => Self::Hyper,
            "Tabby" => Self::Tabby,
            "WezTerm" => Self::WezTerm,
            "ghostty" => Self::Ghostty,
            "vscode" => Self::VsCode,
            "iTerm.app" => Self::Iterm2,
            _ => Self::Unknown,
        };

        if emulator == Self::Unknown {
            tracing::warn!("Emulator could not identified!");
        }

        emulator
    }
}

fn get_term_env() -> (String, String) {
    (
        std::env::var("TERM").unwrap_or_default(),
        std::env::var("TERM_PROGRAM").unwrap_or_default(),
    )
}

fn is_env_set(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|env| !env.is_empty())
}
