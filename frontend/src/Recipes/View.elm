module Recipes.View exposing (..)
import Recipes.Model exposing (..)
import Html exposing (Html, div, text, table, tr, td, input, button, tbody,a)
import Html.Attributes exposing (class, type_, placeholder)
import Html.Events exposing (onInput, onClick)
import Model exposing (Msg)
import Utils.Model exposing (RemoteData(..))
import FeatherIcons as FI
import Model exposing (Msg(..))
import Utils.Main exposing (roleAttr)


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

                    text ("Error loading recipes" )
                Success recipes ->
                    renderRecipes recipes
    in
        div [] [ topBar,list ]

renderRecipes : List Recipe -> Html Msg
renderRecipes recipes =
    table [roleAttr "grid"] [tbody [] (List.map renderRecipe recipes)]

renderRecipe : Recipe -> Html Msg
renderRecipe recipe =
    tr [] 
        ([Html.text (String.fromInt recipe.id)
        , Html.text recipe.name
        , Html.text (Maybe.withDefault "" recipe.comment)
        , a [ onClick <| RecipeMessage <| EditRecipe recipe.id ] [ FI.toHtml [] FI.edit ]
         , a [ onClick <| RecipeMessage <| DeleteRecipe recipe.id ] [ FI.toHtml [] FI.trash2 ]
         ] |> List.map (\x -> td [] [x]) )
