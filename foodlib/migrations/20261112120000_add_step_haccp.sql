-- HACCP-Kontrollpunkt-Flag für Arbeitsschritte (UI_Spec: HACCP-Checkpunkte).
ALTER TABLE steps ADD COLUMN IF NOT EXISTS is_haccp BOOLEAN NOT NULL DEFAULT FALSE;
