use super::crawler;
use chrono::Utc;
use data_structures::{
    config::Settings,
    metadata::{self},
};
use reqwest_middleware::ClientWithMiddleware;
use std::collections::HashMap;
use tokio::{self, task::JoinSet};
use tools;
use url::Url;

fn check_linkpage_res_length(download_res: &HashMap<&str, Vec<String>>) -> usize {
    if !download_res.contains_key("author")
        || !download_res.contains_key("link")
        || !download_res.contains_key("avatar")
    {
        println!("字段`author`或字段`link`或字段`avatar`缺失，请检查css规则");
        return 0;
    }
    let author_field = download_res.get("author").unwrap();
    let link_field = download_res.get("link").unwrap();
    if author_field.len() == 0 || link_field.len() == 0 {
        return 0;
    } else if author_field.len() != link_field.len() {
        println!(
            "字段`author`长度: {}, 字段`link`长度: {},不统一，请检查css规则",
            author_field.len(),
            link_field.len()
        );
        return 0;
    } else {
        return author_field.len();
    }
}

pub async fn start_crawl_postpages(
    base_postpage_url: String,
    settings: &Settings,
    extra_feed_suffix: String,
    css_rules: &tools::Value,
    client: &ClientWithMiddleware,
) -> Result<Vec<metadata::BasePosts>, Box<dyn std::error::Error>> {
    let base_url = Url::parse(&base_postpage_url).unwrap(); // TODO 异常处理
    let css_rules = css_rules.clone();
    let mut joinset = JoinSet::new();

    if css_rules["post_page_rules"].is_mapping() {
        let css_rules = css_rules
            .get("post_page_rules")
            .unwrap()
            .as_mapping()
            .unwrap();
        let css_rules = css_rules.clone();
        let client_ = client.clone();
        joinset.spawn(async move {
            // 获取当前时间
            let download_postpage_res =
                match crawler::crawl_post_page(&base_postpage_url, &css_rules, &client_).await {
                    Ok(v) => v,
                    Err(e) => {
                        println!("{}", e);
                        return Vec::new();
                    }
                }; // TODO 异常处理
            let length;
            // 字段缺失检查

            if download_postpage_res.contains_key("title")
                && download_postpage_res.contains_key("link")
            {
                if download_postpage_res.get("title").unwrap().len()
                    != download_postpage_res.get("link").unwrap().len()
                {
                    println!(
                        "url: {} 解析结果缺失`title`或`link`长度不等",
                        base_postpage_url
                    );
                    return Vec::new();
                } else {
                    // 关键字段长度相等
                    length = download_postpage_res.get("title").unwrap().len()
                }
            } else {
                // 缺失title或link任意一个
                println!(
                    "url: {} 解析结果缺失`title`或`link`任意一个",
                    base_postpage_url
                );
                return Vec::new();
            }
            let mut format_base_posts = vec![];
            for i in 0..length {
                let title = download_postpage_res.get("title").unwrap()[i]
                    .trim()
                    .to_string();
                let link = download_postpage_res.get("link").unwrap()[i]
                    .trim()
                    .to_string();
                // TODO 时间格式校验
                let created = match download_postpage_res.get("created") {
                    Some(ref v) => {
                        if i < v.len() {
                            // 如果有值，则使用该值
                            tools::strftime_to_string_ymd(&v[i].trim())
                        } else {
                            // 否则使用当前时间
                            tools::strptime_to_string_ymd(
                                Utc::now().with_timezone(&crawler::BEIJING_OFFSET.unwrap()),
                            )
                        }
                    }
                    // 缺失created字段，否则使用当前时间
                    None => tools::strptime_to_string_ymd(
                        Utc::now().with_timezone(&crawler::BEIJING_OFFSET.unwrap()),
                    ),
                };
                let updated = match download_postpage_res.get("updated") {
                    Some(v) => {
                        if i < v.len() {
                            // 如果有值，则使用该值
                            tools::strftime_to_string_ymd(&v[i].trim())
                        } else {
                            // 否则使用created
                            created.clone()
                        }
                    }
                    // 否则使用created
                    None => created.clone(),
                };

                let base_post = metadata::BasePosts::new(title, created, updated, link);
                format_base_posts.push(base_post);
            }
            format_base_posts
        });
        for feed_suffix in [
            "atom.xml",
            "feed/atom",
            "rss.xml",
            "rss2.xml",
            "feed",
            "index.xml",
            extra_feed_suffix.as_str(),
        ] {
            let client = client.clone();
            let feed_url = base_url.join(feed_suffix).unwrap(); // TODO
            joinset.spawn(async move {
                let res = match crawler::crawl_post_page_feed(feed_url.as_str(), &client).await {
                    Ok(v) => v,
                    Err(e) => {
                        println!("{}", e);
                        Vec::new()
                    }
                };
                res
            });
        }
        while let Some(res) = joinset.join_next().await {
            if let Ok(success) = res {
                if success.len() > 0 {
                    // println!("success:{:?}", success);
                    return Ok(success);
                }
            }
        }
        Ok(Vec::new())
    } else {
        panic!("css_rule 格式错误");
    }
}

pub async fn start_crawl_linkpages(
    settings: &Settings,
    css_rules: &tools::Value,
    client: &ClientWithMiddleware,
) -> Vec<metadata::Friends> {
    let mut format_base_friends = vec![];
    let start_urls = &settings.LINK;
    for linkmeta in start_urls {
        let download_linkpage_res = match crawler::crawl_link_page(
            &linkmeta.link,
            &linkmeta.theme,
            &css_rules["link_page_rules"],
            &client,
        )
        .await
        {
            Ok(v) => v,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };
        let length = check_linkpage_res_length(&download_linkpage_res);
        for i in 0..length {
            let author = download_linkpage_res.get("author").unwrap()[i]
                .trim()
                .to_string();
            // TODO 链接拼接检查
            let link = download_linkpage_res.get("link").unwrap()[i]
                .trim()
                .to_string();
            // TODO 链接拼接检查
            let avatar;
            let _avatar = download_linkpage_res.get("avatar").unwrap();
            if i < _avatar.len() {
                avatar = download_linkpage_res.get("avatar").unwrap()[i]
                    .trim()
                    .to_string();
            } else {
                // 默认图片
                avatar =
                    String::from("https://sdn.geekzu.org/avatar/57d8260dfb55501c37dde588e7c3852c")
            }
            let tm = Utc::now().with_timezone(&crawler::BEIJING_OFFSET.unwrap());
            let created_at = tools::strptime_to_string_ymdhms(tm);
            let base_post = metadata::Friends::new(author, link, avatar, false, created_at);
            format_base_friends.push(base_post);
        }
    }
    format_base_friends
}
