# Pubsub Example

For this example, we will use the Pubsub API to send and receive messages, so you need to have a Pubsub topic created and a service account with the necessary permissions.

Then save the credentials to `examples/pubsub-example/credentials.json` and start Postgres (from the `summer-pubsub` crate root):

```bash
docker compose -f examples/pubsub-example/compose.yaml up -d
```

Then run the example:

```bash
cargo run --example pubsub-example
```

Visiting the endpoint `http://localhost:8000/auth/signup` will send a message to the Pubsub topic `users_created` and the consumer will receive the message.

The `project_id` in `examples/pubsub-example/config/app.toml` must match the project id of the credentials file.
