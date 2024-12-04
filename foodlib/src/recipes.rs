use crate::{FoodBase, Ingredient, Unit};
use bigdecimal::BigDecimal;
use eyre::{eyre, Result};
use foodlib_new::entities::recipe as new_recipe;
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgInterval;
use std::fmt::{self, Display, Formatter};
use tabled::Tabled;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Tabled)]
pub struct Recipe {
    pub recipe_id: i32,
    pub name: String,
    #[tabled(display_with = "crate::util::display_optional")]
    pub comment: Option<String>,
}

impl Display for Recipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRecipeIngredient {
    pub ingredient_id: i32,
    pub name: String,
    pub weight: BigDecimal,
    pub energy: BigDecimal,
    pub price: BigDecimal,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubRecipe {
    pub subrecipe_id: i32,
    pub recipe: String,
    pub ingredient: String,
    pub subrecipe: String,
    pub weight: BigDecimal,
    pub is_subrecipe: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum RecipeMetaIngredient {
    Ingredient(Ingredient),
    MetaRecipe(Recipe),
}

impl Default for RecipeMetaIngredient {
    fn default() -> Self {
        Self::Ingredient(Ingredient::default())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct RecipeIngredient {
    pub ingredient: RecipeMetaIngredient,
    pub amount: BigDecimal,
    pub unit: Unit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeStep {
    pub step_id: i32,
    pub step_order: f64,
    pub step_name: String,
    pub step_description: String,
    #[serde(
        serialize_with = "crate::util::serialize_interval",
        deserialize_with = "crate::util::deserialize_interval"
    )]
    pub fixed_duration: PgInterval,
    #[serde(
        serialize_with = "crate::util::serialize_interval",
        deserialize_with = "crate::util::deserialize_interval"
    )]
    pub duration_per_kg: PgInterval,
    pub recipe_id: i32,
}

impl Default for RecipeStep {
    fn default() -> Self {
        Self {
            step_id: Default::default(),
            step_order: Default::default(),
            step_name: Default::default(),
            step_description: Default::default(),
            fixed_duration: PgInterval::try_from(std::time::Duration::from_secs(0)).unwrap(),
            duration_per_kg: PgInterval::try_from(std::time::Duration::from_secs(0)).unwrap(),
            recipe_id: Default::default(),
        }
    }
}

impl RecipeMetaIngredient {
    pub fn name(&self) -> &str {
        match self {
            RecipeMetaIngredient::Ingredient(ingredient) => &ingredient.name,
            RecipeMetaIngredient::MetaRecipe(recipe) => &recipe.name,
        }
    }
}

impl Display for RecipeMetaIngredient {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Display for RecipeIngredient {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ingredient.name())
    }
}

impl From<Recipe> for new_recipe::Recipe {
    fn from(r: Recipe) -> Self {
        new_recipe::Recipe {
            id: r.recipe_id,
            name: r.name,
            comment: r.comment,
        }
    }
}

impl From<new_recipe::Recipe> for Recipe {
    fn from(r: new_recipe::Recipe) -> Self {
        Recipe {
            recipe_id: r.id,
            name: r.name,
            comment: r.comment,
        }
    }
}

impl From<RecipeStep> for new_recipe::RecipeStep {
    fn from(s: RecipeStep) -> Self {
        new_recipe::RecipeStep {
            id: s.step_id,
            recipe_id: s.recipe_id,
            order: s.step_order,
            name: s.step_name,
            description: s.step_description,
            fixed_duration: s.fixed_duration,
            duration_per_kg: s.duration_per_kg,
        }
    }
}

impl From<new_recipe::RecipeStep> for RecipeStep {
    fn from(s: new_recipe::RecipeStep) -> Self {
        RecipeStep {
            step_id: s.id,
            recipe_id: s.recipe_id,
            step_order: s.order,
            step_name: s.name,
            step_description: s.description,
            fixed_duration: s.fixed_duration,
            duration_per_kg: s.duration_per_kg,
        }
    }
}

impl FoodBase {
    pub async fn add_recipe(
        &self,
        name: String,
        energy: BigDecimal,
        comment: Option<String>,
    ) -> Result<Recipe> {
        let recipe = self
            .new_lib
            .recipes()
            .create(new_recipe::Recipe {
                id: -1,
                name,
                comment,
            })
            .await?;
        Ok(recipe.into())
    }

    pub async fn update_recipe(&self, recipe: &Recipe) -> Result<Recipe> {
        let recipe = self.new_lib.recipes().update(recipe.clone().into()).await?;
        Ok(recipe.into())
    }

    pub async fn delete_recipe(&self, id: i32) -> Result<()> {
        Ok(self.new_lib.recipes().delete(id).await?)
    }

    pub async fn get_recipes(&self) -> Result<Vec<Recipe>> {
        let recipes = self.new_lib.recipes().list().await?;
        Ok(recipes.into_iter().map(Into::into).collect())
    }

    pub async fn get_recipe(&self, id: i32) -> Result<Recipe> {
        let recipe = self.new_lib.recipes().get(id).await?;
        Ok(recipe.into())
    }

    pub async fn get_recipe_from_string_reference(&self, reference: String) -> Option<Recipe> {
        if let Ok(recipe_id) = reference.parse::<i32>() {
            self.new_lib
                .recipes()
                .get(recipe_id)
                .await
                .ok()
                .map(Into::into)
        } else {
            self.new_lib
                .recipes()
                .get_by_name(&reference)
                .await
                .ok()
                .map(Into::into)
        }
    }

    pub async fn add_recipe_ingredient(
        &self,
        recipe_id: i32,
        ingredient_id: i32,
        amount: BigDecimal,
        unit_id: i32,
    ) -> Result<()> {
        Ok(self
            .new_lib
            .recipes()
            .add_ingredient(new_recipe::RecipeIngredient {
                recipe_id,
                ingredient_id,
                amount,
                unit_id,
            })
            .await?)
    }

    pub async fn update_recipe_ingredient(
        &self,
        recipe_id: i32,
        ingredient_id: i32,
        amount: BigDecimal,
        unit: i32,
    ) -> Result<()> {
        Ok(self
            .new_lib
            .recipes()
            .update_ingredient(new_recipe::RecipeIngredient {
                recipe_id,
                ingredient_id,
                amount,
                unit_id: unit,
            })
            .await?)
    }

    pub async fn delete_recipe_ingredient(&self, recipe_id: i32, ingredient_id: i32) -> Result<()> {
        Ok(self
            .new_lib
            .recipes()
            .delete_ingredient(recipe_id, ingredient_id)
            .await?)
    }

    pub async fn add_recipe_meta_ingredient(
        &self,
        recipe_id: i32,
        meta_recipe_id: i32,
        weight: BigDecimal,
    ) -> Result<()> {
        Ok(self
            .new_lib
            .recipes()
            .add_meta_ingredient(new_recipe::RecipeMetaIngredient {
                parent_id: recipe_id,
                child_id: meta_recipe_id,
                weight,
            })
            .await?)
    }

    pub async fn update_recipe_meta_ingredient(
        &self,
        recipe_id: i32,
        meta_recipe_id: i32,
        weight: BigDecimal,
    ) -> Result<()> {
        Ok(self
            .new_lib
            .recipes()
            .update_meta_ingredient(new_recipe::RecipeMetaIngredient {
                parent_id: recipe_id,
                child_id: meta_recipe_id,
                weight,
            })
            .await?)
    }

    pub async fn delete_recipe_meta_ingredient(
        &self,
        recipe_id: i32,
        meta_recipe_id: i32,
    ) -> Result<()> {
        Ok(self
            .new_lib
            .recipes()
            .delete_meta_ingredient(recipe_id, meta_recipe_id)
            .await?)
    }

    pub async fn add_recipe_step(&self, step: &RecipeStep) -> Result<RecipeStep> {
        let step = self.new_lib.recipes().add_step(step.clone().into()).await?;
        Ok(step.into())
    }

    pub async fn update_recipe_step(&self, step: &RecipeStep) -> Result<RecipeStep> {
        let step = self
            .new_lib
            .recipes()
            .update_step(step.clone().into())
            .await?;
        Ok(step.into())
    }

    pub async fn delete_step(&self, recipe_id: i32, step_id: i32) -> Result<()> {
        Ok(self
            .new_lib
            .recipes()
            .delete_step(recipe_id, step_id)
            .await?)
    }

    pub async fn get_recipe_steps(&self, recipe_id: i32) -> Result<Vec<RecipeStep>> {
        let steps = self.new_lib.recipes().get_steps(recipe_id).await?;
        Ok(steps.into_iter().map(Into::into).collect())
    }

    pub async fn update_recipe_entries(
        &self,
        recipe_id: i32,
        entries: impl Iterator<Item = RecipeIngredient>,
    ) -> Result<()> {
        let mut regular_ingredients = Vec::new();
        let mut meta_ingredients = Vec::new();

        // Split entries into regular and meta ingredients
        for entry in entries {
            match entry.ingredient {
                RecipeMetaIngredient::Ingredient(ingredient) => {
                    regular_ingredients.push(new_recipe::RecipeIngredient {
                        recipe_id,
                        ingredient_id: ingredient.ingredient_id,
                        amount: entry.amount.clone(),
                        unit_id: entry.unit.unit_id,
                    });
                }
                RecipeMetaIngredient::MetaRecipe(recipe) => {
                    meta_ingredients.push(new_recipe::RecipeMetaIngredient {
                        parent_id: recipe_id,
                        child_id: recipe.recipe_id,
                        weight: entry.amount,
                    });
                }
            }
        }

        Ok(self
            .new_lib
            .recipes()
            .update_recipe_entries(recipe_id, regular_ingredients, meta_ingredients)
            .await?)
    }

    pub async fn update_recipe_steps(
        &self,
        recipe_id: i32,
        entries: impl Iterator<Item = RecipeStep>,
    ) -> Result<()> {
        let entries = entries.map(Into::into).collect();
        Ok(self
            .new_lib
            .recipes()
            .update_recipe_steps(recipe_id, entries)
            .await?)
    }
}
