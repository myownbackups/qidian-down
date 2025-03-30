use scraper::{Html, Selector, selectable::Selectable};

use crate::books::{BookChapter, BookInfo, BookVolume};

pub mod book_info {
    use std::sync::OnceLock;

    use regex::Regex;

    use super::*;

    /// 非常好 regex, 爱来自 deeepseek v3-0324
    static RELEASE_DATE_COUNT_RE: OnceLock<Regex> = OnceLock::new();

    /// 非常好 regex, 爱来自 deeepseek v3-0324
    fn chapter_info_re() -> &'static Regex {
        RELEASE_DATE_COUNT_RE.get_or_init(|| {
            Regex::new(
                r"(?x)                # 启用注释模式
                ^.*?                  # 匹配章节标题部分
                (?:首\s*发\s*时\s*间[:：]\s*)?  # 可选的时间前缀
                (?P<time>\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2})  # 匹配时间
                .*?                   # 中间任意字符
                (?:章\s*节\s*字\s*数[:：]\s*)?  # 可选的字数前缀
                (?P<count>\d+)        # 匹配字数
                "
            ).expect("Invalid regex pattern")
        })
    }

    /// 把一个完整的 title 解析出他的信息
    ///
    /// 围棋：我和AI五五开 1.应杰 首发时间：2023-04-03 10:19:10 章节字数：2136
    ///
    /// 2023-04-03 10:19:10 & 2136
    fn analyze_chapter_name(raw_name: &str) -> Option<(String, u32)> {
        let raw_name = raw_name.trim();

        let captures = chapter_info_re().captures(raw_name)?;

        let time = captures.name("time").map(|m| m.as_str().trim().to_string())?;
        let count = captures.name("count").map(|m| m.as_str().parse().ok())??;
        Some((time.to_string(), count))
    }

    pub fn parse(html: String) -> BookInfo {
        let raw_html = Html::parse_fragment(&html);
        let header_selector = Selector::parse("label[for] > div.volume-header").unwrap();
        let name_selector = Selector::parse("h3.volume-name").unwrap();
        let is_free_selector = Selector::parse("span.free").unwrap();
        let chapter_selector = Selector::parse("ul.volume-chapters").unwrap();
        let chapter_item_selector = Selector::parse("a.chapter-name").unwrap();
        let header_items: Vec<_> = raw_html.select(&header_selector).collect();
        let chapter_items = raw_html.select(&chapter_selector);

        let mut volumes: Vec<BookVolume> = Vec::with_capacity(header_items.len());

        for volume_header in header_items {
            let label = volume_header
                .parent()
                .and_then(|n| n.value().as_element())
                .expect("找不到父级 label 元素");

            let volume_id = label.attr("for").expect("label 缺少 for 属性");
            let volume_name = volume_header
                .select(&name_selector)
                .next()
                .and_then(|h3| h3.text().next())
                .map(|s| s.trim())
                .expect("找不到卷名");
            let is_free = volume_header.select(&is_free_selector).next().is_some();
            let volume = BookVolume {
                title: volume_name.to_string(),
                is_vip: !is_free,
                id: volume_id.to_string(),
                chapters: Vec::new(),
            };
            volumes.push(volume);
        }

        for (index, chatper) in chapter_items.enumerate() {
            let volume = volumes.get_mut(index).unwrap();
            let inner_chapters = chatper.select(&chapter_item_selector);

            for inner_chapter in inner_chapters {
                let infos = inner_chapter.attr("title").expect("章节未找到标题");
                let href = inner_chapter.attr("href").expect("章节未找到链接");
                let (chapter_id, url) = {
                    // //www.qidian.com/chapter/1036741406/748679604/
                    // -> 748679604, https://www.qidian.com/chapter/1036741406/748679604/
                    (href.split('/').nth(4).unwrap().to_string(), href.replace("//", "https://"))
                };
                let title: String = inner_chapter.text().collect();
                let (date, len) = analyze_chapter_name(infos).expect("解析标题错误");
                let chapter = BookChapter {
                    title,
                    length: len,
                    release_date: date,
                    id: chapter_id,
                    url,
                };
                volume.chapters.push(chapter);
            }
        }

        BookInfo {
            id: "book_id".to_string(),
            volumes,
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        const TEST_HTML: &str = include_str!("test.html");

        fn get_test_html() -> Html {
            Html::parse_document(TEST_HTML)
        }

        #[test]
        fn test_get_volumes_with_ids() {
            let html = get_test_html();
            // 选择包含 volume 信息的 label 元素
            let selector = Selector::parse("label[for] > div.volume-header").unwrap();
            let volumes = html.select(&selector);

            // 预期结果数组（卷名, volume_id）
            let expected = [
                ("正文卷", "vol108613887"),
                ("2009", "vol109312720"),
                ("2011", "vol112215417"),
                ("2012", "vol112861110"),
                ("二零一四", "vol113142102"),
                ("番外", "vol113694436"),
            ];

            let volumes: Vec<_> = volumes.collect();
            assert_eq!(volumes.len(), expected.len(), "卷数量与预期不符");

            for (index, volume_header) in volumes.into_iter().enumerate() {
                // 获取父级 label 的 for 属性
                let label = volume_header
                    .parent()
                    .and_then(|n| n.value().as_element())
                    .expect("找不到父级 label 元素");

                let volume_id = label.attr("for").expect("label 缺少 for 属性");

                // 获取卷名
                let name_selector = Selector::parse("h3.volume-name").unwrap();
                let volume_name = volume_header
                    .select(&name_selector)
                    .next()
                    .and_then(|h3| h3.text().next())
                    .map(|s| s.trim())
                    .expect("找不到卷名");

                println!("正在验证: [{}] {} - {}", index, volume_id, volume_name);

                // 断言 volume_id 和卷名
                assert_eq!(
                    (volume_name, volume_id),
                    expected[index],
                    "第 {} 个卷不匹配",
                    index
                );
            }
        }

        #[test]
        fn test_analyze_chapter_name() {
            let cases = vec![
                ("围棋：我和AI五五开 1.应杰 首发时间：2023-04-03 10:19:10 章节字数：2136",
                 Some(("2023-04-03 10:19:10".to_string(), 2136))),
                ("第一章 测试 时间：2023-01-01 00:00:00 字数：1000",
                 Some(("2023-01-01 00:00:00".to_string(), 1000))),
                ("无效章节", None),
            ];

            for (input, expected) in cases {
                assert_eq!(analyze_chapter_name(input), expected);
            }
        }

        #[test]
        fn test_parse_book() {
            let book = book_info::parse(TEST_HTML.to_string());
            assert_eq!(book.volumes.len(), 6);
        }
    }
}
