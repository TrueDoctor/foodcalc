module Main exposing (..)

import Browser
import Cursor
import Html exposing (div, text)
import Http exposing (get)
import Json.Decode exposing (Decoder)
import Navbar exposing (generateNavbar)
import State exposing (Ingredient, Model, Msg(..), RemoteData(..), Tab(..), WebData)
import Decoding exposing (decodeIngredientList)
import Tabs.Main exposing (viewIngredients)


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
    div []
        [ generateNavbar tabName model.tabs
        , renderSelectedView model
        ]


renderSelectedView : Model -> Html.Html Msg
renderSelectedView model =
    case Cursor.active model.tabs of
        Ingredients i ->
            viewIngredients i

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

        GotIngredients r ->
            ( { model | tabs = Cursor.modifyAt 0 (\_ -> Ingredients <| mapWebdata r) model.tabs }, Cmd.none )


mapWebdata : Result Http.Error a -> WebData a
mapWebdata r =
    case r of
        Ok a ->
            Success a

        Err e ->
            Failure (Debug.log "" e)


changeTab : Tab -> Model -> ( Model, Cmd Msg )
changeTab tab model =
    Cursor.setActiveBy (\t -> tabName t == tabName tab) (Debug.log "tabs" model.tabs)
        |> (\c -> ( Debug.log "new tabs" { model | tabs = c }, Cmd.none ))


init : () -> ( Model, Cmd Msg )
init _ =
    let
        i =
            Ingredients
                (Success
                    [ Ingredient "Mehl" 10
                    , Ingredient "Zucker" 11
                    , Ingredient "Salz" 0
                    ]
                )

        r =
            Recipes

        e =
            Events

        tabs =
            Cursor.create (Debug.log "ingredients" i) [ r, e ]
    in
    ( Model tabs
    , get
        { url = backend "/ingredients/list"
        , expect = Http.expectJson GotIngredients decodeIngredientList
        }
    )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }
