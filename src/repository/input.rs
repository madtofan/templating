use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use common::repository::connection_pool::ServiceConnectionPool;
use mockall::automock;
use sqlx::{query_as, FromRow, Type};

use crate::templating::TemplateInput;

#[derive(FromRow, Type, Debug, Eq, PartialEq, Clone)]
pub struct InputEntity {
    pub id: i64,
    pub name: String,
    pub default_value: String,
    pub template_id: i64,
}

impl From<InputEntity> for TemplateInput {
    fn from(input_entity: InputEntity) -> Self {
        Self {
            name: input_entity.name,
            default_value: input_entity.default_value,
        }
    }
}

#[automock]
#[async_trait]
pub trait InputRepositoryTrait {
    async fn get_template_inputs(&self, template_name: &str) -> anyhow::Result<Vec<InputEntity>>;
    async fn add_inputs(
        &self,
        inputs: &[TemplateInput],
        template_id: i64,
    ) -> anyhow::Result<Vec<InputEntity>>;
    async fn remove_inputs(&self, template_id: i64) -> anyhow::Result<Option<InputEntity>>;
}

pub type DynInputRepositoryTrait = Arc<dyn InputRepositoryTrait + Send + Sync>;

#[derive(Clone)]
pub struct InputRepository {
    pool: ServiceConnectionPool,
}

impl InputRepository {
    pub fn new(pool: ServiceConnectionPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InputRepositoryTrait for InputRepository {
    async fn get_template_inputs(&self, template_name: &str) -> anyhow::Result<Vec<InputEntity>> {
        query_as!(
            InputEntity,
            r#"
                select
                    i.id as id,
                    i.name as name,
                    i.default_value as default_value,
                    i.template_id as template_id
                from inputs as i
                join templates as t
                on i.template_id = t.id
                where t.name = $1::varchar
            "#,
            template_name
        )
        .fetch_all(&self.pool)
        .await
        .context("an unexpected error occured while obtaining inputs for template")
    }

    async fn add_inputs(
        &self,
        inputs: &[TemplateInput],
        template_id: i64,
    ) -> anyhow::Result<Vec<InputEntity>> {
        let mut names: Vec<String> = Vec::new();
        let mut default_values: Vec<String> = Vec::new();
        let mut template_ids: Vec<i64> = Vec::new();
        inputs.to_owned().iter().cloned().for_each(|template| {
            names.push(template.name);
            default_values.push(template.default_value);
            template_ids.push(template_id);
        });
        query_as!(
            InputEntity,
            r#"
                insert into inputs (
                        name,
                        default_value,
                        template_id
                    )
                select * from unnest (
                        $1::text[],
                        $2::text[],
                        $3::bigint[]
                    )
                returning *
            "#,
            &names,
            &default_values,
            &template_ids
        )
        .fetch_all(&self.pool)
        .await
        .context("an unexpected error occured while creating the input")
    }

    async fn remove_inputs(&self, template_id: i64) -> anyhow::Result<Option<InputEntity>> {
        query_as!(
            InputEntity,
            r#"
                delete from inputs 
                where 
                    template_id = $1::bigint 
                returning *
            "#,
            template_id,
        )
        .fetch_optional(&self.pool)
        .await
        .context("an unexpected error occured while removing the input")
    }
}
