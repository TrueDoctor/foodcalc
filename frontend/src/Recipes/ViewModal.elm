module Recipes.ViewModal exposing (..)

import FeatherIcons as FI
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Ingredients.Model exposing (Ingredient)
import Model exposing (Msg(..))
import Recipes.Model exposing (..)
import Utils.Main exposing (..)
import Utils.Model exposing (..)
import Utils.View exposing (listView, searchableDropdown, showWebData)


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
            [ a [ onClick <| RecipeMessage CloseModal ] [ FI.toHtml [] FI.x ]
            , h3 [] [ Html.text (title ++ id_text) ]
            , input [ class "name", type_ "text", placeholder "Name", onInput <| RecipeMessage << ModalMsg << EditName ] []
            , input [ class "comment", type_ "text", placeholder "Comment", onInput <| RecipeMessage << ModalMsg << EditComment ] []
            , recipeIngredientsList data editor
            , footer [ class "grid" ]
                [ button [ onClick <| RecipeMessage CloseModal ] [ Html.text "Cancel" ]
                , button [ onClick <| RecipeMessage <| RecipeChanged editor ] [ Html.text submit ]
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
            listView renderRecipeIngredient ((ingredients |> List.map Just) ++ [ Nothing ])



{- [ thead [] [ tr [] [ td [] [ text "Ingredient" ], td [] [ text "Amount" ] ] ]
   , tbody [] <| List.indexedMap (renderRecipeIngredient data editor) <| Nothing :: List.map Just ingredients
   ]
-}


renderRecipeIngredient : Maybe ( WeightedMetaIngredient, RecipeIngredientEditor ) -> List (Html Msg)
renderRecipeIngredient ingredientEditor =
    let
        ingredient =
            Maybe.map Tuple.first ingredientEditor

        ingredientDropdownData =
            ingredientEditor
                |> Maybe.map Tuple.second
                |> Maybe.map (\e -> e.ingredientDropdown)
                |> Maybe.withDefault (newDropdownData [] <| IsDirect <| Ingredient -1 "" 0 Nothing)

        unitDropdownData =
            ingredientEditor
                |> Maybe.map Tuple.second
                |> Maybe.map (\e -> e.unitDropdown)
                |> Maybe.withDefault (newDropdownData [] <| Unit -1 "")

        id =
            case ingredient of
                Just ig ->
                    case ig.metaIngredient of
                        IsDirect i ->
                            IngredientId i.id

                        IsSubRecipe r ->
                            SubRecipeId r.id

                _ ->
                    NewId

        ingredientDropdown =
            searchableDropdown
                ingredientDropdownData
                { onFilter = RecipeMessage << ModalMsg << EditMetaIngredient id << SetIngredientFilter
                , onSelect = RecipeMessage << ModalMsg << EditMetaIngredient id << SetIngredient
                , property = metaIngredientName << Just
                }

        unitDropdown =
            searchableDropdown
                unitDropdownData
                { onFilter = RecipeMessage << ModalMsg << EditMetaIngredient id << SetUnitFilter
                , onSelect = RecipeMessage << ModalMsg << EditMetaIngredient id << SetUnit
                , property = \u -> u.name
                }
        deleteButton =
            button
                [ class "delete"
                , onClick <| RecipeMessage <| ModalMsg <| EditMetaIngredient id <| Delete
                ]
                [ FI.toHtml [] FI.delete ]

        -- ingredientsDropdown2 ingredients editor.filter (Maybe.map ((==) index) editor.activeIngredientIndex |> Maybe.withDefault False) ingredient
    in
    [ ingredientDropdown
    , input
        [ class "amount"
        , type_ "text"
        , placeholder "Amount"
        , onInput <| RecipeMessage << ModalMsg << EditMetaIngredient id << SetAmount
        ]
        []
        , unitDropdown

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
