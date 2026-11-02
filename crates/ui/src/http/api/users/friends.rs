use gilvave_core::{
    dto::users::friends::{
        BlockedUserView, FriendRequestsList, FriendView, SendFriendRequest, SendFriendResponse,
    },
    error::ErrorInfo,
    ids::UserId,
    settings::BASE_HTTP_URL,
};

use crate::http::api::Api;

impl Api {
    pub async fn get_friends() -> Result<Vec<FriendView>, ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "GET",
            &format!("{BASE_HTTP_URL}/users/friends"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to::<Vec<FriendView>>(res).await
    }

    pub async fn remove_friend(friend_id: UserId) -> Result<(), ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "DELETE",
            &format!("{BASE_HTTP_URL}/users/friends/{friend_id}"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to_empty(res).await
    }

    pub async fn get_friend_requests() -> Result<FriendRequestsList, ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "GET",
            &format!("{BASE_HTTP_URL}/users/friends/requests"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to::<FriendRequestsList>(res).await
    }

    pub async fn send_friend_request(
        req: &SendFriendRequest,
    ) -> Result<SendFriendResponse, ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "POST",
            &format!("{BASE_HTTP_URL}/users/friends/requests"),
            req,
            Some(&token),
        )
        .await?;
        Api::response_to::<SendFriendResponse>(res).await
    }

    pub async fn accept_friend_request(sender_id: UserId) -> Result<(), ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "POST",
            &format!("{BASE_HTTP_URL}/users/friends/requests/{sender_id}/accept"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to_empty(res).await
    }

    pub async fn decline_friend_request(user_id: UserId) -> Result<(), ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "POST",
            &format!("{BASE_HTTP_URL}/users/friends/requests/{user_id}/decline"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to_empty(res).await
    }

    pub async fn get_blocked_users() -> Result<Vec<BlockedUserView>, ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "GET",
            &format!("{BASE_HTTP_URL}/users/friends/blocks"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to::<Vec<BlockedUserView>>(res).await
    }

    pub async fn block_user(user_id: UserId) -> Result<(), ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "POST",
            &format!("{BASE_HTTP_URL}/users/friends/blocks/{user_id}"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to_empty(res).await
    }

    pub async fn unblock_user(user_id: UserId) -> Result<(), ErrorInfo> {
        let token = Self::fetch_access_token().await;
        let res = Api::request_raw(
            "DELETE",
            &format!("{BASE_HTTP_URL}/users/friends/blocks/{user_id}"),
            (),
            Some(&token),
        )
        .await?;
        Api::response_to_empty(res).await
    }
}
