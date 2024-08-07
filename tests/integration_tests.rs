use clap::{Parser, ValueEnum};
use clap_macros::ClapDefault;
use url::Url;

#[clap_macros::prefix]
#[derive(ClapDefault, Parser, Debug, PartialEq)]
#[clap(author, version, about, long_about = None)]
struct Parameters {
    #[arg(long, default_value_t = 8080)]
    port: u16,
    #[arg(long, env, value_parser, default_value = "localhost")]
    host: String,
    #[arg(long, env, value_enum, default_value_t = Mode::Release)]
    mode: Mode,
    #[arg(long)]
    path: String,
    #[arg(long)]
    tls: bool,
    #[arg(long)]
    option: Option<String>,
    #[arg(long, default_value = "https://www.google.com")]
    url: Url,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    Debug,
    Release,
}

#[test]
fn test_clap_default() {
    let parameters = Parameters::default();
    assert_eq!(parameters.port, 8080);
    assert_eq!(parameters.host, "localhost");
    assert_eq!(parameters.mode, Mode::Release);
    assert_eq!(parameters.path, "");
    assert_eq!(parameters.tls, false);
    assert_eq!(parameters.option, None);
    assert_eq!(parameters.url, "https://www.google.com".parse().unwrap());
}
