module Decoding exposing (..)

import Json.Decode as Decode
import State exposing (..)

decodeStringFloat : Decode.Decoder Float
decodeStringFloat =
    let
        parseFloat s =
            String.toFloat s
                |> Maybe.map Decode.succeed
                |> Maybe.withDefault (Decode.fail "Could not parse float")
    in
    
    Decode.string |> Decode.andThen parseFloat

decodeIngredientList : Decode.Decoder (List Ingredient)
decodeIngredientList =
    Decode.list decodeIngredient

decodeIngredient : Decode.Decoder Ingredient
decodeIngredient =
    Decode.map2 Ingredient
        (Decode.field "name" Decode.string)
        (Decode.field "energy" decodeStringFloat)