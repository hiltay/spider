use chrono::Utc;
use data_structures::metadata::{self};
use db::{mongodb, mysql, sqlite};
use downloader::download;
use sqlx;
use tokio::{self};
use tools;

#[tokio::main]
async fn main() {
    let now = Utc::now().with_timezone(&downloader::BEIJING_OFFSET.unwrap());

    let css_rules: tools::Value = tools::get_yaml("./css_rules.yaml").unwrap();
    let fc_settings = tools::get_yaml_settings("./fc_settings.yaml").unwrap();

    let client = download::build_client();

    // let _cssrule = css_rules.clone();
    let format_base_friends =
        download::start_crawl_linkpages(&fc_settings, &css_rules, &client).await;
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
            let format_base_posts = download::start_crawl_postpages(
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
            let tm = now;
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
                let format_base_posts = download::start_crawl_postpages(
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
    let mut success_posts = Vec::new();
    let mut success_friends = Vec::new();
    let mut failed_friends = Vec::new();
    match fc_settings.DATABASE.as_str() {
        "sqlite" => {
            // get sqlite conn pool
            let dbpool = sqlite::connect_sqlite_dbpool("data.db").await.unwrap();
            match sqlx::migrate!("../db/schema/sqlite").run(&dbpool).await {
                Ok(()) => (),
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            };
            sqlite::truncate_friend_table(&dbpool).await.unwrap();
            for mut crawl_res in all_res {
                if crawl_res.1.len() > 0 {
                    let posts = crawl_res.1.iter().map(|post| {
                        metadata::Posts::new(
                            post.clone(),
                            crawl_res.0.name.clone(),
                            crawl_res.0.avatar.clone(),
                            tools::strptime_to_string_ymdhms(now),
                        )
                    });
                    sqlite::delete_post_table(posts.clone(), &dbpool)
                        .await
                        .unwrap();
                    sqlite::bulk_insert_post_table(posts, &dbpool)
                        .await
                        .unwrap();
                    sqlite::insert_friend_table(&crawl_res.0, &dbpool)
                        .await
                        .unwrap();
                    success_friends.push(crawl_res.0);
                    success_posts.push(crawl_res.1);
                } else {
                    crawl_res.0.error = true;
                    sqlite::insert_friend_table(&crawl_res.0, &dbpool)
                        .await
                        .unwrap();
                    failed_friends.push(crawl_res.0);
                }
            }
        }
        "mysql" => {
            // get mysql conn pool
            let mysqlconnstr = tools::load_mysql_conn_env().unwrap();
            let dbpool = mysql::connect_mysql_dbpool(&mysqlconnstr).await.unwrap();
            match sqlx::migrate!("../db/schema/mysql").run(&dbpool).await {
                Ok(()) => (),
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            };
            mysql::truncate_friend_table(&dbpool).await.unwrap();
            for mut crawl_res in all_res {
                if crawl_res.1.len() > 0 {
                    let posts = crawl_res.1.iter().map(|post| {
                        metadata::Posts::new(
                            post.clone(),
                            crawl_res.0.name.clone(),
                            crawl_res.0.avatar.clone(),
                            tools::strptime_to_string_ymdhms(now),
                        )
                    });
                    mysql::delete_post_table(posts.clone(), &dbpool)
                        .await
                        .unwrap();
                    mysql::bulk_insert_post_table(posts, &dbpool).await.unwrap();
                    mysql::insert_friend_table(&crawl_res.0, &dbpool)
                        .await
                        .unwrap();
                    success_friends.push(crawl_res.0);
                    success_posts.push(crawl_res.1);
                } else {
                    crawl_res.0.error = true;
                    mysql::insert_friend_table(&crawl_res.0, &dbpool)
                        .await
                        .unwrap();
                    failed_friends.push(crawl_res.0);
                }
            }
        }
        "mongodb" => {
            let mongodburi = tools::load_mongodb_env().unwrap();
            let clientdb = mongodb::connect_mongodb_clientdb(&mongodburi)
                .await
                .unwrap();
            mongodb::truncate_friend_table(&clientdb).await.unwrap();
            for mut crawl_res in all_res {
                if crawl_res.1.len() > 0 {
                    let posts = crawl_res.1.iter().map(|post| {
                        metadata::Posts::new(
                            post.clone(),
                            crawl_res.0.name.clone(),
                            crawl_res.0.avatar.clone(),
                            tools::strptime_to_string_ymdhms(now),
                        )
                    });
                    mongodb::delete_post_table(posts.clone(), &clientdb)
                        .await
                        .unwrap();
                    mongodb::bulk_insert_post_table(posts, &clientdb)
                        .await
                        .unwrap();
                    mongodb::insert_friend_table(&crawl_res.0, &clientdb)
                        .await
                        .unwrap();
                    success_friends.push(crawl_res.0);
                    success_posts.push(crawl_res.1);
                } else {
                    crawl_res.0.error = true;
                    mongodb::insert_friend_table(&crawl_res.0, &clientdb)
                        .await
                        .unwrap();
                    failed_friends.push(crawl_res.0);
                }
            }
        }
        _ => return,
    };

    println!(
        "成功友链数 {}，失败友链数 {}",
        success_friends.len(),
        failed_friends.len()
    );
    println!(
        "本次获取总文章数 {}",
        success_posts.iter().fold(0, |acc, x| { acc + x.len() })
    );
    println!(
        "失联友链明细 {}",
        serde_json::to_string_pretty(&failed_friends).unwrap()
    );
}
