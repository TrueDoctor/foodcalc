module Recipes.Update exposing (..)

import Ingredients.Model exposing (Ingredient)
import Model exposing (Model, Msg(..), Tab(..))
import Recipes.Model exposing (..)
import Recipes.Service exposing (addOrUpdateRecipe, fetchAllMetaIngredients, fetchRecipeIngredients, fetchRecipes, fetchUnits, updateRecipeExtras)
import Regex
import Utils.Cursor
import Utils.Decoding
import Utils.Main exposing (mapWebdata, toWebdata)
import Utils.Model exposing (RemoteData(..), Unit)


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
                    mapTab <| \r -> Recipes <| { r | recipes = toWebdata recipes }
            in
            ( updateModel save model, Cmd.none )

        MetaIngredientData meta ->
            let
                save =
                    mapTab <| \r -> Recipes <| { r | allIngredients = toWebdata meta }
            in
            ( updateModel save model, Cmd.none )

        UnitData units ->
            let
                save =
                    mapTab <| \r -> Recipes <| { r | allUnits = toWebdata units }
            in
            ( updateModel save model, Cmd.none )

        RecipeIngredientData meta ->
            let
                wd =
                    toWebdata meta
            in
            case Debug.log "" wd of
                Success ingredients ->
                    let
                        newRecipeIngredients =
                            ingredients
                                |> List.map
                                    (\i ->
                                        ( i, buildEditor i )
                                    )

                        save =
                            mapModalUpdate <| \e -> { e | ingredients = Success newRecipeIngredients }
                    in
                    ( updateModel save model, Cmd.none )

                _ ->
                    ( model, Cmd.none )

        RecipeId editor meta ->
            case meta of
                Ok id ->
                    ( model, Cmd.map RecipeMessage <| updateRecipeExtras editor id )

                _ ->
                    ( model, Cmd.none )

        PostResult _ ->
            ( model, Cmd.none )


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
                , Cmd.map RecipeMessage fetchUnits
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
            ( updateModel save model, Cmd.map RecipeMessage (fetchRecipeIngredients id) )

        CloseModal ->
            let
                save =
                    mapTab <| \r -> Recipes <| { r | modal = NoModal }
            in
            ( updateModel save model, Cmd.none )

        RecipeChanged _ ->
            let
                modal =
                    case model.tabs.active of
                        Recipes r ->
                            r.modal

                        _ ->
                            NoModal
            in
            ( model, Cmd.map RecipeMessage <| addOrUpdateRecipe modal )

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
    let
        defaultIngredient =
            WeightedMetaIngredient (IsDirect <| Ingredient -1 "" 0 Nothing) "" (Unit 0 "")

        addEntry entry =
            updateModel (mapModalUpdate (\e -> { e | ingredients = mapWebdata (\d -> d ++ [ entry ]) e.ingredients })) model
    in
    case msg of
        EditComment comment ->
            ( updateModel (mapModalUpdate <| \e -> { e | comment = Just comment }) model, Cmd.none )

        EditName name ->
            ( updateModel (mapModalUpdate <| \e -> { e | name = name }) model, Cmd.none )

        EditMetaIngredient id recipeIngredientMsg ->
            handleMetaIngredientMsg recipeIngredientMsg id model

        AddMetaIngredient recipeIngredientMsg ->
            Debug.log "" <| handleMetaIngredientMsg recipeIngredientMsg (IngredientId -1) (addEntry ( defaultIngredient, buildEditor defaultIngredient ))

        EditStep stepMsg id ->
            handleStepMsg stepMsg id model


isId : MetaId -> { a | metaIngredient : MetaIngredient } -> Bool
isId id meta =
    case ( meta.metaIngredient, id ) of
        ( IsDirect ig, IngredientId i ) ->
            ig.id == i

        ( IsSubRecipe sr, SubRecipeId i ) ->
            sr.id == i

        _ ->
            False


handleMetaIngredientMsg : RecipeIngredientMsg -> MetaId -> Model -> ( Model, Cmd Msg )
handleMetaIngredientMsg msg id model =
    let
        mapIf check f =
            List.map
                (\i ->
                    if check i then
                        f i

                    else
                        i
                )

        save f =
            mapModalUpdate <| \e -> { e | ingredients = mapWebdata (mapIf (isId id << Tuple.first) f) e.ingredients }

        new : ( WeightedMetaIngredient, RecipeIngredientEditor ) -> ( WeightedMetaIngredient, RecipeIngredientEditor )
        new ( i, e ) =
            let
                ingredientDropdown =
                    e.ingredientDropdown

                unitDropdown =
                    e.unitDropdown
            in
            case msg of
                SetIngredientFilter filter ->
                    ( i, { e | ingredientDropdown = { ingredientDropdown | filter = filter } } )

                SetUnitFilter filter ->
                    ( i, { e | unitDropdown = { unitDropdown | filter = filter } } )

                SetIngredient ingredient ->
                    ( { i | metaIngredient = ingredient }
                    , { e | ingredientDropdown = { ingredientDropdown | selected = Just ingredient, open = False } }
                    )

                SetUnit unit ->
                    ( { i | unit = unit }
                    , { e | unitDropdown = { unitDropdown | selected = Just unit, open = False } }
                    )

                SetAmount amount ->
                    if Regex.contains Utils.Decoding.floatRegex amount then
                        ( { i | amount = amount }, e )

                    else
                        ( i, e )

                _ ->
                    ( i, e )
    in
    case msg of
        Delete ->
            ( updateModel (mapModalUpdate <| \e -> { e | ingredients = mapWebdata (List.filter (not << isId id << Tuple.first)) e.ingredients }) model, Cmd.none )

        _ ->
            ( updateModel (save new) model, Cmd.none )


handleStepMsg : StepMsg -> Int -> Model -> ( Model, Cmd Msg )
handleStepMsg msg id model =
    Debug.todo "handleStepMsg"
