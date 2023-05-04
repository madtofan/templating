use tonic::{Request, Response, Status};

use crate::{
    service::templating::DynTemplatingServiceTrait,
    templating::{
        list_template_response::Templates, templating_server::Templating, AddTemplateRequest,
        ComposeRequest, ComposeResponse, ListTemplateRequest, ListTemplateResponse,
        RemoveTemplateRequest, TemplatingResponse,
    },
};

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
    ) -> Result<Response<TemplatingResponse>, Status> {
        let req = request.into_inner();

        self.templating_service
            .add_template(req.name, req.description, req.body, req.template_inputs)
            .await?;

        Ok(Response::new(TemplatingResponse {
            message: String::from("Success add template!"),
        }))
    }

    async fn remove_template(
        &self,
        request: Request<RemoveTemplateRequest>,
    ) -> Result<Response<TemplatingResponse>, Status> {
        let req = request.into_inner();

        self.templating_service.remove_template(req.name).await?;

        Ok(Response::new(TemplatingResponse {
            message: String::from("Success removing template!"),
        }))
    }

    async fn list_templates(
        &self,
        _request: Request<ListTemplateRequest>,
    ) -> Result<Response<ListTemplateResponse>, Status> {
        let templates = self
            .templating_service
            .list_templates()
            .await?
            .into_iter()
            .map(|template| template.into())
            .collect::<Vec<Templates>>();

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
