module Recipes.View exposing (..)

import FeatherIcons as FI
import Html exposing (Html, a, button, div, input, table, tbody, td, text, tr)
import Html.Attributes exposing (class, placeholder, type_)
import Html.Events exposing (onClick, onInput)
import Model exposing (Msg(..))
import Recipes.Model exposing (..)
import Utils.Main exposing (roleAttr)
import Utils.Model exposing (RemoteData(..))
import Recipes.ViewModal exposing (modal)


topBar : Html Msg
topBar =
    table []
        [ tr []
            [ td [] [ input [ class "search", type_ "text", placeholder "Search", onInput <| RecipeMessage << EditFilter ] [] ]
            , td [] [ button [ onClick <| RecipeMessage AddRecipe ] [ FI.toHtml [] FI.plus ] ]
            ]
        ]


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
                    renderRecipes recipes
    in
    div []
        [ topBar
        , modal recipeData
        , list
        ]


renderRecipes : List Recipe -> Html Msg
renderRecipes recipes =
    table [ roleAttr "grid" ] [ tbody [] (List.map renderRecipe recipes) ]


renderRecipe : Recipe -> Html Msg
renderRecipe recipe =
    tr []
        ([ Html.text (String.fromInt recipe.id)
         , Html.text recipe.name
         , Html.text (Maybe.withDefault "" recipe.comment)
         , a [ onClick <| RecipeMessage <| EditRecipe recipe.id ] [ FI.toHtml [] FI.edit ]
         , a [ onClick <| RecipeMessage <| DeleteRecipe recipe.id ] [ FI.toHtml [] FI.trash2 ]
         ]
            |> List.map (\x -> td [] [ x ])
        )

