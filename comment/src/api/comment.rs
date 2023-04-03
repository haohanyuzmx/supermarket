use crate::domain::comment::{
    change_comment as change, comment_to_item, delete_comment as delete, get_item_comment,
    CommentNode,
};
use axum::extract::Query;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use util::axum::auth::UserToken;
use util::axum::Response;
use util::response;

#[derive(Serialize, Deserialize, Debug)]
pub struct Comment {
    item_id: u64,
    comment_to: u64,
    comment: String,
}

pub async fn comment_to(
    Extension(user): Extension<UserToken>,
    Json(comment): Json<Comment>,
) -> Response<CommentNode> {
    response!(
        comment_to_item(
            comment.comment,
            comment.item_id,
            user.user_name,
            user.user_id,
            comment.comment_to
        )
        .await
    )
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentsOfItem {
    pub item_id: u64,
}

#[axum::debug_handler]
pub async fn comments_of(Query(request): Query<CommentsOfItem>) -> Response<Vec<CommentNode>> {
    response!(get_item_comment(request.item_id).await)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentChangeRequest {
    pub comment_id: u64,
    pub comment: String,
}

pub async fn change_comment(
    Extension(user): Extension<UserToken>,
    Json(req): Json<CommentChangeRequest>,
) -> Response<CommentNode> {
    response!(change(req.comment, req.comment_id, user.user_id).await)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentID {
    pub comment_id: u64,
}

pub async fn delete_comment(
    Extension(user): Extension<UserToken>,
    Json(req): Json<CommentID>,
) -> Response<CommentNode> {
    response!(delete(req.comment_id, user.user_id).await)
}
