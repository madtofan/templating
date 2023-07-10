use madtofan_microservice_common::templating::{
    templating_server::Templating, AddTemplateRequest, ComposeRequest, ComposeResponse,
    ListTemplateRequest, ListTemplateResponse, RemoveTemplateRequest, TemplateResponse,
};
use tonic::{Request, Response, Status};

use crate::service::templating::DynTemplatingServiceTrait;

pub struct RequestHandler {
    templating_service: DynTemplatingServiceTrait,
}

impl RequestHandler {
    pub fn new(templating_service: DynTemplatingServiceTrait) -> Self {
        Self { templating_service }
    }
}

#[tonic::async_trait]
impl Templating for RequestHandler {
    async fn add_template(
        &self,
        request: Request<AddTemplateRequest>,
    ) -> Result<Response<TemplateResponse>, Status> {
        let req = request.into_inner();

        let added_template = self
            .templating_service
            .add_template(req.name, req.description, req.body, req.template_inputs)
            .await?;

        Ok(Response::new(added_template))
    }

    async fn remove_template(
        &self,
        request: Request<RemoveTemplateRequest>,
    ) -> Result<Response<TemplateResponse>, Status> {
        let req = request.into_inner();

        let removed_template = self.templating_service.remove_template(req.name).await?;

        Ok(Response::new(removed_template))
    }

    async fn list_templates(
        &self,
        _request: Request<ListTemplateRequest>,
    ) -> Result<Response<ListTemplateResponse>, Status> {
        let templates = self.templating_service.list_templates().await?;

        Ok(Response::new(ListTemplateResponse { templates }))
    }

    async fn compose(
        &self,
        request: Request<ComposeRequest>,
    ) -> Result<Response<ComposeResponse>, Status> {
        let req = request.into_inner();

        let result = self
            .templating_service
            .compose(req.name, req.input_values)
            .await?;

        Ok(Response::new(ComposeResponse { result }))
    }
}
