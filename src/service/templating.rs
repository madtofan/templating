use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use handlebars::Handlebars;
use madtofan_microservice_common::{
    errors::{ServiceError, ServiceResult},
    templating::{
        compose_request::InputValue, ListTemplateRequest, ListTemplateResponse, TemplateInput,
        TemplateResponse,
    },
};
use tracing::{error, info};

use crate::repository::{input::DynInputRepositoryTrait, template::DynTemplateRepositoryTrait};

#[async_trait]
pub trait TemplatingServiceTrait {
    async fn add_template(
        &self,
        name: String,
        description: String,
        body: String,
        inputs: Vec<TemplateInput>,
    ) -> ServiceResult<TemplateResponse>;
    async fn remove_template(&self, name: String) -> ServiceResult<TemplateResponse>;
    async fn list_templates(
        &self,
        request: ListTemplateRequest,
    ) -> ServiceResult<ListTemplateResponse>;
    async fn compose(&self, name: String, inputs: Vec<InputValue>) -> ServiceResult<String>;
}

pub type DynTemplatingServiceTrait = Arc<dyn TemplatingServiceTrait + Send + Sync>;

pub struct TemplatingService {
    template_repository: DynTemplateRepositoryTrait,
    inputs_repository: DynInputRepositoryTrait,
}

impl TemplatingService {
    pub fn new(
        template_repository: DynTemplateRepositoryTrait,
        inputs_repository: DynInputRepositoryTrait,
    ) -> Self {
        Self {
            template_repository,
            inputs_repository,
        }
    }
}

#[async_trait]
impl TemplatingServiceTrait for TemplatingService {
    async fn add_template(
        &self,
        name: String,
        description: String,
        body: String,
        inputs: Vec<TemplateInput>,
    ) -> ServiceResult<TemplateResponse> {
        let existing_template = self.template_repository.get_template(&name).await?;

        if inputs.is_empty() {
            error!("Cannot create template with no inputs");
            return Err(ServiceError::BadRequest(
                "Cannot create template with no inputs".to_string(),
            ));
        }

        if existing_template.is_some() {
            error!("template {:?} already exists", &name);
            return Err(ServiceError::ObjectConflict(String::from(
                "template name is taken",
            )));
        }

        info!("creating template {:?}", &name);
        let created_template = self
            .template_repository
            .add_template(&name, &description, &body, &inputs)
            .await?;

        info!("group successfully created");

        Ok(created_template.into_template_response())
    }

    async fn remove_template(&self, name: String) -> ServiceResult<TemplateResponse> {
        let existing_template = self.template_repository.get_template(&name).await?;

        match existing_template {
            Some(template) => {
                info!("removed template {:?}", &name);
                self.template_repository.remove_template(&name).await?;

                info!("successfully removed subscriber from group");
                Ok(template.into_template_response())
            }
            None => {
                error!("template {:?} does not exists", &name);
                Err(ServiceError::ObjectConflict(String::from(
                    "template name does not exist",
                )))
            }
        }
    }

    async fn list_templates(
        &self,
        request: ListTemplateRequest,
    ) -> ServiceResult<ListTemplateResponse> {
        let templates = self
            .template_repository
            .list_templates(request.offset, request.limit)
            .await?;
        let count = self.template_repository.get_templates_count().await?;

        Ok(ListTemplateResponse {
            templates: templates
                .into_iter()
                .map(|input| input.into_template_response())
                .collect(),
            count,
        })
    }

    async fn compose(&self, name: String, inputs: Vec<InputValue>) -> ServiceResult<String> {
        let existing_template = self.template_repository.get_template(&name).await?;

        if existing_template.is_none() {
            error!("template {:?} does not exists", &name);
            return Err(ServiceError::NotFound(String::from(
                "template name does not exist",
            )));
        }

        let mut handlebars = Handlebars::new();
        let source = existing_template.unwrap();
        assert!(handlebars
            .register_template_string("t1", source.body)
            .is_ok());

        let default_inputs = self
            .inputs_repository
            .get_template_inputs(&source.name)
            .await?;

        let mut data = BTreeMap::new();
        default_inputs.into_iter().for_each(|input| {
            data.insert(input.name, input.default_value);
        });
        inputs.into_iter().for_each(|input| {
            data.insert(input.name, input.value);
        });

        handlebars.render("t1", &data).map_err(|_| {
            ServiceError::InternalServerErrorWithContext("Failed to render template".to_string())
        })
    }
}
