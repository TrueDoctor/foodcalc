-- Guard against concurrent event-inventory links (e.g. a double submit):
-- without a unique constraint two requests can both pass the check-then-insert
-- and insert duplicate rows, inflating the event's covered_kg.
DELETE FROM event_inventories a
USING event_inventories b
WHERE a.event_id = b.event_id
  AND a.inventory_id = b.inventory_id
  AND a.ctid > b.ctid;

ALTER TABLE event_inventories
    ADD CONSTRAINT event_inventories_event_inventory_pk
    PRIMARY KEY (event_id, inventory_id);
