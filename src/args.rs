#[cfg(not(test))]
use super::APP_NAME;
use std::{convert::Infallible, path::PathBuf, str::FromStr};

use clap::{value_parser, Parser, Subcommand};
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
    #[clap(default_value = default_config_dir().into_os_string())]
    pub config_dir: PathBuf,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            config_dir: default_config_dir(),
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
    #[command(subcommand)]
    Store(StoreCmd),
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum StoreCmd {
    Get {
        key: String,
    },
    Set {
        key: String,
        #[arg(value_parser = value_parser!(CliJson))]
        value: CliJson,
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
    }
}
