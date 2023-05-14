module Main exposing (..)

import Browser
import Html exposing (div, text)
import Html.Attributes exposing (class)
import Ingredients.Main exposing (handleIngredientsMsg, viewIngredients)
import Ingredients.Model as IModel exposing (IngredientMsg(..), IngredientTabData)
import Ingredients.Service exposing (fetchIngredients)
import Model exposing (..)
import Navbar exposing (generateNavbar)
import Recipes.Main exposing (handleRecipesMsg, viewRecipes)
import Recipes.Model as RModel exposing (RecipeTabData)
import Settings exposing (..)
import Utils.Cursor
import Utils.Model exposing (RemoteData(..))


view : Model -> Html.Html Msg
view model =
    div [ class "container" ]
        [ generateNavbar tabName model.tabs
        , renderSelectedView model
        ]


renderSelectedView : Model -> Html.Html Msg
renderSelectedView model =
    case Utils.Cursor.active model.tabs of
        Ingredients i ->
            viewIngredients i

        Recipes r ->
            viewRecipes r

        Events ->
            text "Events"


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        None ->
            ( model, Cmd.none )

        ChangeTab tab ->
            changeTab tab model

        IngredientMessage m ->
            handleIngredientsMsg m model

        RecipeMessage m ->
            handleRecipesMsg m model


initTab : Model -> ( Model, Cmd Msg )
initTab model =
    case model.tabs.active of
        Ingredients _ ->
            ( model, Cmd.none )

        Recipes _ ->
            update (RecipeMessage RModel.InitTab) model

        Events ->
            ( model, Cmd.none )


changeTab : Tab -> Model -> ( Model, Cmd Msg )
changeTab tab model =
    let
        c =
            Utils.Cursor.setActiveBy (\t -> tabName t == tabName tab) model.tabs
    in
    initTab (Model c)


init : () -> ( Model, Cmd Msg )
init _ =
    let
        tabs =
            Utils.Cursor.create (Ingredients <| IngredientTabData Loading "" IModel.NoModal)
                [ Recipes <| RecipeTabData NotAsked "" RModel.NoModal NotAsked
                , Events
                ]
    in
    ( Model tabs
    , Cmd.map IngredientMessage fetchIngredients
    )


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.none


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }
