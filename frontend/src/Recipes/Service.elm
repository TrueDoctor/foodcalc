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


decodeNestedWeightedMetaIngredients : Decoder (List WeightedMetaIngredient)
decodeNestedWeightedMetaIngredients =
    list decodeNestedWeightedMetaIngredient


decodeNestedWeightedMetaIngredient : Decoder WeightedMetaIngredient
decodeNestedWeightedMetaIngredient =
    map3 WeightedMetaIngredient
        (field "ingredient" decodeMetaIngredient)
        (field "amount" string)
        (field "unit" decodeUnit)



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


fetchRecipeIngredients : Int -> Cmd RecipeMsg
fetchRecipeIngredients recipeId =
    Http.get
        { url = "http://localhost:3000/recipes/" ++ String.fromInt recipeId ++ "/meta_ingredients/list"
        , expect = Http.expectJson (GotWebData << RecipeIngredientData) decodeNestedWeightedMetaIngredients
        }

