# Collections
`collections::Collection` is a trait that represents basic information a collection of data.

# Operations
sqlx-compatible operations 

# JsonClient versions
In the codebase, I have three versions of the json client:

1. json_client.rs
2. json_client_v0.rs
3. json_client_v1.rs

V0 and V1 are kept for backward compatibility, and will be removed in the future. do not introduce any breaking changes to them. When I use json_client, JsonClient, or json client, client here I'm referring to the latest version (json_client.rs).

# JsonClient (latest version)

Do not depend on serde or serde_json, instead use gen_serde.rs code.

json_client::client_interface::ops is a central place for incoorpirating all json_client::$(operation)_mod::*, you can only update the only one ops invocation if you implement a new operation.