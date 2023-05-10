module Main exposing (..)

import Browser
import Cursor
import Decoding exposing (decodeIngredientList)
import Html exposing (div, text)
import Http exposing (get)
import Navbar exposing (generateNavbar)
import State exposing (..)
import Tabs.Main exposing (viewIngredients)
import Tabs.Ingredients exposing (handleMsg)
import Html.Attributes exposing (class)


tabName : Tab -> String
tabName tab =
    case tab of
        Ingredients _ ->
            "Ingredients"

        Recipes ->
            "Recipes"

        Events ->
            "Events"


backend : String -> String
backend path =
    "http://localhost:3000" ++ path


view : Model -> Html.Html Msg
view model =
    div [class "container"]
        [ generateNavbar tabName model.tabs
        , renderSelectedView model
        ]


renderSelectedView : Model -> Html.Html Msg
renderSelectedView model =
    case Cursor.active model.tabs of
        Ingredients i ->
            viewIngredients i.ingredients

        Recipes ->
            text "Recipes"

        Events ->
            text "Events"


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        None ->
            ( model, Cmd.none )

        ChangeTab tab ->
            changeTab (Debug.log "new tab" tab) model

        IngredientMessage m ->
            handleMsg m model




changeTab : Tab -> Model -> ( Model, Cmd Msg )
changeTab tab model =
    Cursor.setActiveBy (\t -> tabName t == tabName tab) (Debug.log "tabs" model.tabs)
        |> (\c -> ( { model | tabs = c }, Cmd.none ))


init : () -> ( Model, Cmd Msg )
init _ =
    let
        tabs =
            Cursor.create (Ingredients { ingredients = NotAsked, filter = "" })
                [ Recipes
                , Events
                ]
    in
    ( Model tabs
    , get
        { url = backend "/ingredients/list"
        , expect = Http.expectJson (IngredientMessage << GotIngredients) decodeIngredientList
        }
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
