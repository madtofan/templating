pub mod templating;

#[cfg(test)]
pub mod test {
    use std::sync::Arc;

    use sqlx::PgPool;

    use crate::{
        repository::{
            input::{DynInputRepositoryTrait, InputRepository},
            template::{DynTemplateRepositoryTrait, TemplateRepository},
        },
        templating::{compose_request::InputValue, TemplateInput},
    };

    use super::templating::{DynTemplatingServiceTrait, TemplatingService};

    struct AllTraits {
        templating_service: DynTemplatingServiceTrait,
        templates_repository: DynTemplateRepositoryTrait,
    }

    fn initialize_handler(pool: PgPool) -> AllTraits {
        let inputs_repository =
            Arc::new(InputRepository::new(pool.clone())) as DynInputRepositoryTrait;
        let templates_repository = Arc::new(TemplateRepository::new(
            pool.clone(),
            inputs_repository.clone(),
        )) as DynTemplateRepositoryTrait;
        let templating_service = Arc::new(TemplatingService::new(
            templates_repository.clone(),
            inputs_repository.clone(),
        )) as DynTemplatingServiceTrait;

        AllTraits {
            templates_repository,
            templating_service,
        }
    }

    #[sqlx::test]
    async fn add_template_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let input = TemplateInput {
            name: "input_name".to_string(),
            default_value: "default_value".to_string(),
        };
        let template_name = "template_name";
        let template_description = "template_description";

        all_traits
            .templating_service
            .add_template(
                template_name.to_string(),
                template_description.to_string(),
                "<p>{input}</p>".to_string(),
                vec![input],
            )
            .await?;

        let created_template = all_traits
            .templates_repository
            .get_template(template_name)
            .await?;

        assert_eq!(created_template.unwrap().description, template_description);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_template_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let template_name = "template_name";
        let template_description = "template_description";
        let input = vec![TemplateInput {
            name: "input_name".to_string(),
            default_value: "default_value".to_string(),
        }];

        all_traits
            .templates_repository
            .add_template(
                template_name,
                template_description,
                "template body {input_name}",
                &input,
            )
            .await?;

        all_traits
            .templating_service
            .remove_template(template_name.to_string())
            .await?;

        let get_template = all_traits
            .templates_repository
            .get_template(template_name)
            .await?;

        assert!(get_template.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn list_templates_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let template_name = "template_name";
        let input = vec![TemplateInput {
            name: "input_name".to_string(),
            default_value: "default_value".to_string(),
        }];

        all_traits
            .templates_repository
            .add_template(
                template_name,
                "template_description",
                "template body {input_name}",
                &input,
            )
            .await?;

        let templates_list = all_traits.templating_service.list_templates().await?;

        assert_eq!(templates_list.len(), 1);
        assert_eq!(templates_list.first().unwrap().name, template_name);

        Ok(())
    }

    #[sqlx::test]
    async fn compose_test(pool: PgPool) -> anyhow::Result<()> {
        let all_traits = initialize_handler(pool);

        let template_name = "template_name";
        let input_name = "input_name";
        let input = vec![TemplateInput {
            name: input_name.to_string(),
            default_value: "default_value".to_string(),
        }];

        all_traits
            .templates_repository
            .add_template(
                template_name,
                "template_description",
                "composed text: {{input_name}}",
                &input,
            )
            .await?;

        let composed_text = all_traits
            .templating_service
            .compose(template_name.to_string(), vec![])
            .await?;

        assert_eq!(&composed_text, "composed text: default_value");

        let input = InputValue {
            name: input_name.to_string(),
            value: "value".to_string(),
        };

        let composed_text = all_traits
            .templating_service
            .compose(template_name.to_string(), vec![input])
            .await?;

        assert_eq!(&composed_text, "composed text: value");

        Ok(())
    }
}
