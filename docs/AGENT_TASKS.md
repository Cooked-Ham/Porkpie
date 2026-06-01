# Agent Tasks

The implementation was split into ten ordered tasks:

1. Workspace bootstrap.
2. Security and architecture documentation.
3. Shared domain types.
4. Cryptography.
5. Vault core.
6. SQLite storage.
7. CLI.
8. Dioxus UI.
9. Sync protocol and HTTP API.
10. Import/export and final validation.

Each task has a corresponding specification in `tasks/`. Later tasks depend on the earlier crates being typechecked and tested.
