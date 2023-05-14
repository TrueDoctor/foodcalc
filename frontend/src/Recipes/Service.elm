module Recipes.Service exposing (..)

import Http
import Ingredients.Service exposing (decodeIngredient)
import Json.Decode exposing (..)
import Recipes.Model exposing (..)
import Utils.Decoding exposing (decodeUnit)


decodeRecipe : Decoder Recipe
decodeRecipe =
    map3 Recipe
        (field "recipe_id" int)
        (field "name" string)
        (field "comment" (nullable string))


decodeWeightedMetaIngredients : Decoder (List WeightedMetaIngredient)
decodeWeightedMetaIngredients =
    list decodeWeightedMetaIngredient


decodeWeightedMetaIngredient : Decoder WeightedMetaIngredient
decodeWeightedMetaIngredient =
    map3 WeightedMetaIngredient
        (field "meta_ingredient" decodeMetaIngredient)
        (field "weight" string)
        (field "unit" decodeUnit    )


decodeMetaIngredient : Decoder MetaIngredient
decodeMetaIngredient =
    Json.Decode.oneOf
        [ map IsSubRecipe <| field "MetaRecipe" decodeRecipe
        , map IsDirect <| field "Ingredient" decodeIngredient
        ]


decodeMetaIngredients : Decoder (List MetaIngredient)
decodeMetaIngredients =
    list decodeMetaIngredient


decodeRecipes : Decoder (List Recipe)
decodeRecipes =
    list decodeRecipe


fetchRecipes : Cmd RecipeMsg
fetchRecipes =
    Http.get
        { url = "http://localhost:3000/recipes/list"
        , expect = Http.expectJson (GotWebData << RecipesData) decodeRecipes
        }


fetchAllMetaIngredients : Cmd RecipeMsg
fetchAllMetaIngredients =
    Http.get
        { url = "http://localhost:3000/recipes/meta_ingredients/list"
        , expect = Http.expectJson (GotWebData << MetaIngredientData) decodeMetaIngredients
        }
