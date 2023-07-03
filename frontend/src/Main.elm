module Main exposing (..)

import Browser
import Events
import Html exposing (div)
import Html.Attributes exposing (class)
import Ingredients.Main exposing (handleIngredientsMsg, viewIngredients)
import Ingredients.Model as IModel exposing (IngredientMsg(..), emptyIngredientsTabData)
import Model exposing (..)
import Navbar exposing (generateNavbar)
import Recipes.Main exposing (handleRecipesMsg, viewRecipes)
import Recipes.Model as RModel exposing (emptyRecipeTabData)
import Settings exposing (..)
import Utils.Cursor
import Utils.Model exposing (RemoteData(..))
import Recipes.Model exposing (RecipeTabData)
import Browser.Dom exposing (Element)
import Element
import IngredientList


tabName : Tab -> String
tabName tab =
    case tab of
        Ingredients _ ->
            "Ingredients"

        Recipes _ ->
            "Recipes"

        Events ->
            "Events"


view : Model -> Html.Html Msg
view model =
    div [ class "container" ]
        [ generateNavbar tabName model.tabs
        , renderSelectedView model
        ]

viewUI : Model -> Html.Html Msg
viewUI m = 
    Element.layout [] (Element.map Model.IngredientUIMsg (IngredientList.view m.ingredientList))


renderSelectedView : Model -> Html.Html Msg
renderSelectedView model =
    case Utils.Cursor.active model.tabs of
        Ingredients i ->
            viewIngredients i

        Recipes r ->
            viewRecipes r

        Events ->
            Html.map EventsMessage <| Events.viewEvents model.events


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

        EventsMessage e ->
            let
                ( events, cmd ) =
                    Events.handleEventTabMsg e model.events
            in
            ( { model | events = events }
            , Cmd.map EventsMessage cmd
            )

        IngredientUIMsg m -> 
            let
                
              (list, cmd) = IngredientList.update m model.ingredientList
            in
                ({model | ingredientList = list}, Cmd.map IngredientUIMsg cmd)


initTab : Model -> ( Model, Cmd Msg )
initTab model =
    case model.tabs.active of
        Ingredients _ ->
            update (IngredientMessage IModel.InitTab) model

        Recipes _ ->
            update (RecipeMessage RModel.InitTab) model

        Events ->
            update (EventsMessage Events.init) model


changeTab : Tab -> Model -> ( Model, Cmd Msg )
changeTab tab model =
    let
        c =
            Utils.Cursor.setActiveBy (\t -> tabName t == tabName tab) model.tabs
    in
    initTab { model | tabs = c }


init : () -> ( Model, Cmd Msg )
init _ =
    let
        (ingredientsTabData, recipeTabData, eventsData) =
            (emptyIngredientsTabData, emptyRecipeTabData, Events.emptyEventsData)
        
        ingredientsList = IngredientList.init


        tabs =
            Utils.Cursor.create (Ingredients ingredientsTabData)
                [ Recipes recipeTabData
                , Events
                ]
    in
    ( Model tabs ingredientsTabData recipeTabData eventsData ingredientsList
    , Cmd.map (always <| ChangeTab <| Ingredients emptyIngredientsTabData) Cmd.none
    )


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.none


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , view = viewUI
        , update = update
        , subscriptions = subscriptions
        }
