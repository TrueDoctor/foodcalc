module Tabs.Ingredients exposing (..)

import Html exposing (Html, div, li)
import State exposing (Ingredient, Msg, Tab(..), WebData)
import Html.Attributes exposing (class)
import Html exposing (ul)
import Html exposing (span)
import Html exposing (tr)
import Html exposing (table)
import Html exposing (td)
import Json.Decode as Decode



view : WebData (List Ingredient) -> Html Msg
view ingredients =
    case ingredients of
        State.NotAsked ->
            div [] [ Html.text "Not Asked" ]

        State.Loading ->
            div [] [ Html.text "Loading" ]

        State.Success is ->
            renderIngredients is

        State.Failure _ ->
            div [] [ Html.text "Failure" ]

renderIngredients : List Ingredient -> Html Msg
renderIngredients ingredients =
    div [] [table [] (ingredients |> List.map renderSingleIngredient)]

renderSingleIngredient : Ingredient -> Html Msg
renderSingleIngredient ingredient =
    tr [class "ingredient-item"] ([ 
        Html.text ingredient.name 
        , Html.text (String.fromFloat ingredient.energy)
        ] |> List.map (\x -> td [] [x]))