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

}
