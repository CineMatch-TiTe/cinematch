use crate::{Database, QdrantService, vector::models::VectorCollection};
use qdrant_client::{
    Qdrant,
    qdrant::{QueryPointsBuilder, RecommendInputBuilder},
};

use uuid::Uuid;

const RECOMMENDATION_COUNT: u64 = 5;
const NAME: &str = "movies";

impl Database {
    // get recommendations for a user based on their taste, using the colleciton specified
    pub async fn get_recommendations(
        &self,
        user: Uuid,
        party: Option<Uuid>,
        vector: VectorCollection,
    ) {

        // self.vector.client.query(
        //     QueryPointsBuilder::new(NAME)
        //         .query(
        //             RecommendInputBuilder::default()
        //                 .add_positive()
        //         )
        // )
    }

    // get the point id, of the movie and whether the user liked it or not and add to pg
    pub async fn update_taste(&self, user: Uuid, movie_id: &str, like: bool) {
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
