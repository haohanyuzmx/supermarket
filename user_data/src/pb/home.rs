use crate::domain::home::get_all_by_user;
use tonic::{Request, Response, Status};
use util::pb::home::{
    home_server::Home, GetAllHomeRequest, GetAllHomeResponse, HomeAddress, HomeId,
};

#[derive(Debug, Default)]
pub struct HomeImpl {}

#[tonic::async_trait]
impl Home for HomeImpl {
    async fn get_all_home(
        &self,
        request: Request<GetAllHomeRequest>,
    ) -> Result<Response<GetAllHomeResponse>, Status> {
        let homes = get_all_by_user(request.into_inner().user_id)
            .await
            .map_err(|e| Status::unknown(e.to_string()))?;
        Ok(Response::new(GetAllHomeResponse {
            home_addresses: homes
                .into_iter()
                .map(|home| HomeAddress {
                    address_id: home.id,
                    user_id: home.user_id,
                    address: home.home_address,
                })
                .collect(),
        }))
    }

    async fn get_home_by_id(
        &self,
        request: Request<HomeId>,
    ) -> Result<Response<HomeAddress>, Status> {
        let home = crate::repo::home::Home::select_by_id(request.into_inner().home_id)
            .await
            .ok_or(Status::unknown("invalid id"))?;
        Ok(Response::new(HomeAddress {
            address_id: home.id.unwrap(),
            user_id: home.user_id,
            address: home.home_address,
        }))
    }
}
