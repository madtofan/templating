use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use common::repository::connection_pool::ServiceConnectionPool;
use sqlx::{query_as, types::time::OffsetDateTime, FromRow};

use crate::templating::{TemplateInput, TemplateResponse};

use super::input::{DynInputRepositoryTrait, InputEntity};

#[derive(FromRow)]
pub struct TemplateEntity {
    pub id: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub name: String,
    pub description: String,
    pub body: String,
}

#[derive(FromRow, Debug, Clone)]
pub struct TemplateInputsEntity {
    pub id: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub name: String,
    pub description: String,
    pub body: String,
    pub inputs: Vec<InputEntity>,
}

impl TemplateInputsEntity {
    pub fn into_template_response(self) -> TemplateResponse {
        TemplateResponse {
            name: self.name,
            description: self.description,
            template_inputs: self
                .inputs
                .into_iter()
                .map(|input| input.into())
                .collect::<Vec<TemplateInput>>(),
        }
    }
}

impl From<TemplateInputsEntity> for TemplateResponse {
    fn from(template_entity: TemplateInputsEntity) -> Self {
        Self {
            name: template_entity.name,
            description: template_entity.description,
            template_inputs: template_entity
                .inputs
                .into_iter()
                .map(|input| input.into())
                .collect::<Vec<TemplateInput>>(),
        }
    }
}

#[async_trait]
pub trait TemplateRepositoryTrait {
    async fn list_templates(&self) -> anyhow::Result<Vec<TemplateInputsEntity>>;
    async fn get_template(&self, name: &str) -> anyhow::Result<Option<TemplateInputsEntity>>;
    async fn add_template(
        &self,
        name: &str,
        description: &str,
        body: &str,
        template_inputs: &Vec<TemplateInput>,
    ) -> anyhow::Result<TemplateInputsEntity>;
    async fn remove_template(&self, name: &str) -> anyhow::Result<Option<TemplateInputsEntity>>;
}

pub type DynTemplateRepositoryTrait = Arc<dyn TemplateRepositoryTrait + Send + Sync>;

#[derive(Clone)]
pub struct TemplateRepository {
    pool: ServiceConnectionPool,
    inputs_repository: DynInputRepositoryTrait,
}

impl TemplateRepository {
    pub fn new(pool: ServiceConnectionPool, inputs_repository: DynInputRepositoryTrait) -> Self {
        Self {
            pool,
            inputs_repository,
        }
    }
}

#[async_trait]
impl TemplateRepositoryTrait for TemplateRepository {
    async fn list_templates(&self) -> anyhow::Result<Vec<TemplateInputsEntity>> {
        query_as!(
            TemplateInputsEntity,
            r#"
                select
                    t.id as id,
                    t.name as name,
                    t.description as description,
                    t.body as body,
                    t.created_at as created_at,
                    t.updated_at as updated_at,
                    array_agg((
                        i.id,
                        i.name,
                        i.default_value,
                        i.template_id
                    )) as "inputs!: Vec<InputEntity>"
                from templates as t
                left join inputs as i
                    on t.id = i.template_id
                group by t.id
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("an unexpected error occured while obtaining template")
    }

    async fn get_template(&self, name: &str) -> anyhow::Result<Option<TemplateInputsEntity>> {
        query_as!(
            TemplateInputsEntity,
            r#"
                select
                    t.id as id,
                    t.name as name,
                    t.description as description,
                    t.body as body,
                    t.created_at as created_at,
                    t.updated_at as updated_at,
                    array_agg((
                        i.id,
                        i.name,
                        i.default_value,
                        i.template_id
                    )) as "inputs!: Vec<InputEntity>"
                from templates as t
                left join inputs as i
                    on t.id = i.template_id
                where t.name = $1::varchar
                group by t.id
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await
        .context("an unexpected error occured while obtaining template")
    }

    async fn add_template(
        &self,
        name: &str,
        description: &str,
        body: &str,
        template_inputs: &Vec<TemplateInput>,
    ) -> anyhow::Result<TemplateInputsEntity> {
        let add_template_response = query_as!(
            TemplateEntity,
            r#"
                insert into templates (
                        name,
                        description,
                        body
                    )
                values (
                        $1::varchar,
                        $2::varchar,
                        $3::varchar
                    )
                returning *
            "#,
            name,
            description,
            body,
        )
        .fetch_one(&self.pool)
        .await
        .context("an unexpected error occured while creating the template")?;

        self.inputs_repository
            .add_inputs(template_inputs, add_template_response.id)
            .await?;

        self.get_template(name)
            .await?
            .context("an unexpected error occured while obtaining the newly created template")
    }

    async fn remove_template(&self, name: &str) -> anyhow::Result<Option<TemplateInputsEntity>> {
        let template_to_remove = self
            .get_template(name)
            .await?
            .context("an unexpected error occured while looking for template to remove")
            .unwrap();

        self.inputs_repository
            .remove_inputs(template_to_remove.id)
            .await?;

        query_as!(
            TemplateEntity,
            r#"
                delete from templates 
                where 
                    name = $1::varchar 
                returning *
            "#,
            name,
        )
        .fetch_optional(&self.pool)
        .await
        .context("an unexpected error occured while removing the template")
        .unwrap();

        Ok(Some(template_to_remove))
    }
}
