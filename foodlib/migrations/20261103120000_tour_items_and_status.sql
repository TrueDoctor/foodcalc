-- Manual per-tour item pins: an overlay on the view-derived shopping-list
-- assignment. A pinned ingredient is FORCED onto its tour (and hidden from
-- "without tour"), independent of store/date-window matching.
CREATE TABLE public.shopping_tour_items (
    tour_id INTEGER NOT NULL REFERENCES shopping_tours(tour_id) ON DELETE CASCADE,
    ingredient_id INTEGER NOT NULL REFERENCES ingredients(ingredient_id) ON DELETE CASCADE,
    PRIMARY KEY (tour_id, ingredient_id)
);

-- Tour lifecycle: a row here marks the tour as completed (locked). Keeping
-- status in its own table avoids touching the hot shopping-list views.
CREATE TABLE public.shopping_tour_completed (
    tour_id INTEGER PRIMARY KEY REFERENCES shopping_tours(tour_id) ON DELETE CASCADE,
    completed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
