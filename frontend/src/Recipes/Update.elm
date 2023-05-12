module Recipes.Update exposing (..)
import Recipes.Model exposing (RecipeMsg)
import Model exposing (Model)
import Recipes.Model exposing (RecipeMsg(..))
import Recipes.Model exposing (RecipeTabData)
import Model exposing (Tab)
import Model exposing (Tab(..))
import Utils.Main exposing (mapWebdata)
import Utils.Cursor
import Model exposing (Msg)
import Model exposing (Msg(..))
import Recipes.Service exposing (fetchRecipes)
import Utils.Model



mapTab : (RecipeTabData -> Tab) -> Tab -> Tab
mapTab f tab =
    case tab of
        Recipes r ->
            f r

        any ->
            any

updateModel : (Tab -> Tab) -> Model -> Model
updateModel f model =
    { model | tabs = Utils.Cursor.modifyAt 1 f model.tabs }

handleMsg : RecipeMsg -> Model -> ( Model, Cmd Msg )
handleMsg msg model =
    case msg of
        GotRecipes result ->
            let
                save = mapTab <| \r -> Recipes <| { r | recipes = mapWebdata result }
            in
            ( updateModel save model, Cmd.none )    

        InitTab ->
            let
                save = mapTab <| \r -> Recipes <| { r | recipes = Utils.Model.Loading }
            in
            ( updateModel save model, Cmd.map RecipeMessage fetchRecipes )
        _ ->
            ( model, Cmd.none )