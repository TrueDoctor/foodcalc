module Recipes.ViewModal exposing (..)

import FeatherIcons as FI
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Model exposing (Msg(..))
import Recipes.Model exposing (..)
import Utils.Main exposing (..)
import Utils.Model exposing (..)
import Utils.View exposing (listView, searchableDropdown)


modal : RecipeTabData -> Html Msg
modal data =
    case data.modal of
        NoModal ->
            Html.node "dialog" [] []

        Add recipe ->
            recipeDetails data "Add" "Add recipe" recipe

        Edit recipe ->
            recipeDetails data "Save" "Edit recipe" recipe


recipeDetails : RecipeTabData -> String -> String -> RecipeEditor -> Html Msg
recipeDetails data submit title editor =
    let
        id_text =
            case editor.id of
                Just i ->
                    " (id: " ++ String.fromInt i ++ ")"

                Nothing ->
                    ""
    in
    Html.node "dialog"
        [ attribute "open" "" ]
        [ article []
            [ header []
                [ a [ onClick <| RecipeMessage CloseModal, href "#" ] [ FI.toHtml [] FI.x ]
                , h3 [] [ Html.text (title ++ id_text) ]
                ]
            , p [ class "container" ]
                [ input
                    [ class "name"
                    , type_ "text"
                    , placeholder "Name"
                    , onInput <| RecipeMessage << ModalMsg << EditName
                    , value editor.name
                    ]
                    []
                , input
                    [ class "comment"
                    , type_ "text"
                    , placeholder "Comment"
                    , onInput <| RecipeMessage << ModalMsg << EditComment
                    , value (Maybe.withDefault "" editor.comment)
                    ]
                    []
                , recipeIngredientsList data editor
                ]
            , footer []
                [ a
                    [ role "button"
                    , class "secondary"
                    , onClick <| RecipeMessage CloseModal
                    , href "#"
                    ]
                    [ Html.text "Cancel" ]
                , a
                    [ role "button"
                    , onClick <| RecipeMessage <| RecipeChanged editor
                    , href "#"
                    ]
                    [ Html.text submit ]
                ]
            ]
        ]


recipeIngredientsList : RecipeTabData -> RecipeEditor -> Html Msg
recipeIngredientsList data editor =
    case editor.ingredients of
        NotAsked ->
            text "Loading ingredients not initiated"

        Loading ->
            text "Loading ingredients ..."

        Failure _ ->
            text "Error loading ingredients"

        Success ingredients ->
            listView (renderRecipeIngredient data) <| (ingredients |> List.map Just) ++ [ Nothing ]



{- [ thead [] [ tr [] [ td [] [ text "Ingredient" ], td [] [ text "Amount" ] ] ]
   , tbody [] <| List.indexedMap (renderRecipeIngredient data editor) <| Nothing :: List.map Just ingredients
   ]
-}


webDataList : WebData (List a) -> List a
webDataList data =
    case data of
        Success items ->
            items

        _ ->
            []


renderRecipeIngredient : RecipeTabData -> Maybe ( WeightedMetaIngredient, RecipeIngredientEditor ) -> List (Html Msg)
renderRecipeIngredient data ingredientEditor =
    let
        ingredient =
            Maybe.map Tuple.first ingredientEditor

        editor =
            ingredientEditor
                |> Maybe.map Tuple.second

        iDropdownData =
            editor
                |> Maybe.map (\e -> e.ingredientDropdown)
                |> Maybe.withDefault (newDropdownData <| Nothing)

        uDropdownData =
            editor
                |> Maybe.map (\e -> e.unitDropdown)
                |> Maybe.withDefault (newDropdownData <| Nothing)

        msg =
            case ingredient of
                Just _ ->
                    RecipeMessage << ModalMsg << EditMetaIngredient (getId ingredient)

                Nothing ->
                    RecipeMessage << ModalMsg << AddMetaIngredient

        ingredientDropdown =
            searchableDropdown
                iDropdownData
                { onFilter = msg << SetIngredientFilter
                , onSelect = msg << SetIngredient
                , property = metaIngredientName << Just
                }
                (webDataList data.allIngredients)

        unitDropdown =
            searchableDropdown
                uDropdownData
                { onFilter = msg << SetUnitFilter
                , onSelect = msg << SetUnit
                , property = \u -> u.name
                }
                (webDataList data.allUnits)

        editAmount =
            input
                [ class "amount"
                , type_ "text"
                , placeholder "Amount"
                , onInput <| msg << SetAmount
                ]
                []

        deleteButton =
            button
                [ class "delete"
                , onClick <| msg Delete
                ]
                [ FI.toHtml [] FI.trash2 ]

        -- ingredientsDropdown2 ingredients editor.filter (Maybe.map ((==) index) editor.activeIngredientIndex |> Maybe.withDefault False) ingredient
    in
    case ingredient of
        Just _ ->
            [ ingredientDropdown
            , editAmount
            , unitDropdown
            , deleteButton
            ]

        Nothing ->
            [ ingredientDropdown
            ]


metaIngredientName : Maybe MetaIngredient -> String
metaIngredientName ig =
    ig
        |> Maybe.map
            (\e ->
                case e of
                    IsDirect i ->
                        i.name

                    IsSubRecipe r ->
                        r.name
            )
        |> Maybe.withDefault ""


picoOption : List (Html Msg) -> Html Msg
picoOption content =
    li [] [ a [] content ]
