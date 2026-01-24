use crate::{Database, QdrantService, vector::models::VectorCollection};
use qdrant_client::{
    Qdrant,
    qdrant::{QueryPointsBuilder, RecommendInputBuilder},
};

use uuid::Uuid;

const RECOMMENDATION_COUNT: u64 = 10;
const NAME: &str = "movies";

impl Database {
    pub async fn get_recommendations(&self, user: Uuid, vector: VectorCollection) {

        // self.vector.client.query(
        //     QueryPointsBuilder::new(NAME)
        //         .query(
        //             RecommendInputBuilder::default()
        //                 .add_positive()
        //         )
        // )
    }

    pub async fn add_taste(&self, user: Uuid, movie_id: &str, like: bool) {
        // let point_id = format!("{}-{}", user, movie_id);
        // let vector = self.vector.generate_taste_vector(user, movie_id, like).await;

        // self.vector.client.upsert_points(
        //     UpsertPointsBuilder::new(NAME)
        //         .add_point(
        //             PointStruct::new(point_id, vector)
        //         )
        // ).await.unwrap();
    }
}
