module Ingredients.Service exposing (..)

import Http exposing (..)
import Ingredients.Model exposing (..)
import Json.Decode exposing (..)
import Model exposing (..)
import Settings exposing (backend)
import Utils.Decoding exposing (..)


decodeIngredientList : Decoder (List Ingredient)
decodeIngredientList =
    list decodeIngredient


decodeIngredient : Decoder Ingredient
decodeIngredient =
    map4 Ingredient
        (field "ingredient_id" int)
        (field "name" string)
        (field "energy" decodeStringFloat)
        (field "comment" (nullable string))


fetchIngredients : Cmd IngredientMsg
fetchIngredients =
    get
        { url = backend "/ingredients/list"
        , expect = Http.expectJson Ingredients.Model.GotIngredients decodeIngredientList
        }
