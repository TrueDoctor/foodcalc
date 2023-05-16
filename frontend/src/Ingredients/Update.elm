module Ingredients.Update exposing (handleMsg)

import Ingredients.Model exposing (..)
import Ingredients.Service exposing (addOrUpdateIngredient)
import Model exposing (..)
import Utils.Cursor
import Utils.Main exposing (..)
import Utils.Model exposing (RemoteData(..))


editor : Ingredients.Model.IngredientTabData -> Int -> Modal
editor itab id =
    case itab.ingredients of
        Success ingredients ->
            ingredients
                |> List.filter (\i -> i.id == id)
                |> List.head
                |> Maybe.map (\i -> Edit (IngredientEditor (Just i.id) i.name (String.fromFloat i.energy) (i.comment |> Maybe.withDefault "")))
                |> Maybe.withDefault itab.modal

        _ ->
            itab.modal


mapTab : (IngredientTabData -> Tab) -> Tab -> Tab
mapTab f tab =
    case tab of
        Ingredients i ->
            f i

        any ->
            any


updateModel : (Tab -> Tab) -> Model -> Model
updateModel f model =
    { model | tabs = Utils.Cursor.modifyAt 0 f model.tabs }


handleMsg : IngredientMsg -> Model -> ( Model, Cmd Msg )
handleMsg msg model =
    case msg of
        GotWebData data ->
            handleWebData data model

        EditFilter s ->
            let
                save =
                    mapTab <| \i -> Ingredients { i | filter = s }
            in
            ( updateModel save model, Cmd.none )

        AddIngredient ->
            let
                save =
                    mapTab <| \i -> Ingredients { i | modal = Add (IngredientEditor Nothing "" "" "") }
            in
            ( updateModel save model, Cmd.none )

        EditIngredient id ->
            let
                save =
                    mapTab <| \i -> Ingredients { i | modal = editor i id }
            in
            ( updateModel save model, Cmd.none )

        CloseModal ->
            let
                save =
                    mapTab <| \i -> Ingredients { i | modal = Ingredients.Model.NoModal }
            in
            ( updateModel save model, Cmd.none )

        ModalMsg m ->
            handleModalMsg m model

        _ ->
            ( model, Cmd.none )


handleWebData : IngredientWebData -> Model -> ( Model, Cmd Msg )
handleWebData data model =
    case data of
        IngredientsList ingredients ->
            let
                save =
                    mapTab <| \i -> Ingredients { i | ingredients = mapWebdata ingredients }
            in
            ( updateModel save model, Cmd.none )

        _ ->
            ( model, Cmd.none )


handleModalMsg : ModalMsg -> Model -> ( Model, Cmd Msg )
handleModalMsg msg model =
    let
        update modal f =
            case modal of
                Edit e ->
                    Edit (f e)

                Add e ->
                    Add (f e)

                any ->
                    any

        mapUpdate f =
            mapTab <| \i -> Ingredients { i | modal = update i.modal f }
    in
    case msg of
        EditName name ->
            ( updateModel (mapUpdate (\e -> { e | name = name })) model, Cmd.none )

        EditEnergy energy ->
            ( updateModel (mapUpdate (\e -> { e | energy = energy })) model, Cmd.none )

        EditComment comment ->
            ( updateModel (mapUpdate (\e -> { e | comment = comment })) model, Cmd.none )

        Save e ->
            let
                save =
                    mapTab <| \i -> Ingredients { i | modal = Ingredients.Model.NoModal }
                
            in
            ( updateModel save model
            , Cmd.batch
                [ Cmd.map IngredientMessage (addOrUpdateIngredient e)
                , Cmd.map (\_ -> IngredientMessage CloseModal) Cmd.none
                ]
            )
