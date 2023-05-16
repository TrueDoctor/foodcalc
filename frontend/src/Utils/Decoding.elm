module Utils.Decoding exposing (..)

import Ingredients.Model exposing (..)
import Json.Decode exposing (..)
import Model exposing (..)
import Utils.Model exposing (Unit)


decodeStringFloat : Decoder Float
decodeStringFloat =
    let
        parseFloat s =
            String.toFloat s
                |> Maybe.map succeed
                |> Maybe.withDefault (fail "Could not parse float")
    in
    string |> andThen parseFloat


decodeUnit : Decoder Unit
decodeUnit =
    map2 Unit
        (field "unit_id" int)
        (field "name" string)

