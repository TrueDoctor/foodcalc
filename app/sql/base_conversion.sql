CREATE OR REPLACE VIEW conversions AS (
    WITH RECURSIVE conversion_table AS (
        (SELECT from_unit, to_unit, from_amount, to_amount FROM base_conversions)
        UNION (
            SELECT conversion_table.from_unit, base_conversions.to_unit, conversion_table.from_amount, conversion_table.to_amount * (base_conversions.to_amount / base_conversions.from_amount)
            FROM conversion_table JOIN base_conversions ON (conversion_table.to_unit = conversion_table.from_unit)
            UNION
            SELECT to_unit, from_unit, to_amount, from_amount
            FROM conversion_table
        )
)

CREATE OR REPLACE VIEW ingredient_weight AS (
    SELECT ingredient_id, unit_id, weight FROM weights;
    UNION
    SELECT ingredient_id, from_unit, to_amount \ from_amount
        FROM ingredients, conversions
        WHERE to_unit = 0
)
