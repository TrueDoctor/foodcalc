module Utils.Decoding exposing (..)

import Ingredients.Model exposing (..)
import Json.Decode as Decode
import Json.Encode as Encode
import Model exposing (..)
import Utils.Model exposing (Unit)


decodeStringFloat : Decode.Decoder Float
decodeStringFloat =
    let
        parseFloat s =
            String.toFloat s
                |> Maybe.map Decode.succeed
                |> Maybe.withDefault (Decode.fail "Could not parse float")
    in
    Decode.string |> Decode.andThen parseFloat


decodeUnit : Decode.Decoder Unit
decodeUnit =
    Decode.map2 Unit
        (Decode.field "unit_id" Decode.int)
        (Decode.field "name" Decode.string)

encodeUnit : Unit -> Encode.Value
encodeUnit unit =
    Encode.object
        [ ( "unit_id", Encode.int unit.unit_id )
        , ( "name", Encode.string unit.name )
        ]

maybe : (a -> Encode.Value) -> Maybe a -> Encode.Value
maybe f m =
    case m of
        Just value ->
            f value

        Nothing ->
            Encode.null