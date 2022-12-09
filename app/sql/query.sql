WITH RECURSIVE meta AS (
	(SELECT parent_id, child_id, weight FROM meta_recipes)
	UNION (
		SELECT meta.parent_id, meta_recipes.child_id, 1
		FROM meta
		JOIN meta_recipes ON (meta.child_id = meta_recipes.parent_id)
	)
)
SELECT  
(SELECT name from recipes where recipe_id = meta.parent_id) as recipe,
--(SELECT name from recipes where recipe_id = meta.child_id) as sub_recipe,
--(SELECT name from recipes where recipe_id = recipe_ingredients.recipe_id) as recipe_in_ingedients,
(SELECT name from ingredients where ingredient_id = recipe_ingredients.ingredient_id) as ingredient,
SUM(amount), units.name as unit
from meta, recipe_ingredients JOIN units USING(unit_id)

WHERE meta.parent_id = recipe_ingredients.recipe_id OR meta.child_id = recipe_ingredients.recipe_id
GROUP BY recipe_id, ingredient_id, unit_id, parent_id, units.name
ORDER BY recipe, ingredient ASC
;
