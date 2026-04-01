# ClawStudio v2.0 - Unit Tests

This directory contains unit tests for all ClawStudio modules.

## Test Structure

```
tests/
├── rust/                    # Rust backend tests
│   ├── setup_test.rs        # Environment detection tests
│   ├── gateway_test.rs      # Gateway lifecycle tests
│   ├── audit_test.rs        # Audit logging tests
│   ├── channel_test.rs      # Channel management tests
│   ├── template_test.rs     # Template import/export tests
│   └── docker_test.rs       # Docker sandbox tests
│
├── ts/                      # TypeScript frontend tests
│   ├── stores.test.ts       # Pinia store tests
│   ├── utils.test.ts        # Utility function tests
│   └── components.test.ts   # Component tests
│
└── e2e/                     # End-to-end tests
    └── setup-wizard.test.ts # Full installation flow
```

## Running Tests

### Rust Tests
```bash
cd src-tauri
cargo test
```

### TypeScript Tests
```bash
pnpm test
```

### All Tests
```bash
pnpm test:all
```
