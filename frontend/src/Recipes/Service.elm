module Recipes.Service exposing (..)

import Http
import Json.Decode exposing (..)
import Recipes.Model exposing (..)


decodeRecipe : Decoder Recipe
decodeRecipe =
    Json.Decode.map3 Recipe
        (field "recipe_id" int)
        (field "name" string)
        (field "comment" (nullable string))


decodeRecipes : Decoder (List Recipe)
decodeRecipes =
    list decodeRecipe


fetchRecipes : Cmd RecipeMsg
fetchRecipes =
    Http.get
        { url = "http://localhost:3000/recipes/list"
        , expect = Http.expectJson GotRecipes decodeRecipes
        }
