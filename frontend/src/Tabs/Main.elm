module Tabs.Main exposing (..)
import State exposing (WebData, Ingredient, Msg)
import Tabs.Ingredients as Ingredients
import Html exposing (Html)


viewIngredients: {ingredients : WebData (List Ingredient), filter: String} -> Html Msg
viewIngredients = Ingredients.view