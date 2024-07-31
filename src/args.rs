#[cfg(not(test))]
use super::APP_NAME;
use std::{convert::Infallible, fmt::Display, path::PathBuf, str::FromStr};

use clap::{Parser, Subcommand, ValueEnum};
use serde_json::Value;

#[derive(Debug, Parser, PartialEq)]
pub struct Args {
    #[command(subcommand)]
    pub cmd: Cmd,

    #[command(flatten)]
    pub opts: Options,
}

#[derive(Debug, Parser, PartialEq)]
pub struct Options {
    #[clap(long, default_value = default_config_dir().into_os_string())]
    pub config_dir: PathBuf,

    #[arg(long, default_value_t = JsonFormat::Convenient)]
    pub json_format: JsonFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, ValueEnum)]
pub enum JsonFormat {
    /// Accurate JSON format
    ///
    /// Top-level strings are enclosed by double quotes
    Accurate,

    /// Convenient JSON format
    ///
    /// Top-level strings are not escaped and are not enclosed by double quotes
    #[default]
    Convenient,
}

impl Display for JsonFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accurate => write!(f, "accurate"),
            Self::Convenient => write!(f, "convenient"),
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            config_dir: default_config_dir(),
            json_format: JsonFormat::default(),
        }
    }
}

#[cfg(not(test))]
fn default_config_dir() -> PathBuf {
    xdg::BaseDirectories::new()
        .unwrap()
        .get_config_home()
        .join(APP_NAME)
}

#[cfg(test)]
fn default_config_dir() -> PathBuf {
    PathBuf::from("/test/config/dir")
}

#[derive(Debug, Parser, PartialEq)]
pub enum Cmd {
    /// Commands for interacting with a simple key-value store
    ///
    /// Keys are strings, values are JSONs
    #[command(subcommand)]
    Store(StoreCmd),
    /// Commands for getting system information
    #[command(subcommand)]
    System(SystemCmd),
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum StoreCmd {
    /// Get a value from the store
    Get { key: String },

    /// Set a value in the store
    ///
    /// Values will be parsed as JSON where possible
    Set { key: String, value: CliJson },

    /// Remove the value associated with a key
    ///
    /// Prints `true` if the key had a value associated with it, `false` otherwise
    ///
    /// Note that this is different to setting a value to `null`
    Unset { key: String },

    /// List all key-value pairs in the database
    List,

    /// Cycle between a list of values in order, and print the current value
    ///
    /// If the key does not exist, the first item in the list will be selected
    ///
    /// If the key does exist:
    ///  - if the value is in the list, the next item will be selected
    ///  - if the value is not in the list, the first item in the list will be selected
    Cycle {
        /// The key of the value to be cycled
        key: String,

        /// The list of values to cycle through
        values: Vec<CliJson>,

        /// Whether to cycle in reverse order
        #[arg(long, short)]
        reverse: bool,
    },

    /// Run a command and print the output, caching the result
    ///
    /// On subsequent runs, if the command fails, the cached value will be used instead
    ///
    /// The command is run with `sh -c`
    Cached { cmd: String },
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum SystemCmd {
    /// Print the current battery status in a pretty format
    Battery {
        // The number of spaces to put between the symbol and the percentage
        #[arg(long, short, default_value = "1")]
        num_spaces: u32,

        /// Override the current charge level
        ///
        // Must be in the range `0..=100`
        #[arg(long)]
        override_percentage: Option<u8>,

        // Override whether the battery is currently charging
        #[arg(long)]
        override_charging: Option<bool>,
    },
}

/// A wrapper type around [`serde_json::Value`] that tries to parse the text as a JSON, but falls
/// back to a string if parsing fails
#[derive(Debug, Clone, PartialEq)]
pub struct CliJson(pub Value);

impl FromStr for CliJson {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = serde_json::from_str(s).unwrap_or_else(|_| Value::String(s.to_string()));
        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::APP_NAME;

    use super::*;

    fn check(parts: impl IntoIterator<Item = &'static str>, expected_args: Args) {
        let parsed_args = Args::parse_from(Some(APP_NAME).into_iter().chain(parts));
        assert_eq!(parsed_args, expected_args);
    }

    #[test]
    fn args_parse_correctly() {
        check(
            ["store", "get", "foo"],
            Args {
                cmd: Cmd::Store(StoreCmd::Get { key: "foo".into() }),
                opts: Options::default(),
            },
        );

        check(
            ["store", "set", "foo", "bar"],
            Args {
                cmd: Cmd::Store(StoreCmd::Set {
                    key: "foo".into(),
                    value: CliJson(Value::String("bar".into())),
                }),
                opts: Options::default(),
            },
        );
    }

    #[test]
    fn json_parsing() {
        check(
            ["store", "set", "foo", "123"],
            Args {
                cmd: Cmd::Store(StoreCmd::Set {
                    key: "foo".into(),
                    value: CliJson(json!(123)),
                }),
                opts: Options::default(),
            },
        );

        check(
            ["store", "set", "foo", r#""123""#],
            Args {
                cmd: Cmd::Store(StoreCmd::Set {
                    key: "foo".into(),
                    value: CliJson(json!("123")),
                }),
                opts: Options::default(),
            },
        );

        check(
            ["store", "set", "foo", "true"],
            Args {
                cmd: Cmd::Store(StoreCmd::Set {
                    key: "foo".into(),
                    value: CliJson(json!(true)),
                }),
                opts: Options::default(),
            },
        );

        check(
            ["store", "set", "foo", r#"{"bar": 123}"#],
            Args {
                cmd: Cmd::Store(StoreCmd::Set {
                    key: "foo".into(),
                    value: CliJson(json!({"bar": 123})),
                }),
                opts: Options::default(),
            },
        );

        check(
            ["store", "set", "foo", r#"[1, true, "hello"]"#],
            Args {
                cmd: Cmd::Store(StoreCmd::Set {
                    key: "foo".into(),
                    value: CliJson(json!([1, true, "hello"])),
                }),
                opts: Options::default(),
            },
        );

        check(
            [
                "--json-format",
                "convenient",
                "store",
                "set",
                "foo",
                r#"[1, true, "hello"]"#,
            ],
            Args {
                cmd: Cmd::Store(StoreCmd::Set {
                    key: "foo".into(),
                    value: CliJson(json!([1, true, "hello"])),
                }),
                opts: Options {
                    json_format: JsonFormat::Convenient,
                    ..Options::default()
                },
            },
        );

        check(
            [
                "--json-format",
                "accurate",
                "store",
                "set",
                "foo",
                r#"[1, true, "hello"]"#,
            ],
            Args {
                cmd: Cmd::Store(StoreCmd::Set {
                    key: "foo".into(),
                    value: CliJson(json!([1, true, "hello"])),
                }),
                opts: Options {
                    json_format: JsonFormat::Accurate,
                    ..Options::default()
                },
            },
        );
    }
}
