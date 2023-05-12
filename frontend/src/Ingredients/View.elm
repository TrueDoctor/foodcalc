module Ingredients.View exposing (..)

import FeatherIcons as FI
import Html exposing (Html, a, article, button, div, footer, h3, i, input, table, tbody, td, tr)
import Html.Attributes exposing (attribute, class, placeholder, type_, value)
import Html.Events exposing (onClick, onInput)
import Ingredients.Model as IM exposing (Ingredient, IngredientEditor, IngredientTabData, Modal)
import Model exposing (..)
import Utils.Cursor exposing (..)
import Utils.Main exposing (..)
import Utils.Model exposing (..)


view : IngredientTabData -> Html Msg
view ingredients =
    let
        list =
            case ingredients.ingredients of
                NotAsked ->
                    Html.text "Not Asked"

                Loading ->
                    Html.text "Loading"

                Success is ->
                    renderIngredients (List.filter (\i -> String.contains (String.toLower ingredients.filter) (String.toLower i.name)) is)

                Failure _ ->
                    Html.text "Failure"
    in
    div []
        [ topBar
        , modal ingredients.modal
        , list
        ]


topBar : Html Msg
topBar =
    table []
        [ tr []
            [ td [] [ input [ class "search", type_ "text", placeholder "Search", onInput <| IngredientMessage << IM.EditFilter ] [] ]
            , td [] [ button [ onClick <| IngredientMessage IM.AddIngredient ] [ FI.toHtml [] FI.plus ] ]
            ]
        ]


modal : Modal -> Html Msg
modal m =
    case m of
        IM.Add ingredient ->
            ingredientDetails "Add" "Add ingredient" ingredient

        IM.Edit ingredient ->
            ingredientDetails "Save" "Edit ingredient" ingredient

        IM.NoModal ->
            Html.node "dialog" [] []


ingredientDetails : String -> String -> IngredientEditor -> Html Msg
ingredientDetails submit title ingredient =
    let 
        id_text = case ingredient.id of
            Nothing -> ""
            Just i -> " (id: " ++ String.fromInt i ++ ")"
    in
    Html.node "dialog"
        [ attribute "open" "" ]
        [ article []
            [ a [ onClick <| IngredientMessage IM.CloseModal ] [ FI.toHtml [] FI.x ]
            , h3 [] [ Html.text (title ++ id_text) ]
            , div [ class "grid" ]
                [ input [ type_ "text", placeholder "name", onInput <| IngredientMessage << IM.ModalMsg << IM.EditName, value ingredient.name ] []
                , input [ type_ "number", placeholder "energy", onInput <| IngredientMessage << IM.ModalMsg << IM.EditEnergy, value ingredient.energy ] []
                ]
            , input [ type_ "text", placeholder "comment", onInput <| IngredientMessage << IM.ModalMsg << IM.EditComment, value ingredient.comment ] []
            , footer [ class "grid" ]
                [ button [ onClick <| IngredientMessage IM.CloseModal ] [ Html.text "Cancel" ]
                , button [ onClick <| IngredientMessage <| IM.IngredientChanged ingredient ] [ Html.text submit ]
                ]
            ]
        ]


renderIngredients : List Ingredient -> Html Msg
renderIngredients ingredients =
    table [ roleAttr "grid" ] [ tbody [] (ingredients |> List.map renderSingleIngredient) ]


renderSingleIngredient : Ingredient -> Html Msg
renderSingleIngredient ingredient =
    tr [ ]
        ([ Html.text (String.fromInt ingredient.id)
         , Html.text ingredient.name
         , Html.text (String.fromFloat ingredient.energy)
         , Html.text (ingredient.comment |> Maybe.withDefault "")
         , a [ onClick <| IngredientMessage <| IM.EditIngredient ingredient.id ] [ FI.toHtml [] FI.edit ]
         , a [ onClick <| IngredientMessage <| IM.DeleteIngredient ingredient.id ] [ FI.toHtml [] FI.trash2 ]
         ]
            |> List.map (\x -> td [] [ x ])
        )
