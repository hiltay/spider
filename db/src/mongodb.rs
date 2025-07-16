use data_structures::metadata::{self, Friends, Posts};
use mongodb::{
    bson::doc,
    options::ClientOptions,
    Client, Database,
};

pub async fn connect_mongodb_clientdb(
    mongodburi: &str,
) -> Result<Database, Box<dyn std::error::Error>> {
    let client_options = ClientOptions::parse(mongodburi).await?;
    let client = Client::with_options(client_options)?;
    Ok(client.database("fcircle"))
}

pub async fn insert_post_table(
    post: &Posts,
    db: &Database,
) -> Result<(), Box<dyn std::error::Error>> {
    let collection = db.collection::<Posts>("Posts");
    collection.insert_one(post).await?;
    Ok(())
}

pub async fn insert_friend_table(
    friends: &Friends,
    db: &Database,
) -> Result<(), Box<dyn std::error::Error>> {
    let collection = db.collection::<Friends>("Friends");
    collection.insert_one(friends).await?;
    Ok(())
}

pub async fn bulk_insert_post_table(
    tuples: impl Iterator<Item = metadata::Posts>,
    db: &Database,
) -> Result<(), Box<dyn std::error::Error>> {
    let collection = db.collection::<Posts>("Posts");
    collection.insert_many(tuples).await?;
    Ok(())
}

pub async fn bulk_insert_friend_table(
    tuples: impl Iterator<Item = Friends>,
    db: &Database,
) -> Result<(), Box<dyn std::error::Error>> {
    let collection = db.collection::<Friends>("Friends");
    collection.insert_many(tuples).await?;
    Ok(())
}

pub async fn delete_post_table(
    tuples: impl Iterator<Item = Posts>,
    db: &Database,
) -> Result<(), Box<dyn std::error::Error>> {
    let collection = db.collection::<Posts>("Posts");
    for posts in tuples {
        let filter = doc! { "link": posts.meta.link,"author":posts.author };
        collection.delete_many(filter).await?;
    }
    Ok(())
}

pub async fn truncate_friend_table(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let collection = db.collection::<Friends>("Friends");
    collection.drop().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    

    #[tokio::test]
    async fn internal() {
        assert_eq!(2, 2);
    }
}