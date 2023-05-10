module Tabs.Main exposing (..)
import State exposing (WebData, Ingredient, Msg)
import Tabs.Ingredients as Ingredients
import Html exposing (Html)


viewIngredients: WebData (List Ingredient) -> Html Msg
viewIngredients = Ingredients.view