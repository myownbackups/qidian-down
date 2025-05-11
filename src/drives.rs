use rand::Rng;
use serde_json::from_str;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use thirtyfour::{
    By, ChromiumLikeCapabilities, Cookie, DesiredCapabilities, Key, WebDriver, WebElement,
    prelude::ElementWaitable,
};

use crate::CliArg;

#[derive(Debug, Clone)]
pub struct Driver {
    pub driver: WebDriver,
    pub cfg: CliArg,
}

const ROOT_QIDIAN: &str = "https://www.qidian.com";

impl Driver {
    pub async fn new_from_cli(config: CliArg) -> anyhow::Result<Self> {
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
        driver
            .set_implicit_wait_timeout(Duration::from_secs(5))
            .await?;

        Ok(Self {
            driver,
            cfg: config,
        })
    }

    pub async fn get_cookie(&self) -> anyhow::Result<Vec<Cookie>> {
        self.driver.refresh().await?;
        // 检测是否需要登录 (寻找 login-btn)
        let login_btn = self.driver.find(By::Id("login-btn")).await;
        match login_btn {
            Ok(login_btn) => {
                if login_btn.is_displayed().await? {
                    println!("点击登录!");
                    login_btn.click().await?;
                    // 等待登录完成
                    println!("等待用户登录(等你两分钟)");
                    // 等你两分钟
                    login_btn
                        .wait_until()
                        .wait(Duration::from_secs(120), Duration::from_secs(1))
                        .not_displayed()
                        .await?;
                }
            }
            Err(_) => {
                // 看来是不需要登录
                println!("似乎不需要登录");
            }
        }

        Ok(self.driver.get_all_cookies().await?)
    }

    /// 检查并更新 cookie
    ///
    /// 运行后会留下一个起点首页
    pub async fn check_cookie(&self) -> anyhow::Result<()> {
        // 新建一页做操作
        // self.driver.new_tab().await?;
        // let pages = self.driver.windows().await?;
        // let new_page = pages.last().unwrap();
        // self.driver.switch_to_window(new_page.clone()).await?;

        self.driver.goto(ROOT_QIDIAN).await?;
        let cookie_path = Path::new(&self.cfg.cookie_path);
        if cookie_path.exists() {
            let str = std::fs::read_to_string(cookie_path)?;
            let cookies: Vec<Cookie> = from_str(&str)?;
            for cookie in cookies {
                self.driver.add_cookie(cookie).await?;
            }
        }
        let cookies = self.get_cookie().await?;
        println!("正在保存新的 cookie");
        let json = serde_json::to_string_pretty(&cookies)?;
        std::fs::write(cookie_path, json)?;
        println!("保存成功");
        if self.driver.windows().await?.len() != 1 {
            self.driver.close_window().await?;
            self.driver
                .switch_to_window(self.driver.windows().await?.first().unwrap().clone())
                .await?;
        }

        Ok(())
    }

    pub async fn close_pop_window(&self) -> anyhow::Result<()> {
        match self.driver.find(By::Css("#reader .content button")).await {
            Ok(element) => {
                println!("=====找到按键提示弹窗，点击关闭(祈祷起点没有瞎改页面)=====");
                element.click().await?;
            }
            Err(_) => {
                println!("未找到弹窗元素");
            }
        }
        Ok(())
    }

    /// 翻页并保存当前页面内容
    pub async fn flip_page_and_save(&self) -> anyhow::Result<String> {
        let content = self.driver.find(By::Css("#reader .content")).await?;
        let content = content.text().await?;
        Ok(content)
    }

    pub async fn download_book(&self, book_url: &str) -> anyhow::Result<Vec<Vec<String>>> {
        println!("开始下载 url: {}", book_url);
        self.driver.goto(book_url).await?;
        let title = self.driver.title().await?;
        let title = title.split("》").next().unwrap().to_string();
        println!("书名: {}", title);

        let all = self.driver.find(By::Id("allCatalog")).await?;
        // println!("{}", all.inner_html().await?);
        let book_info = crate::parse_page::book_info::parse(all.inner_html().await?);

        println!("书长度: {}", book_info.length());
        let first_chapter = book_info.volumes.first().unwrap().chapters.first().unwrap();
        let chatper_item = self
            .driver
            .find(By::Css(format!(
                "a[href*='{}']",
                first_chapter.a_href_tag()
            )))
            .await?;

        println!("等1s看看第一章");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        chatper_item.click().await?;

        let mut datas = Vec::with_capacity(book_info.volumes.len());
        let mut rng = rand::rng();
        let out_path = PathBuf::from_str("./out")?;
        if !out_path.exists() {
            std::fs::create_dir(&out_path)?;
        }
        let mut counter = 0;
        for (count, vol) in book_info.volumes.iter().enumerate() {
            let mut chapter_htmls = Vec::with_capacity(vol.chapters.len());
            let mut volume_path = out_path.clone();
            volume_path.push(format!("{count}_{}", vol.title));
            if !volume_path.exists() {
                std::fs::create_dir(&volume_path)?;
            }
            for (chp_count, chapter) in vol.chapters.iter().enumerate() {
                counter += 1;
                match self.driver.find(By::Tag("main")).await {
                    Ok(main_element) => {
                        let html = main_element.inner_html().await?;
                        // 随机等一段时间 再 关弹窗
                        println!("正在 阅读 《{}》", chapter.title);

                        let mut chp_path = volume_path.clone();
                        chp_path.push(format!(
                            "{counter}_{}_{chp_count}-{}.html",
                            chapter.title, chapter.id
                        ));
                        println!("保存到 {chp_path:?}");
                        std::fs::write(chp_path, &html)?;

                        chapter_htmls.push(html);
                    }
                    Err(e) => {
                        println!("出毛病啦! {e}")
                    }
                };

                tokio::time::sleep(Duration::from_millis(50 + rng.random_range(0..50))).await;
                self.close_pop_window().await?;
                // 随机再等一会再看下一章

                tokio::time::sleep(Duration::from_millis(50 + rng.random_range(0..50))).await;
                self.driver
                    .active_element()
                    .await?
                    .send_keys(Key::Right)
                    .await?;
            }
            datas.push(chapter_htmls);
        }

        for (vol, vol_data) in book_info.volumes.iter().zip(datas.iter()) {
            for (chatper, chapter_html) in vol.chapters.iter().zip(vol_data.iter()) {
                let mut path = out_path.clone();
                path.push(format!("{}-{}", chatper.title, chatper.id));
                std::fs::write(path, chapter_html)?;
            }
        }

        // tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        // self.close_pop_window().await?;
        // let data = main_element.inner_html().await?;
        // println!("{}", data);
        // println!("等3s看看第二章");
        // tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        // let main_element = self.driver.find(By::Tag("main")).await?;
        // self.close_pop_window().await?;
        // println!("data: {}", main_element.text().await?);
        // tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        // self.driver
        //     .active_element()
        //     .await?
        //     .send_keys(Key::Right)
        //     .await?;
        // tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        Ok(datas)
    }
}

pub async fn main(config: CliArg) -> anyhow::Result<()> {
    let driver = Driver::new_from_cli(config).await?;

    driver.check_cookie().await?;

    // tokio::signal::ctrl_c().await?;

    driver
        .download_book("https://www.qidian.com/book/1042804894/")
        // .download_book("https://www.qidian.com/book/1036741406/")
        .await?;

    tokio::signal::ctrl_c().await?;

    Ok(())
}

pub async fn a_main(config: CliArg) -> anyhow::Result<()> {
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
        }
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
