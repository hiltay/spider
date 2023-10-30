use chrono::{FixedOffset, Utc};
use data_structures::{
    config::Settings,
    metadata::{self, Friends},
};
use downloader;
use reqwest::{Client, ClientBuilder};
use std::collections::HashMap;
use std::time::Duration;
use tokio::{self, task::JoinSet};
use tools;
use url::Url;

fn check_length(download_res: &HashMap<&str, Vec<String>>) -> usize {
    let mut length = 0;

    for field in download_res.iter() {
        let len = field.1.len();
        if length == 0 {
            length = len;
        } else if length != len {
            // TODO 更好的逻辑？
            println!("爬取的字段长度不统一");
            return 0;
            // length = if length > len { len } else { length };
        }
    }
    length
}

async fn start_crawl_postpages(
    base_postpage_url: String,
    settings: &Settings,
    css_rules: &tools::Value,
    client: &Client,
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
                match downloader::crawl_post_page(&base_postpage_url, &css_rules, &client_).await {
                    Ok(v) => v,
                    Err(e) => {
                        println!("{}", e);
                        HashMap::new()
                    }
                }; // TODO 异常处理
            let mut length = 0;
            // 字段缺失检查
            if download_postpage_res.len() != 4 {
                if download_postpage_res.contains_key("title")
                    && download_postpage_res.contains_key("link")
                {
                    if download_postpage_res.get("title").unwrap().len()
                        != download_postpage_res.get("link").unwrap().len()
                    {
                        return Vec::new();
                    } else {
                        // 关键字段长度相等
                        length = download_postpage_res.get("title").unwrap().len()
                    }
                } else {
                    // 缺失title或link任意一个
                    return Vec::new();
                }
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
                                Utc::now().with_timezone(&downloader::BEIJING_OFFSET.unwrap()),
                            )
                        }
                    }
                    // 缺失created字段，否则使用当前时间
                    None => tools::strptime_to_string_ymd(
                        Utc::now().with_timezone(&downloader::BEIJING_OFFSET.unwrap()),
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
        ] {
            let client = client.clone();
            let feed_url = base_url.join(feed_suffix).unwrap(); // TODO
            joinset.spawn(async move {
                let res = match downloader::crawl_post_page_feed(feed_url.as_str(), &client).await {
                    Ok(v) => v,
                    Err(e) => {
                        println!("{}", e);
                        Vec::new()
                    }
                }; // TODO
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

async fn start_crawl_linkpages(
    settings: &Settings,
    css_rules: &tools::Value,
    client: &Client,
) -> Vec<metadata::Friends> {
    let mut format_base_friends = vec![];
    let start_urls = &settings.LINK;
    for linkmeta in start_urls {
        let download_linkpage_res = downloader::crawl_link_page(
            &linkmeta.link,
            &linkmeta.theme,
            &css_rules["link_page_rules"],
            &client,
        )
        .await
        .unwrap();
        let length = check_length(&download_linkpage_res);
        for i in 0..length {
            let author = download_linkpage_res.get("author").unwrap()[i]
                .trim()
                .to_string();
            // TODO 时间格式校验
            let link = download_linkpage_res.get("link").unwrap()[i]
                .trim()
                .to_string();
            let avatar = download_linkpage_res.get("avatar").unwrap()[i]
                .trim()
                .to_string();
            let tm = Utc::now().with_timezone(&downloader::BEIJING_OFFSET.unwrap());
            let created_at = tools::strptime_to_string_ymdhms(tm);
            let base_post = metadata::Friends::new(author, link, avatar, false, created_at);
            format_base_friends.push(base_post);
        }
    }
    format_base_friends
}

#[tokio::main]
async fn main() {
    let css_rules: tools::Value = tools::get_yaml("./css_rules.yaml").unwrap();
    let fc_settings = tools::get_yaml_settings("./fc_settings.yaml").unwrap();
    let timeout = Duration::new(5, 0);
    let client = ClientBuilder::new().timeout(timeout).build().unwrap();
    // let _cssrule = css_rules.clone();
    let format_base_friends = start_crawl_linkpages(&fc_settings, &css_rules, &client).await;
    // println!("{:?}", format_base_friends);
    let mut final_res = vec![];

    for friend in format_base_friends.clone() {
        let fc_settings = fc_settings.clone();
        let client = client.clone();
        let css_rules = css_rules.clone();
        let r = tokio::spawn(async move {
            let format_base_posts =
                start_crawl_postpages(friend.link, &fc_settings, &css_rules, &client)
                    .await
                    .unwrap();
            
            format_base_posts
            
        })
        .await;
        let t = r.unwrap();
        if t.len() > 0 {
            final_res.push(t);
        }
        
        // let format_base_posts =start_crawl_postpages(friend.link, &fc_settings, &css_rules, &client).await;
        // println!("{:?}",format_base_posts);
        break;
    }
    println!("{:?}", final_res);
    println!("{:?}", final_res.len());

    // let settings_friends_links = &settings.SETTINGS_FRIENDS_LINKS;;

    // for t in tasks {
    //     let r = t.await.unwrap();
    //     println!("{:?}", r);
    // }
    // println!("{:#?}", format_base_posts);
}
