module Ingredients.Main exposing (..)

import Html exposing (Html)
import Ingredients.Model exposing (..)
import Ingredients.Update exposing (handleMsg)
import Ingredients.View exposing (view)
import Model exposing (Model, Msg, Tab(..))

viewIngredients : IngredientTabData -> Html Msg
viewIngredients =
    view


handleIngredientsMsg : IngredientMsg -> Model -> ( Model, Cmd Msg )
handleIngredientsMsg =
    handleMsg
