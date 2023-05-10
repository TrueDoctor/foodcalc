module Decoding exposing (..)

import Json.Decode exposing (..)
import State exposing (..)

decodeStringFloat : Decoder Float
decodeStringFloat =
    let
        parseFloat s =
            String.toFloat s
                |> Maybe.map succeed
                |> Maybe.withDefault (fail "Could not parse float")
    in
    
    string |> andThen parseFloat

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