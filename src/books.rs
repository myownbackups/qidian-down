/// 整本书的信息
#[derive(Debug, Clone)]
pub struct BookInfo {
    pub volumes: Vec<BookVolume>,
    pub id: String,
}

/// 一本书的一卷
#[derive(Debug, Clone)]
pub struct BookVolume {
    /// 标题
    pub title: String,
    /// 是否为 VIP 卷
    pub is_vip: bool,
    /// id
    pub id: String,
    /// 章节列表
    pub chapters: Vec<BookChapter>,
}

/// 一本书的一章
#[derive(Debug, Clone)]
pub struct BookChapter {
    /// 标题
    pub title: String,
    /// 字数
    pub length: u32,
    /// "首发时间" (发布日期)
    pub release_date: String,
    /// id
    pub id: String,
    /// url
    pub url: String,
}

impl BookInfo {
    pub fn length(&self) -> u32 {
        self.volumes.iter().map(|volume| volume.length()).sum()
    }
}

impl BookVolume {
    pub fn length(&self) -> u32 {
        self.chapters.iter().map(|chapter| chapter.length).sum()
    }
}

impl BookChapter {
    pub fn new(title: String, length: u32, release_date: String, id: String, url: String) -> Self {
        Self {
            title,
            length,
            release_date,
            id,
            url,
        }
    }

    pub fn new_from_html(href: &str, release_date: String, title: String, length: u32) -> Self {
        let (chapter_id, url) = {
            // //www.qidian.com/chapter/1036741406/748679604/
            (
                href.split('/').nth(5).unwrap().to_string(),
                href.to_string(),
            )
        };
        Self::new(title, length, release_date, chapter_id, url)
    }

    pub fn a_href_tag(&self) -> String {
        self.url.clone()
    }

    pub fn http_url(&self) -> String {
        self.url.replace("//", "https://")
    }
}
