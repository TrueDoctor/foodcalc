# Foodcalc

Foodcalc ist ein Planungs- und Verwaltungssystem für das Catering großer Veranstaltungen —
vom Abendessen für 30 Personen bis zur Bewirtung von rund 500 Personen über 7 Tage.
Es ist als self-hosted Anwendung für Gruppen und Vereine gedacht: mehrere Personen planen
gemeinsam Events, kochen nach denselben Unterlagen und behalten Lager und Kosten im Blick.

## Kernidee: Energiebasierte Skalierung

Rezepte skalieren nicht beliebig linear — ein Rezept, das für 4 Personen geschrieben ist,
verhält sich bei 300 Personen anders. Foodcalc skaliert deshalb über die Energie:

- Jede Zutat hat eine **Energiedichte (kJ/g)**; daraus ergibt sich die Energiedichte des Rezepts.
- Für jede Speise/Mahlzeit wird ein **gewünschter Energiebedarf pro Person** und die Personenzahl
  angegeben — daraus berechnet die App die **Kochmenge (kg)**, sodass der Energiebedarf der Leute
  gesichert ist.
- Positionen, die sich nicht über Energie skalieren lassen (Getränke, Stückgut), werden als
  feste Stück-/Mengen-Ziele geplant.

## Funktionen

- **Veranstaltungen** — Budget, mehrere Mahlzeiten über mehrere Tage, Zeitplanung als
  Gantt-artige Timeline (Mahlzeiten, Einkaufstouren, Vorkochen; Drag&Drop), Event-Vorlagen per Klon.
- **Rezepte** — klassifiziert als Mahlzeit oder Komponente (beim Event austauschbar einsetzbar),
  mit Zutaten, Sub-Komponenten und Arbeitsschritten (fixe + mengenabhängige Arbeitszeit,
  HACCP-Kontrollpunkte). Suche/Filter beim Zusammenstellen, Schnell-Anlegen fehlender Zutaten.
- **Zutaten** — Energiedichte, mehrere Eigenschaften (Gewürz, vegan, Allergene — Grundlage für
  Filter und Allergenlisten), Einheitsgewichte für Umrechnungen, Bezugsquellen und Preise
  (inkl. automatischer Preisabruf für Metro).
- **Inventare** — Lagerbestände von Gruppen mit Bestandsabgleich, private Event-Inventare,
  protokollierte Buchungen.
- **Einkaufsplanung** — Einkaufslisten pro Händler (Touren), Abdeckung gegen Bestände mit
  manueller Aufteilung pro Position, sperrbare Listen, Listen-Export.
- **Exporte** — skalierte Koch-PDFs (je Speise oder gesamt), Allergenlisten, Einkaufslisten,
  Event-Übersicht.
- **Rechte** — gruppenscopes Rollenmodell (Leser / Bearbeiter / Verwalter), übertragbare
  Eigentümerschaft (Person ↔ Gruppe), Administratoren ohne pauschalen Ressourcenzugriff.

## Architektur

```
foodcalc/
├── server/                 # REST-API (axum) inkl. PDF-Export (Typst);
│                           # enthält noch die legacy server-rendered UI (maud)
├── foodlib_new/            # Datenlayer: Ops, Entities, Auth, Migrationen (sqlx / PostgreSQL)
├── frontend/spa-frontend/  # SPA: React + TypeScript + Vite (TanStack Query, i18n de/en)
├── foodlib/                # legacy Datenlayer (wird zurückgebaut)
└── cli-client/             # legacy CLI (wird zurückgebaut)
```

Das SPA ersetzt die legacy server-rendered UI schrittweise (Feature-Parität in Arbeit).

## Entwicklung

Voraussetzungen: Rust-Toolchain, PostgreSQL, Node.js/npm.

Backend:

```sh
# DATABASE_URL (PostgreSQL) in foodlib_new/.env setzen, dann:
cd foodlib_new && sqlx migrate run
cargo run -p server
```

Frontend:

```sh
cd frontend/spa-frontend
npm ci
npm run dev   # Entwicklungsserver mit Proxy zum Backend
```

Qualitäts-Gates:

```sh
cd frontend/spa-frontend && npm run build && npm run lint && npm test
cargo check && cargo test -p foodlib_new
```

Frontend-Konventionen (Komponenten-Schichten, i18n-Pflicht, REMOVED-BY-DESIGN-Marker):
siehe `frontend/spa-frontend/README.md`.

## Dokumentation

- [REQUIREMENTS.md](./REQUIREMENTS.md) — Requirements/User Stories (MoSCoW) mit Soll-Ist-Status
- [REST-API-Spezifikation](./server/foodcalc_rest_specification.yaml) — REST-API-Spezifikation

## Status / Fahrplan

Im SPA-Umbau noch offen (Details siehe REQUIREMENTS.md): Stück-Ziele für nicht
energieskalierbare Positionen, Kalkulations-Tab (Kosten je Mahlzeit/Portion/Tour vs. Budget),
Allergenlisten-PDF, globale Dokumentenverwaltung, Nachbestellungs-/Verfügbarkeitsmodul,
OAuth2-Anmeldung (optional).
