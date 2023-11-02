/// 包含基本数据结构定义
pub mod metadata {
    use serde::{Deserialize, Serialize};

    /// 文章结构定义
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct BasePosts {
        pub title: String,
        pub created: String,
        pub updated: String,
        pub link: String,
        pub rule: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Posts {
        #[serde(flatten)]
        pub meta: BasePosts,
        pub author: String,
        pub avatar: String,
        pub createAt: String,
    }

    impl BasePosts {
        pub fn new(
            title: String,
            created: String,
            updated: String,
            link: String,
            rule: String,
        ) -> BasePosts {
            BasePosts {
                title,
                created,
                updated,
                link,
                rule,
            }
        }
    }

    impl Posts {
        pub fn new(meta: BasePosts, author: String, avatar: String, createAt: String) -> Posts {
            Posts {
                meta,
                author,
                avatar,
                createAt,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Friends {
        pub name: String,
        pub link: String,
        pub avatar: String,
        pub error: bool,
        pub createAt: String,
    }

    impl Friends {
        pub fn new(
            name: String,
            link: String,
            avatar: String,
            error: bool,
            createAt: String,
        ) -> Friends {
            Friends {
                name,
                link,
                avatar,
                error,
                createAt,
            }
        }
    }
}

pub mod config {
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct LinkMeta {
        pub link: String,
        pub theme: String,
    }
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct SettingsFriendsLinksMeta {
        pub enable: bool,
        pub json_api: String,
        pub list: Vec<Vec<String>>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Settings {
        pub LINK: Vec<LinkMeta>,
        pub SETTINGS_FRIENDS_LINKS: SettingsFriendsLinksMeta,
        pub BLOCK_SITE: Vec<String>,
        // pub MAX_POSTS_NUM: usize,
        // pub HTTP_PROXY: bool,
        pub OUTDATE_CLEAN: usize,
        pub DATABASE: String,
        pub DEPLOY_TYPE: String,
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
