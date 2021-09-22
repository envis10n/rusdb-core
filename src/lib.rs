pub mod grpc {
    tonic::include_proto!("grpc");
}

pub mod collection;
pub mod database;
use bson::Document;
use grpc::rus_db_client::RusDbClient;
use grpc::*;
use serde::{de::DeserializeOwned, Serialize};
use tonic::{transport::Channel, Request, Response, Status};

pub use bson;
pub use collection::{RusCollection, RusDocument};
pub use database::RusDatabase;
pub use tonic;

#[derive(Clone)]
pub struct RusDbConnection {
    client: RusDbClient<Channel>,
}

impl RusDbConnection {
    pub async fn connect(dst: &'static str) -> Self {
        let client = RusDbClient::connect(dst).await.unwrap();
        Self { client }
    }
    pub fn collection<T>(&self, collection: &str) -> RusCollection<T>
    where
        T: Serialize + DeserializeOwned + Clone,
    {
        RusCollection::create(collection.to_string(), self.clone())
    }
    pub async fn insert(
        &mut self,
        collection: &str,
        document: Document,
        return_old: bool,
    ) -> Result<Response<InsertResponses>, Status> {
        self.client
            .insert(Request::new(InsertRequest {
                collection: collection.to_string(),
                documents: vec![bson::to_vec(&document).unwrap()],
                return_old,
            }))
            .await
    }
    pub async fn insert_many(
        &mut self,
        collection: &str,
        documents: Vec<Document>,
        return_old: bool,
    ) -> Result<Response<InsertResponses>, Status> {
        self.client
            .insert(Request::new(InsertRequest {
                collection: collection.to_string(),
                documents: documents
                    .into_iter()
                    .map(|v| bson::to_vec(&v).unwrap())
                    .collect(),
                return_old,
            }))
            .await
    }
    pub async fn update(
        &mut self,
        collection: &str,
        filter: Document,
        updates: Document,
        limit: Option<u32>,
    ) -> Result<Response<UpdateResponses>, Status> {
        self.client
            .update(Request::new(UpdateRequest {
                collection: collection.to_string(),
                filter: bson::to_vec(&filter).unwrap(),
                updates: bson::to_vec(&updates).unwrap(),
                limit,
            }))
            .await
    }
    pub async fn remove(
        &mut self,
        collection: &str,
        filter: Document,
        limit: Option<u32>,
    ) -> Result<Response<RemoveResponse>, Status> {
        self.client
            .remove(Request::new(RemoveRequest {
                collection: collection.to_string(),
                filter: bson::to_vec(&filter).unwrap(),
                limit,
            }))
            .await
    }
    pub async fn get(
        &mut self,
        collection: &str,
        id: &str,
    ) -> Result<Response<GetResponse>, Status> {
        self.client
            .get(Request::new(GetRequest {
                collection: collection.to_string(),
                id: id.to_string(),
            }))
            .await
    }
    pub async fn find(
        &mut self,
        collection: &str,
        filter: Option<Document>,
        limit: Option<u32>,
    ) -> Result<Response<FindResponse>, Status> {
        let filter = {
            if let Some(doc) = filter {
                Some(bson::to_vec(&doc).unwrap())
            } else {
                None
            }
        };
        self.client
            .find(Request::new(FindRequest {
                collection: collection.to_string(),
                filter,
                limit,
            }))
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::RusDbConnection;
    use bson::{bson, doc};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Default, Debug)]
    pub struct TestDoc {
        hello: String,
    }

    #[tokio::test]
    async fn it_works() {
        let client = RusDbConnection::connect("http://127.0.0.1:3010").await;
        let mut col = client.collection::<TestDoc>("test");
        let doc = col
            .get(bson::from_bson(bson!("56a0212f-e21d-48de-81af-7fdc08fee5a2")).unwrap())
            .await
            .unwrap()
            .expect("doc is not there");
        println!("DOC = {:?}", doc.document);
    }
}
