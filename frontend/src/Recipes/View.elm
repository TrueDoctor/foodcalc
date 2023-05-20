module Recipes.View exposing (..)

import FeatherIcons as FI
import Html exposing (Html, a, div, text)
import Html.Events exposing (onClick)
import Model exposing (Msg(..))
import Recipes.Model exposing (..)
import Recipes.ViewModal exposing (modal)
import Utils.Model exposing (RemoteData(..))
import Utils.View exposing (filterListView)
import Utils.Main exposing (nameFilter)


view : RecipeTabData -> Html Msg
view recipeData =
    let
        list =
            case recipeData.recipes of
                NotAsked ->
                    text "Loading not initiated"

                Loading ->
                    text "Loading..."

                Failure _ ->
                    text "Error loading recipes"

                Success recipes ->
                    filterListView
                        { row = renderRecipe
                        , filter = \r -> nameFilter recipeData.filter r.name
                        , filterChange = RecipeMessage << EditFilter
                        , onAdd = RecipeMessage AddRecipe
                        }
                        recipes
    in
    div []
        [ list
        , modal recipeData
        ]


renderRecipe : Recipe -> List (Html Msg)
renderRecipe recipe =
    [ Html.text (String.fromInt recipe.id)
    , Html.text recipe.name
    , Html.text (Maybe.withDefault "" recipe.comment)
    , a [ onClick <| RecipeMessage <| EditRecipe recipe.id ] [ FI.toHtml [] FI.edit ]
    , a [ onClick <| RecipeMessage <| DeleteRecipe recipe.id ] [ FI.toHtml [] FI.trash2 ]
    ]
