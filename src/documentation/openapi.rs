use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(info(title = "Reporting API", description = "Powered by AXUM."))]
pub struct ApiDoc {}
