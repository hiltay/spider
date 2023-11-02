use data_structures::metadata;
use futures::stream::TryStreamExt;
use mongodb::{bson::doc, options::ClientOptions, options::FindOptions, Client};

pub async fn connect_mongodb_client(
    mongodburi: &str,
) -> Result<Client, Box<dyn std::error::Error>> {
    let client_options = ClientOptions::parse(mongodburi).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    Ok(client)
}

// let db = client.database("fcircle");
// for collection_name in db.list_collection_names(None).await.unwrap() {
//     println!("{}", collection_name);
// }
// // Get a handle to a collection of `Book`.
// let typed_collection = db.collection::<metadata::Posts>("Post");
// // Query the books in the collection with a filter and an option.
// let filter = doc! { "author": "贰猹的小窝" };
// let find_options = FindOptions::builder().sort(doc! { "title": 1 }).build();
// let mut cursor = typed_collection.find(filter, find_options).await.unwrap();

// // Iterate over the results of the cursor.
// while let Some(post) = cursor.try_next().await.unwrap() {
//     println!("author: {:?}", post);
// }
