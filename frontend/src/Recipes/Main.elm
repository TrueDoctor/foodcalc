module Recipes.Main exposing (..)

import Recipes.View exposing (view)
import Recipes.Model exposing (RecipeTabData)
import Model exposing (Msg)
import Html exposing (Html)
import Model exposing (Model)
import Recipes.Model exposing (RecipeMsg)
import Recipes.Update exposing (handleMsg)

viewRecipes : RecipeTabData -> Html Msg
viewRecipes = view

handleRecipesMsg : RecipeMsg -> Model -> (Model, Cmd Msg)
handleRecipesMsg = handleMsg
