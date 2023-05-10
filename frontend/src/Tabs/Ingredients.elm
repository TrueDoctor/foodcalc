module Tabs.Ingredients exposing (..)

import Cursor exposing (..)
import Html exposing (Html, div, input, table, tbody, td, tr)
import Html.Attributes exposing (class, placeholder, type_)
import State exposing (Ingredient, IngredientMsg(..), Model, Msg, Tab(..), WebData)
import Util exposing (..)
import Html.Events exposing (onInput)


view : { ingredients : WebData (List Ingredient), filter : String } -> Html Msg
view ingredients =
    let
        list =
            case ingredients.ingredients of
                State.NotAsked ->
                    Html.text "Not Asked"

                State.Loading ->
                    Html.text "Loading"

                State.Success is ->
                    renderIngredients (List.filter (\i -> String.contains (String.toLower ingredients.filter) (String.toLower i.name)) is)

                State.Failure _ ->
                    Html.text "Failure"
    in
    div []
        [ div [] [ input [ class "search", type_ "text", placeholder "Search", onInput <| State.IngredientMessage << EditFilter ] [] ]
        , list
        ]


handleMsg : IngredientMsg -> Model -> ( Model, Cmd Msg )
handleMsg msg model =
    let
        mapTab f tab =
            case tab of
                Ingredients i ->
                    f i

                any ->
                    any
    in
    case msg of
        GotIngredients r ->
            let
                save =
                    mapTab <| \i -> Ingredients { ingredients = mapWebdata r, filter = i.filter }
            in
            ( { model | tabs = Cursor.modifyAt 0 save model.tabs }, Cmd.none )

        EditFilter s ->
            let
                save =
                    mapTab <| \i -> Ingredients { ingredients = i.ingredients, filter = s }
            in
            ( { model | tabs = Cursor.modifyAt 0 save model.tabs }, Cmd.none )

        _ ->
            ( model, Cmd.none )


renderIngredients : List Ingredient -> Html Msg
renderIngredients ingredients =
    table [ roleAttr "grid" ] [ tbody [] (ingredients |> List.map renderSingleIngredient) ]


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
