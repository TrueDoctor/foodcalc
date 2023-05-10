module Tabs.Ingredients exposing (..)

import Cursor exposing (..)
import Html exposing (Html, div, input, table, td, tr)
import Html.Attributes exposing (class, placeholder, type_)
import State exposing (Ingredient, IngredientMsg(..), Model, Msg, Tab(..), WebData)
import Util exposing (mapWebdata, roleAttr)
import Html exposing (tbody)


view : WebData (List Ingredient) -> Html Msg
view ingredients =
    let
        list =
            case ingredients of
                State.NotAsked ->
                    Html.text "Not Asked"

                State.Loading ->
                    Html.text "Loading"

                State.Success is ->
                    renderIngredients is

                State.Failure _ ->
                    Html.text "Failure"
    in
    div []
        [ div [] [ input [ class "search", type_ "text", placeholder "Search" ] [] ]
        , list
        ]


handleMsg : IngredientMsg -> Model -> ( Model, Cmd Msg )
handleMsg msg model =
    case msg of
        GotIngredients r ->
            let
                save tab =
                    case tab of
                        Ingredients i ->
                            Ingredients { ingredients = mapWebdata r, filter = i.filter }

                        any ->
                            any
            in
            ( { model | tabs = Cursor.modifyAt 0 save model.tabs }, Cmd.none )

        _ ->
            ( model, Cmd.none )


renderIngredients : List Ingredient -> Html Msg
renderIngredients ingredients =
    table [ roleAttr "grid" ] [ tbody [] (ingredients |> List.map renderSingleIngredient)]


renderSingleIngredient : Ingredient -> Html Msg
renderSingleIngredient ingredient =
    tr [ class "ingredient-item" ]
        ([ Html.text (String.fromInt ingredient.id)
         , Html.text ingredient.name
         , Html.text (String.fromFloat ingredient.energy)
         , Html.text (ingredient.comment |> Maybe.withDefault "")
         ]
            |> List.map (\x -> td [] [ x ])
        )
