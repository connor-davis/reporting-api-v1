use crate::routes::cybercns;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(cybercns::agents::index, cybercns::agents::import),
    info(title = "Reporting API", description = "Powered by AXUM."),
    tags(
        (name = "CyberCNS", description = "CyberCNS API endpoints.")
    )
)]
pub struct ApiDoc {}
