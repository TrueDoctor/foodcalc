# Foodcalc

Foodcalc is a self-hosted catering planning and management system for large-scale events — from a 30-person dinner to a 7-day event feeding around 500 people. Built for volunteer groups and associations, it enables teams to collaboratively plan menus, cook from unified production sheets, and track inventory and costs.

## Core Concept: Energy-Based Scaling

An explicit energy target replaces ambiguous portion sizes as the scaling unit. Foodcalc scales production quantities linearly from nutritional energy:

- Every ingredient carries an **energy density (kJ/g)**, from which the recipe's overall energy density is calculated.
- Each meal defines a **target energy per person** and headcount — Foodcalc calculates the required **cooking weight (kg)** so attendees' energy needs are met.
- Items that cannot be scaled by energy (beverages, packaged unit goods) are planned as fixed piece or volume targets.

## Core Features

- **Events** — Budget tracking, multi-day meal schedules, Gantt-style timeline (meals, shopping tours, food prep with drag & drop), and event cloning from templates.
- **Recipes** — Classified as full meals or reusable sub-components, with ingredients, nested components, and preparation steps (fixed + quantity-dependent prep times, HACCP control points). Includes search/filtering and inline creation of missing ingredients.
- **Ingredients** — Energy density, multi-tag properties (spices, vegan, allergens — powering filters and allergen matrices), unit weights for conversions, suppliers, and pricing (including automated Metro price synchronization).
- **Inventories** — Group-level stock tracking with reconciliation, event-scoped inventories, and auditable stock bookings.
- **Shopping Planning** — Supplier-grouped shopping lists (tours), automatic stock deduction with manual per-item split overrides, tour locking, and printable/exportable shopping lists.
- **Exports** — Scaled cooking sheets (per dish or full event), allergen lists and dish labels rendered as Typst PDFs; shopping lists as plain text (optionally Metro CSV). Event-overview PDFs are not yet implemented.
- **Access Control & Auditing** — Group-scoped RBAC (`viewer` / `editor` / `manager`), transferable ownership (user ↔ group), per-event role overrides, and an append-only security audit log.

## Interfaces & Integrations

### REST API (`/api/*`)
Foodcalc provides a JSON REST API backed by a dedicated application layer (`server/src/app/`), exposing events, meals, food prep, shopping tours, recipes, ingredients, inventories, status boards, user/group RBAC management, and audit logs to external tools and modern frontends.
- Full OpenAPI specification: [`docs/openapi.yaml`](docs/openapi.yaml)


### Web Frontend
Currently, Foodcalc provides a server-rendered HTML UI built with `maud` and HTMX/vanilla JS.
It is available at `/`.
> Note: It is planned to be complemented and eventually replaced by a Single Page Application (SPA) via the REST-API

### Planned & Upcoming Features
- **OAuth2 / OIDC Login** *(planned on a dedicated branch)* — Optional single sign-on alongside local password authentication.
- **Single Page Application (SPA)** *(planned on a dedicated branch)* — React + TypeScript + Vite frontend with full REST API coverage, including event planning, recipe management, and inventory tracking.
- **External Food Database Integration (USDA FDC)** *(planned on a dedicated branch)* — Automated ingredient nutrition and allergen lookup.
- **SPA Feature Parity & Enhancements** — Piece-based targets for non-energy-scalable items, cost calculation tab (cost per meal/portion/tour vs. budget), standalone allergen PDFs, global document management, and reorder/availability tracking.

## Architecture

```text
foodcalc/
├── server/                 # Axum web server: JSON REST API (/api/*) and
│                           # legacy server-rendered UI (maud)
├── pdf_export/             # Typst templating and PDF rendering engine
├── foodlib_new/            # Core domain library: ops, entities, RBAC policy, SQLx migrations
├── metro-scrape/           # Metro supplier catalog & price synchronization
├── frontend/spa-frontend/  # React + TypeScript + Vite SPA (on feat/spa-frontend)
├── foodlib/                # Legacy data layer (being phased out)
└── cli-client/             # Legacy CLI client (being phased out)
``` 


## Development

### Prerequisites

* Rust toolchain (stable)
* PostgreSQL
* `sqlx-cli` (for running database migrations)

### Setup & Run

```sh
# Configure DATABASE_URL (PostgreSQL) in foodlib_new/.env, then apply migrations:
cd foodlib_new && sqlx migrate run
cargo run -p server

```

Typst PDF export is optional and kept out of default builds. Enable it with the `typst` feature:

```sh
cargo build -p server --features typst

```

### Quality Gates

```sh
cargo check --workspace
cargo test -p foodlib_new

```

## Documentation

* [`docs/openapi.yaml`](docs/openapi.yaml) — OpenAPI REST specification
