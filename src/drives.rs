use serde_json::from_str;
use std::{path::Path, time::Duration};

use thirtyfour::{
    By, ChromiumLikeCapabilities, Cookie, DesiredCapabilities, WebDriver, prelude::ElementWaitable,
};

use crate::CliArg;

pub async fn main(config: CliArg) -> anyhow::Result<()> {
    let driver = match config.driver_type {
        crate::DriverType::Edge => {
            let mut cap = DesiredCapabilities::edge();
            cap.add_arg("--disable-blink-features=AutomationControlled")?;
            WebDriver::new(&config.driver_url, cap).await?
        }
        crate::DriverType::Chrome => {
            let mut cap = DesiredCapabilities::chrome();
            cap.add_arg("--disable-blink-features=AutomationControlled")?;
            WebDriver::new(&config.driver_url, cap).await?
        } // crate::DriverType::Firefox => {
          //     // let mut cap = DesiredCapabilities::firefox();
          //     // let mut prefrence = FirefoxPreferences::new();
          //     // prefrence.set("dom.webdriver.enabled", false)?;
          //     // prefrence.set("useAutomationExtension", false)?;
          //     // cap.set_preferences(prefrence)?;
          // }
    };

    // 检测是否存在 cookie 文件
    let cookie_path = Path::new(&config.cookie_path);
    let mut need_cookie = true;

    driver.goto("https://www.qidian.com").await?;

    if cookie_path.exists() {
        let str = std::fs::read_to_string(cookie_path)?;
        let cookies: Vec<Cookie> = from_str(&str)?;
        for cookie in cookies {
            driver.add_cookie(cookie).await?;
        }
        need_cookie = false;
    }

    // disable-blink-features=AutomationControlled

    // let mut cookie = Cookie::new("", value);

    // driver
    //     .execute(
    //         "Object.defineProperty(navigator, 'webdriver', {get: () => false})",
    //         Vec::new(),
    //     )
    //     .await?;
    // driver
    //     .execute(
    //         "delete window.navigator.wrappedJSObject.__proto__.webdriver",
    //         Vec::new(),
    //     )
    //     .await?;

    // driver.goto("https://bot.sannysoft.com/").await?;
    // tokio::signal::ctrl_c().await?;

    driver.goto("https://www.qidian.com").await?;

    driver
        .set_implicit_wait_timeout(Duration::from_secs(2))
        .await?;

    // for value in raw_cookies.split("; ") {
    //     println!("{}", value);
    //     if let Some((name, value)) = value.split_once("=") {
    //         let mut cookie = Cookie::new(name, value);
    //         cookie.set_domain("www.qidian.com");
    //         cookie.set_path("/");
    //         cookie.set_same_site(thirtyfour::SameSite::Lax);
    //         driver.add_cookie(cookie).await?;
    //     } else {
    //         println!("!!! error cookie")
    //     }
    // }

    // tokio::time::sleep(Duration::from_secs(10)).await;

    if need_cookie {
        let login_btn = driver.find(By::Id("login-btn")).await?;
        login_btn.click().await?;
        println!("等待用户登录(等你两分钟)");
        // 等你两分钟
        login_btn
            .wait_until()
            .wait(Duration::from_secs(120), Duration::from_secs(1))
            .not_displayed()
            .await?;
    }

    // driver
    //     .goto("https://www.qidian.com/chapter/1036741406/763645826/")
    //     .await?;
    // driver.refresh().await?;
    {
        // 每次登录后都写入 cookie
        let cookies = driver.get_all_cookies().await?;
        let str = serde_json::to_string_pretty(&cookies)?;

        // 写入
        std::fs::write(cookie_path, str.as_bytes()).unwrap();

        driver
            .goto("https://www.qidian.com/book/1036741406/")
            .await?;
        println!("cookie已经更新到文件中");
    }

    // let chapter = driver.find(By::Css(css))

    let volumes = {
        let volumes = driver.find_all(By::ClassName("volume-chapters")).await?;
        // volumes.iter().map(|volume| {
        //     volume.find_all(By::ClassName("chapter-item")).await?
        // }).collect()
        let mut ret = Vec::new();
        for volume in volumes {
            let chapters = volume.find_all(By::ClassName("chapter-item")).await?;
            println!("{}:{}", ret.len(), chapters.len());
            ret.push(chapters);
        }
        ret
    };
    // driver.new_tab().await?;
    // let windows = driver.windows().await?;
    // for win in windows {
    //     driver.switch_to_window(win.clone()).await?;
    //     let current_url = driver.current_url().await?;
    //     println!("当前窗口的 URL: {}", current_url);
    //     let title = driver.title().await?;
    //     println!("当前窗口的标题: {}", title);
    // }

    // driver
    //     .goto("https://www.qidian.com/chapter/1036741406/763719397/")
    //     .await?;

    // driver.execute("debugger;", Vec::new()).await?;
    // driver
    //     .execute(
    //         "document.querySelectorAll('devtools-toggle').forEach((btn) => { btn.click() })",
    //         Vec::new(),
    //     )
    //     .await?;
    // driver.goto("https://bot.sannysoft.com/").await?;

    // 等待 ctrl+c

    tokio::signal::ctrl_c().await?;

    Ok(())
}
