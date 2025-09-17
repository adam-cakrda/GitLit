#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        crate::api::login,
        crate::api::logout,
        crate::api::register,
        crate::api::create_repo,
        crate::api::delete_repo,
        crate::api::list_repos,
        crate::api::branches,
        crate::api::content,
        crate::api::commits,
        crate::api::download
    ),
    components(
        schemas(
            // auth/api
            crate::models::LoginRequest,
            crate::models::LoginResponse,
            crate::models::RegisterRequest,
            crate::models::CreateRepoRequest,
            crate::models::DeleteQuery,
            crate::models::OkResponse,
            crate::models::ReposQuery,
            crate::models::BranchesQuery,
            crate::models::BranchesResponse,
            crate::models::ContentQuery,
            crate::models::ContentResponse,
            crate::models::CommitsQuery,
            // db models
            crate::models::Repository,
            crate::models::Token,
            crate::models::User,
            // repo models
            crate::models::EntryKind,
            crate::models::TreeEntry,
            crate::models::CommitInfo,
            crate::models::Branch,
            // common error response
            crate::models::ErrorResponse
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "repos", description = "Repository management"),
        (name = "git", description = "Git data browsing")
    ),
    modifiers(
        &SecurityAddon
    )
)]
pub struct ApiDoc;
impl ApiDoc {
    pub fn openapi() -> utoipa::openapi::OpenApi {
        <Self as utoipa::OpenApi>::openapi()
    }
}

pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
        use utoipa::openapi::Components;

        let bearer = SecurityScheme::Http(
            HttpBuilder::new()
                .scheme(HttpAuthScheme::Bearer)
                .bearer_format("JWT")
                .build(),
        );

        let mut components = openapi.components.take().unwrap_or_else(Components::new);
        components.add_security_scheme("bearerAuth", bearer);
        openapi.components = Some(components);
    }
}