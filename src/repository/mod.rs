pub mod input;
pub mod template;

#[cfg(test)]
pub mod test {
    use std::sync::Arc;

    use madtofan_microservice_common::templating::TemplateInput;
    use sqlx::PgPool;

    use super::{
        input::{DynInputRepositoryTrait, InputRepository},
        template::{DynTemplateRepositoryTrait, TemplateRepository},
    };

    struct AllTraits {
        templates_repository: DynTemplateRepositoryTrait,
        inputs_repository: DynInputRepositoryTrait,
    }

    fn initialize_handler(pool: PgPool) -> AllTraits {
        let inputs_repository =
            Arc::new(InputRepository::new(pool.clone())) as DynInputRepositoryTrait;
        let templates_repository = Arc::new(TemplateRepository::new(
            pool.clone(),
            inputs_repository.clone(),
        )) as DynTemplateRepositoryTrait;

        AllTraits {
            templates_repository,
            inputs_repository,
        }
    }

    #[sqlx::test]
    async fn remove_template_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let template_to_remove_name = "template_to_remove";
        let template_to_remove_body = "template_to_remove_body";
        let inputs = vec![
            TemplateInput {
                name: "input1".to_string(),
                default_value: "default_value1".to_string(),
            },
            TemplateInput {
                name: "input2".to_string(),
                default_value: "default_value2".to_string(),
            },
        ];

        traits
            .templates_repository
            .add_template("template1", "description1", "body1", &inputs)
            .await?;
        traits
            .templates_repository
            .add_template(
                template_to_remove_name,
                "description_to_remove",
                template_to_remove_body,
                &inputs,
            )
            .await?;

        let removed_template = traits
            .templates_repository
            .remove_template(template_to_remove_name)
            .await?;

        let templates_list = traits.templates_repository.list_templates().await?;

        assert_eq!(templates_list.len(), 1);
        assert_eq!(removed_template.unwrap().body, template_to_remove_body);

        Ok(())
    }

    #[sqlx::test]
    async fn get_template_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);

        let template_to_get_name = "template_to_get";
        let template_to_get_body = "template_to_get_body";
        let inputs = vec![
            TemplateInput {
                name: "input1".to_string(),
                default_value: "default_value1".to_string(),
            },
            TemplateInput {
                name: "input2".to_string(),
                default_value: "default_value2".to_string(),
            },
        ];

        traits
            .templates_repository
            .add_template(
                template_to_get_name,
                "description",
                template_to_get_body,
                &inputs,
            )
            .await?;

        let get_template = traits
            .templates_repository
            .get_template(template_to_get_name)
            .await?;

        assert_eq!(get_template.unwrap().body, template_to_get_body);

        Ok(())
    }

    #[sqlx::test]
    async fn remove_inputs_test(pool: PgPool) -> anyhow::Result<()> {
        let traits = initialize_handler(pool);
        let inputs = vec![
            TemplateInput {
                name: "input1".to_string(),
                default_value: "default_value1".to_string(),
            },
            TemplateInput {
                name: "input2".to_string(),
                default_value: "default_value2".to_string(),
            },
        ];

        let template = traits
            .templates_repository
            .add_template("name", "descriptions", "body", &inputs)
            .await?;

        traits.inputs_repository.remove_inputs(template.id).await?;

        let template_inputs = traits
            .inputs_repository
            .get_template_inputs(&template.name)
            .await;

        assert_eq!(template_inputs.unwrap().len(), 0);

        Ok(())
    }
}
