use chrono::{FixedOffset, Utc};
use data_structures::{
    config::Settings,
    metadata::{self, Friends},
};
use downloader;
use reqwest::{Client, ClientBuilder as CL};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::collections::HashMap;
use std::time::Duration;
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

async fn start_crawl_postpages(
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
                match downloader::crawl_post_page(&base_postpage_url, &css_rules, &client_).await {
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
            extra_feed_suffix.as_str(),
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

async fn start_crawl_linkpages(
    settings: &Settings,
    css_rules: &tools::Value,
    client: &ClientWithMiddleware,
) -> Vec<metadata::Friends> {
    let mut format_base_friends = vec![];
    let start_urls = &settings.LINK;
    for linkmeta in start_urls {
        let download_linkpage_res = match downloader::crawl_link_page(
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
            let tm = Utc::now().with_timezone(&downloader::BEIJING_OFFSET.unwrap());
            let created_at = tools::strptime_to_string_ymdhms(tm);
            let base_post = metadata::Friends::new(author, link, avatar, false, created_at);
            format_base_friends.push(base_post);
        }
    }
    format_base_friends
}

/// 构建请求客户端
fn build_client() -> ClientWithMiddleware {
    let timeout = Duration::new(60, 0);
    let baseclient = CL::new()
        .timeout(timeout)
        .use_rustls_tls()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(baseclient)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();
    client
}

#[tokio::main]
async fn main() {
    let css_rules: tools::Value = tools::get_yaml("./css_rules.yaml").unwrap();
    let fc_settings = tools::get_yaml_settings("./fc_settings.yaml").unwrap();

    let client = build_client();

    // let _cssrule = css_rules.clone();
    let format_base_friends = start_crawl_linkpages(&fc_settings, &css_rules, &client).await;
    // println!("{:?}", format_base_friends);
    let mut all_res = vec![];
    let mut tasks = vec![];

    for friend in format_base_friends {
        // if friend.link != "https://akilar.top/" {
        //     continue;
        // }
        let fc_settings = fc_settings.clone();
        let client = client.clone();
        let css_rules = css_rules.clone();
        let task = tokio::spawn(async move {
            let format_base_posts = start_crawl_postpages(
                friend.link.clone(),
                &fc_settings,
                "".to_string(),
                &css_rules,
                &client,
            )
            .await
            .unwrap();
            // println!("{:?}",format_base_posts);
            (friend, format_base_posts)
        });
        tasks.push(task);
    }
    // 处理配置项友链
    if fc_settings.SETTINGS_FRIENDS_LINKS.enable {
        println!("处理配置项友链...");
        // TODO json_api impl
        let settings_friend_postpages = fc_settings.SETTINGS_FRIENDS_LINKS.list.clone();
        for postpage_vec in settings_friend_postpages {
            let tm = Utc::now().with_timezone(&downloader::BEIJING_OFFSET.unwrap());
            let created_at = tools::strptime_to_string_ymdhms(tm);
            let base_post = metadata::Friends::new(
                postpage_vec[0].clone(),
                postpage_vec[1].clone(),
                postpage_vec[2].clone(),
                false,
                created_at,
            );
            // 请求主页面
            let fc_settings = fc_settings.clone();
            let client = client.clone();
            let css_rules = css_rules.clone();
            let task = tokio::spawn(async move {
                let format_base_posts = start_crawl_postpages(
                    base_post.link.clone(),
                    &fc_settings,
                    if postpage_vec.len() == 3 {
                        String::from("")
                    } else if postpage_vec.len() == 4 {
                        postpage_vec[3].clone()
                    } else {
                        panic!("`SETTINGS_FRIENDS_LINKS-list`下的数组长度只能为3或4");
                    },
                    &css_rules,
                    &client,
                )
                .await
                .unwrap();
                // println!("{:?}",format_base_posts);
                (base_post, format_base_posts)
            });
            tasks.push(task);
        }
    }
    for task in tasks {
        let res = task.await.unwrap();
        all_res.push(res);
    }
    let mut success_friends = Vec::new();
    let mut failed_friends = Vec::new();

    for crawl_res in all_res {
        if crawl_res.1.len() > 0 {
            success_friends.push(crawl_res.0);
        } else {
            failed_friends.push(crawl_res.0);
        }
    }
    println!("成功数 {:?}", success_friends.len());
    println!("失败数 {:?}", failed_friends.len());
    println!("失败友链 {:?}", failed_friends);

    // let settings_friends_links = &settings.SETTINGS_FRIENDS_LINKS;;

    // for t in tasks {
    //     let r = t.await.unwrap();
    //     println!("{:?}", r);
    // }
    // println!("{:?}", all_res);
}
