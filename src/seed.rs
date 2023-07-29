use madtofan_microservice_common::{
    errors::ServiceResult,
    templating::{TemplateInput, TemplateResponse},
};
use mockall::lazy_static;
use tracing::info;

use crate::{
    repository::template::DynTemplateRepositoryTrait,
    service::templating::DynTemplatingServiceTrait,
};

lazy_static! {
    static ref TEMPLATING_REGISTRATION_NAME: &'static str = "registration";
    static ref TEMPLATING_REGISTRATION_DESCRIPTION: &'static str = "Registration email template";
    static ref TEMPLATING_REGISTRATION_BODY: &'static str = "<p>You are now registered to the system, {{name}}</p></br><p>Please click the link below to complete the registration</p></br><a href='{{verification_token}}'>Verify user</a>";
    static ref INPUT_REGISTRATION_NAME_LABEL: &'static str = "name";
    static ref INPUT_REGISTRATION_NAME_DEFAULT_VALUE: &'static str = "";
    static ref INPUT_REGISTRATION_VERIFICATION_TOKEN_LABEL: &'static str = "verification_token";
    static ref INPUT_REGISTRATION_VERIFICATION_TOKEN_DEFAULT_VALUE: &'static str = "";
    static ref TEMPLATING_VERIFIED_NAME: &'static str = "verified";
    static ref TEMPLATING_VERIFIED_DESCRIPTION: &'static str = "Verified registration email template";
    static ref TEMPLATING_VERIFIED_BODY: &'static str = "<p>You are now verified to the system as {{name}}</p></br><p>Enjoy using our application</p>";
    static ref INPUT_VERIFIED_NAME_LABEL: &'static str = "name";
    static ref INPUT_VERIFIED_USERNAME_DEFAULT_VALUE: &'static str = "";
}

pub struct SeedService {
    templating_service: DynTemplatingServiceTrait,
    template_repository: DynTemplateRepositoryTrait,
}

impl SeedService {
    pub fn new(
        templating_service: DynTemplatingServiceTrait,
        template_repository: DynTemplateRepositoryTrait,
    ) -> Self {
        Self {
            templating_service,
            template_repository,
        }
    }

    pub async fn seed(&self) -> ServiceResult<()> {
        let templating_registration_name = String::from(*TEMPLATING_REGISTRATION_NAME);
        let templating_verified_name = String::from(*TEMPLATING_VERIFIED_NAME);

        let existing_templates = self
            .templating_service
            .list_templates()
            .await?
            .into_iter()
            .filter(|template| {
                template.name == templating_registration_name
                    || template.name == templating_verified_name
            })
            .collect::<Vec<TemplateResponse>>();

        if existing_templates.len() == 2 {
            info!("data has already been seeded, bypassing test data setup");
            return Ok(());
        }

        info!("seeding templates...");
        let inputs_registration = vec![
            TemplateInput {
                name: String::from(*INPUT_REGISTRATION_NAME_LABEL),
                default_value: String::from(*INPUT_REGISTRATION_NAME_DEFAULT_VALUE),
            },
            TemplateInput {
                name: String::from(*INPUT_REGISTRATION_VERIFICATION_TOKEN_LABEL),
                default_value: String::from(*INPUT_REGISTRATION_VERIFICATION_TOKEN_DEFAULT_VALUE),
            },
        ];
        self.template_repository
            .add_template(
                *TEMPLATING_REGISTRATION_NAME,
                *TEMPLATING_REGISTRATION_DESCRIPTION,
                *TEMPLATING_REGISTRATION_BODY,
                &inputs_registration,
            )
            .await?;

        let inputs_verified = vec![TemplateInput {
            name: String::from(*INPUT_VERIFIED_NAME_LABEL),
            default_value: String::from(*INPUT_VERIFIED_USERNAME_DEFAULT_VALUE),
        }];
        self.template_repository
            .add_template(
                *TEMPLATING_VERIFIED_NAME,
                *TEMPLATING_VERIFIED_DESCRIPTION,
                *TEMPLATING_VERIFIED_BODY,
                &inputs_verified,
            )
            .await?;
        Ok(())
    }
}
