use getopts::Options;
use std::env;
use std::path::PathBuf;
use systeroid_core::config::CONFIG_ENV;
use systeroid_core::sysctl::section::Section;
use systeroid_core::sysctl::KERNEL_DOCS_ENV;

/// Help message for the arguments.
const HELP_MESSAGE: &str = r#"
Usage:
    {bin} [options]

Options:
{usage}

For more details see {bin}(8)."#;

/// Command-line arguments.
#[derive(Debug, Default)]
pub struct Args {
    /// Location of the configuration file.
    pub config: Option<PathBuf>,
    /// Refresh rate of the terminal.
    pub tick_rate: u64,
    /// Path of the Linux kernel documentation.
    pub kernel_docs: Option<PathBuf>,
    /// Path for the changed parameters.
    pub save_path: Option<PathBuf>,
    /// Sysctl section to filter.
    pub section: Option<Section>,
    /// Query to search on startup.
    pub search_query: Option<String>,
    /// Foreground color.
    pub fg_color: String,
    /// Background color.
    pub bg_color: String,
    /// Do not parse/show Linux kernel documentation.
    pub no_docs: bool,
    /// Whether if the deprecated variables should be included while listing.
    pub display_deprecated: bool,
}

impl Args {
    /// Returns the available options.
    fn get_options() -> Options {
        let mut opts = Options::new();
        opts.optopt(
            "t",
            "tick-rate",
            "set the tick rate of the terminal [default: 250]",
            "<ms>",
        );
        opts.optopt(
            "D",
            "docs",
            "set the path of the kernel documentation",
            "<path>",
        );
        opts.optopt(
            "",
            "save-path",
            "set the path for saving the changed parameters",
            "<path>",
        );
        opts.optopt("s", "section", "set the section to filter", "<section>");
        opts.optopt("q", "query", "set the query to search", "<query>");
        opts.optopt(
            "",
            "bg-color",
            "set the background color [default: black]",
            "<color>",
        );
        opts.optopt(
            "",
            "fg-color",
            "set the foreground color [default: white]",
            "<color>",
        );
        opts.optflag("n", "no-docs", "do not show the kernel documentation");
        opts.optflag(
            "",
            "deprecated",
            "include deprecated variables while listing",
        );
        opts.optopt(
            "c",
            "config",
            "set the path of the configuration file",
            "<path>",
        );
        opts.optflag("h", "help", "display this help and exit");
        opts.optflag("V", "version", "output version information and exit");
        opts
    }

    /// Parses the command-line arguments.
    pub fn parse(env_args: Vec<String>) -> Option<Self> {
        let opts = Self::get_options();
        let matches = opts
            .parse(&env_args[1..])
            .map_err(|e| eprintln!("error: `{e}`"))
            .ok()?;
        if matches.opt_present("h") {
            let usage = opts.usage_with_format(|opts| {
                HELP_MESSAGE
                    .replace("{bin}", env!("CARGO_PKG_NAME"))
                    .replace("{usage}", &opts.collect::<Vec<String>>().join("\n"))
            });
            println!("{usage}");
            None
        } else if matches.opt_present("V") {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            None
        } else {
            Some(Args {
                tick_rate: matches
                    .opt_get("t")
                    .map_err(|e| eprintln!("error: `{e}`"))
                    .ok()?
                    .unwrap_or(250),
                kernel_docs: matches
                    .opt_str("D")
                    .or_else(|| env::var(KERNEL_DOCS_ENV).ok())
                    .map(PathBuf::from),
                save_path: matches.opt_str("save-path").map(PathBuf::from),
                section: matches.opt_str("s").map(Section::from),
                search_query: matches.opt_str("q"),
                fg_color: matches
                    .opt_str("fg-color")
                    .unwrap_or_else(|| String::from("white")),
                bg_color: matches
                    .opt_str("bg-color")
                    .unwrap_or_else(|| String::from("black")),
                no_docs: matches.opt_present("n"),
                display_deprecated: matches.opt_present("deprecated"),
                config: matches
                    .opt_str("c")
                    .or_else(|| env::var(CONFIG_ENV).ok())
                    .map(PathBuf::from),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args() {
        for env_args in [
            vec![String::new(), String::from("-h")],
            vec![String::new(), String::from("-V")],
        ] {
            assert!(Args::parse(env_args).is_none());
        }

        let args = Args::parse(vec![
            String::new(),
            String::from("-t"),
            String::from("1000"),
            String::from("-D"),
            String::from("/docs"),
            String::from("--no-docs"),
            String::from("-s"),
            String::from("vm"),
            String::from("-q"),
            String::from("test"),
        ])
        .expect("failed to parse arguments");

        assert_eq!(1000, args.tick_rate);
        assert_eq!(Some(PathBuf::from("/docs")), args.kernel_docs);
        assert_eq!(Some(Section::Vm), args.section);
        assert_eq!(Some("test"), args.search_query.as_deref());
        assert!(args.no_docs);
    }
}
