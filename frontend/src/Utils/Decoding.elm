module Utils.Decoding exposing (..)

import Json.Decode exposing (..)
import Model exposing (..)
import Ingredients.Model exposing (..)

decodeStringFloat : Decoder Float
decodeStringFloat =
    let
        parseFloat s =
            String.toFloat s
                |> Maybe.map succeed
                |> Maybe.withDefault (fail "Could not parse float")
    in
    
    string |> andThen parseFloat

