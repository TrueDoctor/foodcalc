module EventList exposing (EventListMsg, Events, emptyEvents, update, view, fetchEvents)

import Element exposing (..)
import Element.Background
import Element.Border
import Element.Input
import Http
import Json.Decode as Decode
import RecipesList exposing (Recipe, WebDataMsg, view, viewExpanded)
import Settings exposing (backend)
import Test.ExpandableList as ExpandableList exposing (ExpandableList, ExpandableListMsg, mapElementMsg)
import Test.StringUtils exposing (fuzzyContains)
import Test.Styles exposing (white)
import WebData exposing (RemoteData(..), WebData)


type alias EventList =
    { events : WebData (ExpandableList Event EventListMsg EventMsg)
    , recipes : WebData (List Recipe)
    , places : WebData (List Place)
    }


type Events
    = Events EventList


type alias Place =
    { id : Int, name : String }


type alias EventData =
    { id : Maybe Int
    , name : String
    , comment : Maybe String
    , budget : Maybe String
    }


type Event
    = Event { data : EventData, edit : EventData }


type EventListMsg
    = ListMsg (ExpandableListMsg Event EventMsg)
    | GotWebData WebDataMsg


type WebDataMsg
    = GotEvents (Result Http.Error (List Event))


type EventMsg
    = NameChange String
    | CommentChange String
    | BudgetChange String
    | MealChange
    | Save
    | Cancel
    | FetchData



-- Setup


emptyEvents : Events
emptyEvents =
    Events { events = NotAsked, recipes = NotAsked, places = NotAsked }


stateOf : String -> List ( Bool, Event ) -> WebData (ExpandableList Event EventListMsg EventMsg)
stateOf search items =
    let
        filter : String -> Event -> Bool
        filter string event =
            case event of
                Event { data } ->
                    fuzzyContains string data.name
    in
    Success
        { search = search
        , filter = filter
        , items = items
        , viewElement = viewEvent
        , mapMsg = ListMsg
        , update = updateEvent
        , add = Just <| always <| newEvent Nothing "" Nothing (Just "")
        , expandItem = Just FetchData
        }


newEvent : Maybe Int -> String -> Maybe String -> Maybe String -> Event
newEvent id name budget comment =
    let
        data =
            { id = id, name = name, budget = budget, comment = comment }
    in
    Event { data = data, edit = data }



-- View


view : Events -> Element EventListMsg
view events =
    let
        model =
            case events of
                Events m ->
                    m
    in
    case model.events of
        Success data ->
            ExpandableList.view data

        Failure _ ->
            el [] <| text "Failed to load events"

        _ ->
            el [] <| text "Loading"


viewEvent : Attribute EventListMsg -> Bool -> Event -> Element EventListMsg
viewEvent expand expanded ev =
    case ev of
        Event { edit } ->
            column [ width fill ]
                [ el [ expand, width fill ] (viewRow edit)
                , if expanded then
                    viewExpanded ev

                  else
                    none
                ]


viewRow : EventData -> Element msg
viewRow data =
    row [ spaceEvenly, width fill, paddingXY 50 20]
        [ el [ width (fillPortion 1) ] <| text <| Maybe.withDefault "" <| Maybe.map String.fromInt data.id
        , el [ width (fillPortion 3) ] <| text data.name
        , el [ width (fillPortion 2) ] <| text <| Maybe.withDefault "" data.budget
        , el [ width (fillPortion 5) ] <| text <| Maybe.withDefault "" data.comment
        ]


viewExpanded : Event -> Element EventListMsg
viewExpanded ev =
    let
        viewNameBudget edit =
            row [ width fill, spacing 20]
                [ Element.Input.text []
                    { onChange = NameChange
                    , label = Element.Input.labelAbove [] (text "Name")
                    , placeholder = Just (Element.Input.placeholder [] (text "Name"))
                    , text = edit.name
                    }
                , Element.Input.text []
                    { onChange = BudgetChange
                    , label = Element.Input.labelAbove [] <| text "Budget"
                    , placeholder = Just <| Element.Input.placeholder [] <| text "Budget"
                    , text = Maybe.withDefault "" edit.budget
                    }
                ]

        viewComment edit =
            Element.Input.text []
                { onChange = CommentChange
                , label = Element.Input.labelAbove [] (text "Comment")
                , placeholder = Just (Element.Input.placeholder [] (text "Comment"))
                , text = Maybe.withDefault "" edit.comment
                }

        viewButtons =
            row [ width fill, spacing 25 ]
                [ Element.Input.button [ alignRight ]
                    { onPress = Just Save
                    , label = el [ padding 10 ] <| text "Save"
                    }
                , Element.Input.button [ alignRight ]
                    { onPress = Just Cancel
                    , label = el [ padding 10 ] <| text "Cancel"
                    }
                ]
    in
    case ev of
        Event { edit } ->
            Element.map (ListMsg << mapElementMsg ev) <|
                column
                    [ Element.Background.color white
                    , width fill
                    , padding 10
                    , spacing 10
                    , Element.Border.rounded 5
                    ]
                    [ viewNameBudget edit
                    , viewComment edit
                    , viewButtons
                    ]



-- Update


update : EventListMsg -> Events -> ( Events, Cmd EventListMsg )
update msg model =
    let
        mod =
            case model of
                Events d ->
                    d
    in
    case ( msg, mod.events ) of
        ( ListMsg m, Success data ) ->
            Tuple.mapFirst
                (\result -> Events { mod | events = Success result })
                (ExpandableList.update m data)

        ( GotWebData wd, _ ) ->
            let
                ( new, cmd ) =
                    handleWebData wd mod
            in
            ( Events new, cmd )

        _ ->
            ( model, Cmd.none )


updateEvent : EventMsg -> Event -> ( Event, Cmd EventListMsg )
updateEvent msg event =
    case msg of
        _ ->
            ( event, Cmd.none )


handleWebData : WebDataMsg -> EventList -> ( EventList, Cmd EventListMsg )
handleWebData msg model =
    case ( msg, model.events ) of
        ( GotEvents result, Success list ) ->
            case result of
                Ok new ->
                    ( { model | events = Success { list | items = List.map (Tuple.pair False) new } }, Cmd.none )

                Err e ->
                    ( { model | events = Failure e }, Cmd.none )

        ( GotEvents result, _ ) ->
            case result of
                Ok new ->
                    ( { model | events = stateOf "" <| List.map (Tuple.pair False) new }, Cmd.none )

                Err e ->
                    ( { model | recipes = Failure e }, Cmd.none )



-- Decoding


decodeEvent =
    Decode.map4
        (\id name budget comment ->
            let
                data =
                    { id = id, name = name, comment = comment, budget = budget }
            in
            Event { data = data, edit = data }
        )
        (Decode.field "event_id" <| Decode.nullable Decode.int)
        (Decode.field "name" Decode.string)
        (Decode.field "budget" <| Decode.nullable Decode.string)
        (Decode.field "comment" <| Decode.nullable Decode.string)


decodeEvents =
    Decode.list decodeEvent



-- fetching


fetchEvents : Cmd EventListMsg
fetchEvents =
    Http.get { url = backend "/events/list", expect = Http.expectJson (GotWebData << GotEvents) decodeEvents }
