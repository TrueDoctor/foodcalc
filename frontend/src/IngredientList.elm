module IngredientList exposing (..)

import Element exposing (..)
import Element.Background
import Element.Border
import Element.Input
import Html exposing (a)
import Html.Attributes exposing (name)
import Http
import Ingredients.Model exposing (Ingredient)
import Json.Decode as Decode
import Json.Encode as Encode
import Platform.Cmd as Cmd
import Settings exposing (backend)
import Test.ExpandableList as ExpandableList exposing (ExpandableList, ExpandableListMsg)
import WebData exposing (RemoteData(..), WebData)


type alias IngredientsList =
    WebData (ExpandableList Ingredient IngredientListMsg IngredientMsg)


type Ingredients
    = Ingredients IngredientsList


type alias IngredientData =
    { name : String, id : Maybe Int, energy : String, comment : Maybe String }


type Ingredient
    = Ingredient { data : IngredientData, edit : IngredientData }


type IngredientMsg
    = NameChange String
    | EnergyChange String
    | CommentChange String


type IngredientListMsg
    = ListMsg (ExpandableListMsg Ingredient IngredientMsg)
    | GotIngredients (Result Http.Error (List Ingredient))
    | UpdateSuccessful (Result Http.Error Int)
    | Save Ingredient
    | Cancel Ingredient


stateOf : String -> List ( Bool, Ingredient ) -> IngredientsList
stateOf search items =
    let
        filter : String -> Ingredient -> Bool
        filter string ingredient =
            case ingredient of
                Ingredient { data } ->
                    String.contains (String.toLower string) (String.toLower data.name)
    in
    Success
        { search = search
        , filter = filter
        , items = items
        , viewElement = viewIngredient
        , mapMsg = ListMsg
        , update = updateIngredient
        , add =
            Just
                (\() ->
                    newIngredient Nothing "" "" (Just "")
                )
        }


init : IngredientsList
init =
    stateOf "" [ ( False, newIngredient (Just 1) "test" "1" (Just "") ) ]


newIngredient : Maybe Int -> String -> String -> Maybe String -> Ingredient
newIngredient id name energy comment =
    Ingredient { data = { id = id, name = name, energy = energy, comment = comment }, edit = { id = id, name = name, energy = energy, comment = comment } }


viewExpanded : Ingredient -> Element IngredientListMsg
viewExpanded ingredient =
    case ingredient of
        Ingredient { edit } ->
            column [ Element.Background.color (rgb 1 1 1), width fill, padding 10, spacing 10, Element.Border.rounded 5 ]
                [ row
                    [ width fill
                    , spacing 25
                    ]
                    [ Element.map
                        (ListMsg << ExpandableList.mapElementMsg ingredient)
                        (Element.Input.text []
                            { onChange = NameChange
                            , label = Element.Input.labelLeft [] (text "Name:")
                            , placeholder = Just (Element.Input.placeholder [] (text "Name"))
                            , text = edit.name
                            }
                        )
                    , Element.map
                        (ListMsg << ExpandableList.mapElementMsg ingredient)
                        (Element.Input.text []
                            { onChange = EnergyChange
                            , label = Element.Input.labelLeft [] (text "Energy:")
                            , placeholder = Just (Element.Input.placeholder [] (text "Energy"))
                            , text = edit.energy
                            }
                        )
                    ]
                , Element.map
                    (ListMsg << ExpandableList.mapElementMsg ingredient)
                    (Element.Input.text []
                        { onChange = CommentChange
                        , label = Element.Input.labelLeft [] (text "Comment:")
                        , placeholder = Just (Element.Input.placeholder [] (text "Comment"))
                        , text = Maybe.withDefault "" edit.comment
                        }
                    )
                , row [ width fill, spacing 25 ]
                    [ Element.Input.button [ alignRight ] { onPress = Just (Save ingredient), label = el [ padding 10 ] <| text "Save" }
                    , Element.Input.button [ alignRight ] { onPress = Just (Cancel ingredient), label = el [ padding 10 ] <| text "Cancel" }
                    ]
                ]


viewRow : IngredientData -> Element msg
viewRow data =
    row [ spaceEvenly, width fill, paddingXY 50 20 ]
        [ el [ width (fillPortion 1) ] (text (Maybe.withDefault "" (Maybe.map String.fromInt data.id)))
        , el [ width (fillPortion 4) ] (text data.name)
        , el [ width (fillPortion 2) ] (text data.energy)
        , el [ width (fillPortion 5) ] (text (Maybe.withDefault "" data.comment))
        ]


viewIngredient : Attribute IngredientListMsg -> Bool -> Ingredient -> Element IngredientListMsg
viewIngredient expand expanded ingredient =
    case ingredient of
        Ingredient { data } ->
            column [ width fill ]
                [ el [ expand, width fill ] (viewRow data)
                , if expanded then
                    viewExpanded ingredient

                  else
                    none
                ]


view : IngredientsList -> Element IngredientListMsg
view model =
    case model of
        Success data ->
            ExpandableList.view data

        Failure _ ->
            el [] (text "Failed to load ingredients")

        _ ->
            el [] (text "Loading")


update : IngredientListMsg -> IngredientsList -> ( IngredientsList, Cmd IngredientListMsg )
update msg model =
    let
        id ig =
            case ig of
                Ingredient { data } ->
                    data.id

        updateIg e =
            case e of
                Ingredient { data } ->
                    List.map
                        (\( ex, ig ) ->
                            if id ig == id e then
                                ( False, Ingredient { data = data, edit = data } )

                            else
                                ( ex, ig )
                        )
    in
    case ( msg, model ) of
        ( ListMsg m, Success data ) ->
            Tuple.mapFirst Success <| ExpandableList.update m data

        ( GotIngredients result, _ ) ->
            case result of
                Ok list ->
                    ( stateOf "" (List.map (Tuple.pair False) list), Cmd.none )

                Err e ->
                    ( Failure e, Cmd.none )

        ( UpdateSuccessful result, _ ) ->
            case result of
                Ok _ ->
                    ( model, fetchIngredients )

                Err _ ->
                    ( model, Cmd.none )

        ( Save ig, Success list ) ->
            ( Success list, addOrUpdateIngredient ig )

        ( Cancel ig, Success list ) ->
            ( Success { list | items = updateIg ig list.items }, Cmd.none )

        _ ->
            ( model, Cmd.none )


updateIngredient : IngredientMsg -> Ingredient -> ( Ingredient, Cmd IngredientListMsg )
updateIngredient msg ig =
    let
        updateEdit f =
            case ig of
                Ingredient i ->
                    let
                        edit =
                            i.edit
                    in
                    Ingredient { i | edit = f edit }
    in
    case msg of
        NameChange name ->
            ( updateEdit (\e -> { e | name = name }), Cmd.none )

        CommentChange comment ->
            ( updateEdit (\e -> { e | comment = Just comment }), Cmd.none )

        EnergyChange energy ->
            ( updateEdit (\e -> { e | energy = energy }), Cmd.none )


validate : Ingredient -> Maybe Ingredient
validate ingredient =
    case ingredient of
        Ingredient { data, edit } ->
            if data == edit then
                Nothing

            else
                Maybe.map (\_ -> ingredient) (String.toFloat edit.energy)


decodeIngredientList : Decode.Decoder (List Ingredient)
decodeIngredientList =
    Decode.list decodeIngredient


decodeIngredient : Decode.Decoder Ingredient
decodeIngredient =
    Decode.map4 newIngredient
        (Decode.field "ingredient_id" (Decode.nullable Decode.int))
        (Decode.field "name" Decode.string)
        (Decode.field "energy" Decode.string)
        (Decode.field "comment" (Decode.nullable Decode.string))


encodeIngredient : Ingredient -> Maybe Encode.Value
encodeIngredient ingredient =
    Maybe.map
        (\changed ->
            case changed of
                Ingredient { edit } ->
                    Encode.object
                        [ ( "ingredient_id", Maybe.withDefault Encode.null (Maybe.map Encode.int edit.id) )
                        , ( "name", Encode.string edit.name )
                        , ( "energy", Encode.string edit.energy )
                        , ( "comment", Maybe.withDefault Encode.null (Maybe.map Encode.string edit.comment) )
                        ]
        )
        (validate ingredient)


fetchIngredients : Cmd IngredientListMsg
fetchIngredients =
    Http.get
        { url = backend "/ingredients/list"
        , expect = Http.expectJson GotIngredients decodeIngredientList
        }


addOrUpdateIngredient : Ingredient -> Cmd IngredientListMsg
addOrUpdateIngredient ingredient =
    let
        url =
            case ingredient of
                Ingredient { edit } ->
                    case edit.id of
                        Just id ->
                            "/ingredients/update/" ++ String.fromInt id

                        Nothing ->
                            "/ingredients/create"
    in
    encodeIngredient ingredient
        |> Maybe.map
            (\body ->
                Http.post
                    { url = backend url
                    , body = Http.jsonBody body
                    , expect = Http.expectJson UpdateSuccessful Decode.int
                    }
            )
        |> Maybe.withDefault Cmd.none
