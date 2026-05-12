mod controllers;
mod consumers;

use summer::{App, auto_config};
use summer_pubsub::PubSubPlugin;
use summer_sqlx::SqlxPlugin;
use summer_web::{WebConfigurator, WebPlugin};

#[auto_config(WebConfigurator, PubSubConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .use_config_file(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/pubsub-example/config/app.toml"
        ))
        .add_plugin(WebPlugin)
        .add_plugin(PubSubPlugin)
        .add_plugin(SqlxPlugin)
        .run()
        .await;
}
