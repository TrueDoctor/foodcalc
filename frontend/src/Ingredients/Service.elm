module Ingredients.Service exposing (..)

import Http exposing (..)
import Ingredients.Model exposing (..)
import Json.Decode as Decode
import Json.Encode as Encode
import Model exposing (..)
import Settings exposing (backend)
import Utils.Decoding exposing (..)


decodeIngredientList : Decode.Decoder (List Ingredient)
decodeIngredientList =
    Decode.list decodeIngredient


decodeIngredient : Decode.Decoder Ingredient
decodeIngredient =
    Decode.map4 Ingredient
        (Decode.field "ingredient_id" Decode.int)
        (Decode.field "name" Decode.string)
        (Decode.field "energy" decodeStringFloat)
        (Decode.field "comment" (Decode.nullable Decode.string))


encodeIngredient : IngredientEditor -> Encode.Value
encodeIngredient ingredient =
    Encode.object
        [ ( "ingredient_id", Maybe.withDefault Encode.null (Maybe.map Encode.int ingredient.id) )
        , ( "name", Encode.string ingredient.name )
        , ( "energy", Encode.string ingredient.energy )
        , ( "comment", Encode.string ingredient.comment )
        ]


fetchIngredients : Cmd IngredientMsg
fetchIngredients =
    get
        { url = backend "/ingredients/list"
        , expect = Http.expectJson (Ingredients.Model.GotWebData << IngredientsList) decodeIngredientList
        }


addOrUpdateIngredient : IngredientEditor -> Cmd IngredientMsg
addOrUpdateIngredient ingredient =
    let
        url =
            case ingredient.id of
                Just id ->
                    "/ingredients/update/" ++ String.fromInt id

                Nothing ->
                    "/ingredients/create"
    in
    post
        { url = backend url
        , body = Http.jsonBody (encodeIngredient ingredient)
        , expect = Http.expectJson (GotWebData << SuccessfulPost) Decode.int
        }
