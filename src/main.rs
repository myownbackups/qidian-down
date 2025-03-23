use std::str::FromStr;

use anyhow::Result;
use clap::Parser;

mod drives;

const ABOUT: &str = "起点!";
const LONG_ABOUT: &str = r#"boost !
基于 msedge webdriver"#;
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DriverType {
    #[default]
    Edge,
    Chrome,
    Firefox,
}

impl FromStr for DriverType {
    type Err = std::io::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "edge" => Ok(DriverType::Edge),
            "chrome" => Ok(DriverType::Chrome),
            "firefox" => Ok(DriverType::Firefox),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid driver type",
            )),
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[command(version = VERSION, about = ABOUT, long_about = LONG_ABOUT, name = "namerena-cli-webderiver")]
pub struct CliArg {
    #[arg(
        short = 'd',
        long = "driver",
        default_value = "http://localhost:9515",
        help = "webdriver 的地址",
        long_help = "使用的 msedge webdriver 地址"
    )]
    pub driver_url: String,
    #[arg(
        short = 'c',
        long = "cookie",
        default_value = "cookie.json",
        help = "cookie 存储文件的路径",
        long_help = "这里存储了 cookie"
    )]
    pub cookie_path: String,
    #[arg(
        short = 't',
        long = "type",
        default_value = "edge",
        help = "webdriver 的类型"
    )]
    /// 所使用的 webdriver 类型
    ///
    /// 可选: edge, chrome, firefox
    pub driver_type: DriverType,
}

fn main() -> Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(a_main())
}

async fn a_main() -> Result<()> {
    let args = CliArg::parse();

    drives::main(args).await?;

    Ok(())
}
