pub mod config;

pub use utoipa;
pub use utoipauto::{utoipa_ignore, utoipauto};

use config::UtoipaConfig;
use spring::async_trait;
use spring::config::ConfigRegistry;
use spring::plugin::{MutableComponentRegistry, Plugin};
use spring::{app::AppBuilder, plugin::ComponentRegistry};
use spring_web::{Router, WebConfigurator};
use utoipa::openapi::OpenApi;

#[cfg(not(any(feature = "rapidoc", feature = "swagger-ui")))]
use spring_web::axum::{Json, routing};
#[cfg(feature = "rapidoc")]
use utoipa_rapidoc::RapiDoc;
#[cfg(feature = "redoc")]
use utoipa_redoc::{Redoc, Servable};
#[cfg(feature = "scalar")]
use utoipa_scalar::{Scalar, Servable};
#[cfg(feature = "swagger-ui")]
use utoipa_swagger_ui::SwaggerUi;

pub trait UtoipaConfigurator {
    fn with_openapi(&mut self, openapi: OpenApi) -> &mut Self;
}
impl UtoipaConfigurator for AppBuilder {
    fn with_openapi(&mut self, openapi: OpenApi) -> &mut Self {
        if let Some(_wrapper) = self.get_component_ref::<OpenApi>() {
            panic!("Error adding OpenApi: OpenApi was already added in application")
        } else {
            self.add_component(openapi)
        }
    }
}

pub struct UtoipaPlugin;

#[async_trait]
impl Plugin for UtoipaPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<UtoipaConfig>()
            .expect("utoipa plugin config load failed");

        let openapi = app
            .get_component::<OpenApi>()
            .expect("Expected openapi to be configured");

        let mut router = Router::new();

        #[cfg(any(
            all(feature = "rapidoc", feature = "redoc"),
            all(feature = "rapidoc", feature = "scalar"),
            all(feature = "rapidoc", feature = "swagger-ui"),
            all(feature = "redoc", feature = "scalar"),
            all(feature = "redoc", feature = "swagger-ui"),
            all(feature = "scalar", feature = "swagger-ui"),
        ))]
        panic!("Only one OpenApi visualizer can be used!");

        #[cfg(not(any(feature = "rapidoc", feature = "swagger-ui")))]
        {
            let openapi = openapi.clone();
            router = router.route(&config.path, routing::get(|| async move { Json(openapi) }));
        }

        #[cfg(any(
            feature = "rapidoc",
            feature = "redoc",
            feature = "scalar",
            feature = "swagger-ui"
        ))]
        assert_ne!(
            config.path, config.visualizer_path,
            "openapi.json path shouldn't collide with OpenAPI visualizer path"
        );

        #[cfg(feature = "rapidoc")]
        {
            router = router
                .merge(RapiDoc::with_openapi(config.path, openapi).path(config.visualizer_path));
        }

        #[cfg(feature = "redoc")]
        {
            router = router.merge(Redoc::with_url(config.visualizer_path, openapi));
        }

        #[cfg(feature = "scalar")]
        {
            router = router.merge(Scalar::with_url(config.visualizer_path, openapi));
        }

        #[cfg(feature = "swagger-ui")]
        {
            assert_ne!(
                config.visualizer_path, "",
                "Swagger-UI shouldn't be mounted on root!"
            );
            assert_ne!(
                config.visualizer_path, "/",
                "Swagger-UI shouldn't be mounted on root!"
            );
            router = router.merge(SwaggerUi::new(config.visualizer_path).url(config.path, openapi));
        }

        app.add_router(router);
    }
}
