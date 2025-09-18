#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BrowserAction {
    Version,
    Colors,
    ThemeMode,
    Invalid,
}

const ACTN_VERSION_STR: &str = "debug:version";
const ACTN_COLORS_STR: &str = "action:colors";
const ACTN_THEME_MODE_STR: &str = "action:theme:mode";
const ACTN_INVALID_STR: &str = "action:invalid";
const CMD_UPDATE_STR: &str = "update";
const CMD_AUTO_STR: &str = "auto";
const CMD_DARK_STR: &str = "dark";
const CMD_LIGHT_STR: &str = "light";
const CMD_INVALID_STR: &str = "invalid";

impl BrowserAction {
    pub(crate) fn value(&self) -> &str {
        match self {
            BrowserAction::Version => ACTN_VERSION_STR,
            BrowserAction::Colors => ACTN_COLORS_STR,
            BrowserAction::ThemeMode => ACTN_THEME_MODE_STR,
            BrowserAction::Invalid => ACTN_INVALID_STR,
        }
    }
}

impl std::str::FromStr for BrowserAction {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            ACTN_VERSION_STR => BrowserAction::Version,
            ACTN_COLORS_STR => BrowserAction::Colors,
            ACTN_THEME_MODE_STR => BrowserAction::ThemeMode,
            ACTN_INVALID_STR => BrowserAction::Invalid,
            _ => BrowserAction::Invalid,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SocketCommand {
    Update,
    Auto,
    Dark,
    Light,
    Unknown(String),
}

impl SocketCommand {
    pub(crate) fn value(&self) -> &str {
        match self {
            SocketCommand::Update => CMD_UPDATE_STR,
            SocketCommand::Auto => CMD_AUTO_STR,
            SocketCommand::Dark => CMD_DARK_STR,
            SocketCommand::Light => CMD_LIGHT_STR,
            SocketCommand::Unknown(_) => CMD_INVALID_STR,
        }
    }
}
impl std::str::FromStr for SocketCommand {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            CMD_UPDATE_STR => SocketCommand::Update,
            CMD_AUTO_STR => SocketCommand::Auto,
            CMD_DARK_STR => SocketCommand::Dark,
            CMD_LIGHT_STR => SocketCommand::Light,
            _ => SocketCommand::Unknown(s.to_string()),
        })
    }
}
