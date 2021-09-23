use bson::{doc, Document};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use tonic::Status;
use uuid::Uuid;

use crate::RusDbConnection;

#[derive(Clone)]
pub struct RusDocument<T>
where
    T: Serialize + DeserializeOwned + Clone + std::fmt::Debug,
{
    _id: Uuid,
    collection: RusCollection<T>,
    pub document: T,
}

impl<T> std::fmt::Debug for RusDocument<T>
where
    T: Serialize + DeserializeOwned + Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.document)
    }
}

impl<T> RusDocument<T>
where
    T: Serialize + DeserializeOwned + Clone + std::fmt::Debug,
{
    pub fn create(_id: Uuid, collection: RusCollection<T>, document: T) -> Self {
        Self {
            _id,
            collection,
            document,
        }
    }
    pub fn from_slice(slice: &[u8], collection: RusCollection<T>) -> Result<Self, bson::de::Error> {
        let doc = bson::from_slice::<Document>(slice)?;
        Self::from_document(doc, collection)
    }
    pub fn from_document(
        doc: Document,
        collection: RusCollection<T>,
    ) -> Result<Self, bson::de::Error> {
        let _id: Uuid = bson::from_bson(doc.get("_id").expect("no id").clone())?;
        let document: T = bson::from_document(doc)?;
        Ok(Self {
            _id,
            document,
            collection,
        })
    }
    pub fn id(&self) -> Uuid {
        self._id.clone()
    }
    pub fn to_document(&self) -> Result<Document, bson::ser::Error> {
        let mut doc = bson::to_document(&self.document)?;
        doc.insert("_id", bson::to_bson(&self._id)?);
        Ok(doc)
    }
    fn id_doc(&self) -> Result<Document, bson::ser::Error> {
        Ok(doc!("_id": bson::to_bson(&self._id)?))
    }
    pub fn to_vec(&self) -> Result<Vec<u8>, bson::ser::Error> {
        bson::to_vec(&self.to_document()?)
    }
    pub async fn sync(&mut self) -> Result<(), Status> {
        let doc = self.to_document().unwrap();
        self.collection
            .update(self.id_doc().unwrap(), doc, Some(1))
            .await?;
        Ok(())
    }
    pub async fn delete(mut self) -> Result<(), Status> {
        if self
            .collection
            .remove(self.id_doc().unwrap(), Some(1))
            .await?
            == 1
        {
            drop(self);
            Ok(())
        } else {
            Err(Status::aborted("no document was removed."))
        }
    }
}

#[derive(Clone)]
pub struct RusCollection<T>
where
    T: Serialize + DeserializeOwned + Clone + std::fmt::Debug,
{
    collection: String,
    conn: RusDbConnection,
    document_type: PhantomData<T>,
}

impl<T> RusCollection<T>
where
    T: Serialize + DeserializeOwned + Clone + std::fmt::Debug,
{
    pub fn create(collection: String, conn: RusDbConnection) -> Self {
        Self {
            collection,
            conn,
            document_type: PhantomData::default(),
        }
    }
    fn create_document_vec(&self, data: &[u8]) -> Result<RusDocument<T>, bson::de::Error> {
        RusDocument::from_slice(data, self.clone())
    }
    pub async fn insert(&mut self, doc: T) -> Result<RusDocument<T>, Status> {
        let res = self
            .conn
            .insert(&self.collection, bson::to_document(&doc).unwrap(), true)
            .await?;
        let res = res.get_ref();
        if res.count == 0 {
            Err(Status::aborted("no documents inserted."))
        } else {
            let docs: Vec<RusDocument<T>> = res
                .inserts
                .iter()
                .map(|v| {
                    let data = v.document.as_ref().expect("document data missing");
                    self.create_document_vec(data).unwrap()
                })
                .collect();
            for d in docs {
                return Ok(d);
            }
            Err(Status::aborted("no document returned"))
        }
    }
    pub async fn insert_many(&mut self, docs: Vec<T>) -> Result<Vec<RusDocument<T>>, Status> {
        let res = self
            .conn
            .insert_many(
                &self.collection,
                docs.into_iter()
                    .map(|v| bson::to_document(&v).unwrap())
                    .collect(),
                true,
            )
            .await?;
        let res = res.get_ref();
        if res.count == 0 {
            Err(Status::aborted("no documents inserted."))
        } else {
            Ok(res
                .inserts
                .iter()
                .map(|v| {
                    let data = v.document.as_ref().expect("document data missing");
                    self.create_document_vec(data).unwrap()
                })
                .collect())
        }
    }
    pub async fn update(
        &mut self,
        filter: Document,
        updates: Document,
        limit: Option<u32>,
    ) -> Result<Vec<RusDocument<T>>, Status> {
        let res = self
            .conn
            .update(&self.collection, filter, updates, limit)
            .await?;
        let res = res.get_ref();
        Ok(res
            .updated
            .iter()
            .map(|v| self.create_document_vec(v).unwrap())
            .collect())
    }
    pub async fn remove(&mut self, filter: Document, limit: Option<u32>) -> Result<u32, Status> {
        let res = self.conn.remove(&self.collection, filter, limit).await?;
        let res = res.get_ref();
        Ok(res.count)
    }
    pub async fn find(
        &mut self,
        filter: Document,
        limit: Option<u32>,
    ) -> Result<Vec<RusDocument<T>>, Status> {
        let res = self
            .conn
            .find(&self.collection, Some(filter), limit)
            .await?;
        let res = res.get_ref();
        Ok(res
            .documents
            .iter()
            .map(|v| self.create_document_vec(v).unwrap())
            .collect())
    }
    pub async fn find_all(&mut self, limit: Option<u32>) -> Result<Vec<RusDocument<T>>, Status> {
        let res = self.conn.find(&self.collection, None, limit).await?;
        let res = res.get_ref();
        Ok(res
            .documents
            .iter()
            .map(|v| self.create_document_vec(v).unwrap())
            .collect())
    }
    pub async fn get(&mut self, id: Uuid) -> Result<Option<RusDocument<T>>, Status> {
        let id = id.to_string();
        let res = self.conn.get(&self.collection, &id).await?;
        let res = res.get_ref();
        if let Some(doc) = &res.document {
            Ok(Some(self.create_document_vec(doc).unwrap()))
        } else {
            Ok(None)
        }
    }
    pub async fn truncate(&mut self) -> Result<u32, Status> {
        let res = self.conn.remove(&self.collection, doc!(), None).await?;
        let res = res.get_ref();
        Ok(res.count)
    }
}
