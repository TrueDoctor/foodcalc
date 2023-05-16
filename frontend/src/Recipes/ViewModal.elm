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
            listView (renderRecipeIngredient data editor 0) ((ingredients |> List.map Just) ++ [ Nothing ])



{- [ thead [] [ tr [] [ td [] [ text "Ingredient" ], td [] [ text "Amount" ] ] ]
   , tbody [] <| List.indexedMap (renderRecipeIngredient data editor) <| Nothing :: List.map Just ingredients
   ]
-}


renderRecipeIngredient : RecipeTabData -> RecipeEditor -> Int -> Maybe WeightedMetaIngredient -> List (Html Msg)
renderRecipeIngredient data editor index ingredient =
    let
        dropdown =
            case data.allIngredients of
                NotAsked ->
                    text "Loading ingredients not initiated"

                Loading ->
                    text "Loading ingredients ..."

                Failure _ ->
                    text "Error loading ingredients"

                Success ingredients ->
                    searchableDropdown
                        { filterChange = RecipeMessage << ModalMsg << EditIngredientFilter
                        , onSelect = \i -> RecipeMessage <| ModalMsg <| EditIngredient { old = ingredient, new = Maybe.map2 (\x y -> { x | metaIngredient = y }) ingredient i }
                        , property = metaIngredientName
                        , filter = editor.filter
                        , list = List.map Just ingredients
                        , selected = Maybe.map (\i -> i.metaIngredient) ingredient
                        }

        -- ingredientsDropdown2 ingredients editor.filter (Maybe.map ((==) index) editor.activeIngredientIndex |> Maybe.withDefault False) ingredient
    in
    [ dropdown
    , input
        [ class "amount"
        , type_ "text"
        , placeholder "Amount"
        , onInput <| RecipeMessage << ModalMsg << EditIngredientAmount
        , onFocus <| RecipeMessage <| ModalMsg <| EditActiveIngredientIndex index
        ]
        []
    ]


ingredientsDropdown2 : List MetaIngredient -> String -> Bool -> Maybe WeightedMetaIngredient -> Html Msg
ingredientsDropdown2 ingredients filter hasDropdown selected =
    let
        visible =
            summary
                [ attribute "aria-haspopup" "listbox" ]
                [ text <| metaIngredientName <| Maybe.map (\x -> x.metaIngredient) selected ]

        search =
            input
                [ class "search"
                , type_ "text"
                , placeholder "Search"
                , onInput <| RecipeMessage << ModalMsg << EditIngredientFilter
                , value filter
                ]
                []
    in
    details [ role "list" ]
        [ visible
        , ul
            [ role "listbox" ]
            (if hasDropdown then
                picoOption [ search ]
                    :: dropdownList2
                        (ingredients
                            |> List.filter (String.contains filter << metaIngredientName << Just)
                        )
                        (Maybe.map (\x -> x.metaIngredient) selected)

             else
                [ visible ]
            )
        ]


ingredientsDropdown : List MetaIngredient -> String -> Bool -> Maybe WeightedMetaIngredient -> Html Msg
ingredientsDropdown ingredients filter hasDropdown selected =
    let
        visible =
            option [] [ text <| metaIngredientName <| Maybe.map (\x -> x.metaIngredient) selected ]

        search =
            input
                [ class "search"
                , type_ "text"
                , placeholder "Search"
                , onInput <| RecipeMessage << ModalMsg << EditIngredientFilter
                , value <| metaIngredientName (selected |> Maybe.map (\x -> x.metaIngredient))
                ]
                []
    in
    select []
        (if hasDropdown then
            option [] [ search ]
                :: dropdownList
                    (ingredients
                        |> List.filter (String.contains filter << metaIngredientName << Just)
                    )
                    (Maybe.map (\x -> x.metaIngredient) selected)

         else
            [ visible ]
        )


dropdownList2 : List MetaIngredient -> Maybe MetaIngredient -> List (Html Msg)
dropdownList2 ingredients selected =
    List.map
        (\x ->
            picoOption
                [ text <| metaIngredientName <| Just x ]
        )
        ingredients


dropdownList : List MetaIngredient -> Maybe MetaIngredient -> List (Html Msg)
dropdownList ingredients selected =
    List.map
        (\x ->
            option
                [ value <| metaIngredientName <| Just x ]
                [ text <| metaIngredientName <| Just x ]
        )
        ingredients


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
