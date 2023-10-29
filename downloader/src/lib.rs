use chrono::{FixedOffset, Utc};
use data_structures::metadata;
use feed_rs::parser;
use std::collections::HashMap;
use reqwest::{Client, ClientBuilder};
use tools;
// time zones
// +08:00
pub static BEIJING_OFFSET: Option<FixedOffset> = FixedOffset::east_opt(8 * 60 * 60);

pub async fn crawl_link_page<'a>(
    url: &str,
    theme: &str,
    css_rule: &serde_yaml::Value,
    client: &Client,
) -> Result<HashMap<&'a str, Vec<String>>, Box<dyn std::error::Error>> {
    if css_rule.is_mapping() {
        let theme_rule = match css_rule.get(theme) {
            Some(s) => s,
            None => panic!("`{theme}` field not found in css_rule"),
        };
        let html = client.get(url).send().await?.text().await?;
        let document = nipper::Document::from(&html);
        // 返回结果init
        let mut result: HashMap<&str, Vec<String>> = HashMap::new();
        for rule in ["author", "link", "avatar"] {
            let fields = theme_rule.get(rule).ok_or(format!("`{rule}` 字段缺失"))?;
            let fields = fields
                .as_sequence()
                .ok_or(format!("`{rule}-selector` 字段格式错误"))?;

            let mut res = vec![];
            for field in fields {
                let match_rule: &str = field
                    .get("selector")
                    .ok_or(format!("`{rule}-selector` 字段缺失"))?
                    .as_str()
                    .ok_or(format!("`{rule}-selector` 字段格式错误"))?;
                let attr = field
                    .get("attr")
                    .ok_or(format!("`{rule}-attr` 字段缺失"))?
                    .as_str()
                    .ok_or(format!("`{rule}-attr` 字段格式错误"))?;

                for elem in document.select(match_rule).iter() {
                    let parsed_field = match attr {
                        "text" => elem.text().to_string(),
                        _ => elem
                            .attr(attr)
                            .map(|r| r.to_string())
                            .unwrap_or(String::from("")),
                        // _ => String::from(""),
                    };
                    res.push(parsed_field);
                }
                // 当前规则获取到结果，则认为规则是有效的，短路后续规则
                if res.len() > 0 {
                    break;
                }
            }

            // println!("{:?}",html);

            result.insert(rule, res);
        }
        Ok(result)
    } else {
        panic!("css_rule 格式错误");
    }
}

pub async fn crawl_post_page<'a>(
    url: &str,
    css_rule: &serde_yaml::Value,
    client: &Client,
) -> Result<HashMap<&'a str, Vec<String>>, Box<dyn std::error::Error>> {
    println!("正在请求：{}",url);
    // let html = reqwest::get(url).await?.text().await?;
    let html = client.get(url).send().await?.text().await?;
    let document = nipper::Document::from(&html);
    // 返回结果init
    let mut result: HashMap<&str, Vec<String>> = HashMap::new();
    for rule in ["title", "link", "created", "updated"] {
        let field = css_rule.get(rule).ok_or(format!("`{rule}` 字段缺失"))?;
        let match_rule = field
            .get("selector")
            .ok_or(format!("`{rule}-selector` 字段缺失"))?
            .as_str()
            .ok_or(format!("`{rule}-selector` 字段格式错误"))?;
        let attr = field
            .get("attr")
            .ok_or(format!("`{rule}-attr` 字段缺失"))?
            .as_str()
            .ok_or(format!("`{rule}-attr` 字段格式错误"))?;
        let mut res = vec![];
        for elem in document.select(match_rule).iter() {
            let parsed_field = match attr {
                "text" => elem.text().to_string(),
                _ => elem
                    .attr(attr)
                    .map(|r| r.to_string())
                    .unwrap_or(String::from("")),
                // _ => String::from(""),
            };
            res.push(parsed_field);
        }
        result.insert(rule, res);
    }
    Ok(result)
}

pub async fn crawl_post_page_feed(
    url: &str,
   client: &Client,
) -> Result<Vec<metadata::BasePosts>, Box<dyn std::error::Error>> {
    println!("feed.....{}",url);
    let html = client.get(url).send().await?.bytes().await?;
    // let html = reqwest::get(url).await?.bytes().await?;
    if let Ok(feed_from_xml) = parser::parse(html.as_ref()){
        let entries = feed_from_xml.entries;
        // 返回结果init
        let mut format_base_posts = vec![];
        for entry in entries {
            // 标题
            let title = entry
                .title
                .map_or(String::from("无题"), |t| t.content.to_string());
            // url链接
            let link = if entry.links.len() > 0 {
                entry.links[0].href.clone()
            } else {
                //TODO 日志记录
                continue;
            };
            // 时间
            let created = match entry.published {
                Some(t) => t.fixed_offset(),
                None => Utc::now().with_timezone(&BEIJING_OFFSET.unwrap()),
            };
            let created = tools::strptime_to_string(created);
    
            let updated = match entry.updated {
                Some(t) => t.fixed_offset(),
                None => Utc::now().with_timezone(&BEIJING_OFFSET.unwrap()),
            };
            let updated = tools::strptime_to_string(updated);
            let base_post = metadata::BasePosts::new(title, created, updated, link);
            format_base_posts.push(base_post);
        }
        Ok(format_base_posts)
    }else{
        
        Ok(Vec::new())
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        
    }
}
