module Recipes.Update exposing (..)

import Model exposing (Model, Msg(..), Tab(..))
import Recipes.Model exposing (..)
import Recipes.Service exposing (fetchAllMetaIngredients, fetchRecipes)
import Svg.Attributes exposing (from)
import Utils.Cursor
import Utils.Main exposing (mapWebdata)
import Utils.Model exposing (RemoteData(..))
import Recipes.Service exposing (fetchRecipeIngredients)


mapTab : (RecipeTabData -> Tab) -> Tab -> Tab
mapTab f tab =
    case tab of
        Recipes r ->
            f r

        any ->
            any


recipeList : Model -> Maybe (List Recipe)
recipeList model =
    case model.tabs.active of
        Recipes r ->
            case r.recipes of
                Success recipes ->
                    Just recipes

                _ ->
                    Nothing

        _ ->
            Nothing


updateModel : (Tab -> Tab) -> Model -> Model
updateModel f model =
    { model | tabs = Utils.Cursor.modifyAt 1 f model.tabs }


handleWebData : RecipeWebData -> Model -> ( Model, Cmd Msg )
handleWebData result model =
    case result of
        RecipesData recipes ->
            let
                save =
                    mapTab <| \r -> Recipes <| { r | recipes = mapWebdata recipes }
            in
            ( updateModel save model, Cmd.none )

        MetaIngredientData meta ->
            let
                save =
                    mapTab <| \r -> Recipes <| { r | allIngredients = mapWebdata meta }
            in
            ( updateModel save model, Cmd.none )

        RecipeIngredientData meta ->
            ( updateModel (mapModalUpdate <| \editor -> { editor | ingredients = mapWebdata meta }) model, Cmd.none )


handleMsg : RecipeMsg -> Model -> ( Model, Cmd Msg )
handleMsg msg model =
    case msg of
        GotWebData data ->
            handleWebData data model

        InitTab ->
            let
                save =
                    mapTab <| \r -> Recipes <| { r | recipes = Utils.Model.Loading }
            in
            ( updateModel save model
            , Cmd.batch
                [ Cmd.map RecipeMessage fetchAllMetaIngredients
                , Cmd.map RecipeMessage fetchRecipes
                ]
            )

        AddRecipe ->
            let
                save =
                    mapTab <| \r -> Recipes <| { r | modal = Add <| emptyRecipeEditor }
            in
            ( updateModel save model, Cmd.map RecipeMessage fetchRecipes )

        EditFilter filter ->
            let
                save =
                    mapTab <| \r -> Recipes <| { r | filter = filter }
            in
            ( updateModel save model, Cmd.none )

        EditRecipe id ->
            let
                editor =
                    recipeList model
                        |> Maybe.map (List.filter <| \r -> r.id == id)
                        |> Maybe.andThen List.head
                        |> Maybe.map editorFromReipe
                        |> Maybe.withDefault emptyRecipeEditor

                save =
                    mapTab <| \r -> Recipes <| { r | modal = Edit editor }
            in
            ( updateModel save model, Cmd.map RecipeMessage (fetchRecipeIngredients id)  )

        CloseModal ->
            let
                save =
                    mapTab <| \r -> Recipes <| { r | modal = NoModal }
            in
            (  updateModel save model, Cmd.none )

        

        ModalMsg m ->
            handleModalMsg m model

        _ ->
            ( model, Cmd.none )


updateModal : Modal -> (RecipeEditor -> RecipeEditor) -> Modal
updateModal modal f =
    case modal of
        Edit e ->
            Edit (f e)

        Add e ->
            Add (f e)

        any ->
            any


mapModalUpdate : (RecipeEditor -> RecipeEditor) -> Tab -> Tab
mapModalUpdate f =
    mapTab <| \i -> Recipes { i | modal = updateModal i.modal f }


handleModalMsg : ModalMsg -> Model -> ( Model, Cmd Msg )
handleModalMsg msg model =
    case msg of
        EditComment comment ->
            ( updateModel (mapModalUpdate <| \e -> { e | comment = Just comment }) model, Cmd.none )

        EditName name ->
            ( updateModel (mapModalUpdate <| \e -> { e | name = name }) model, Cmd.none )

        EditActiveIngredientIndex index ->
            ( updateModel (mapModalUpdate <| \e -> { e | activeIngredientIndex = Just index }) model, Cmd.none )

        EditIngredientFilter filter ->
            ( updateModel (mapModalUpdate <| \e -> { e | filter = filter }) model, Cmd.none )

        _ ->
            ( model, Cmd.none )
