module Events exposing
    (  -- MODEL
       EventTabMsg

    , EventsData
    , emptyEventsData
    ,  -- UPDATE
       handleEventTabMsg

    ,  -- Model read/write
       init

    , viewEvents
    )

import FeatherIcons as FI
import Html exposing (Html, a, div, span, text)
import Html.Attributes
import Html.Events
import Http
import Json.Decode as Decode
import Modal
import SearchList exposing (SearchList)
import Settings exposing (backend)
import Utils.Main exposing (mapWebdata, propertyFilter)
import Utils.Model exposing (RemoteData(..), WebData)



-- MODEL


type EventsData
    = Data
        { events : WebData (SearchList Event EventMsg)
        , modal : Maybe EventDetails
        }


type Event
    = Exists { name : String, budget : String, id : Int, comment : Maybe String }
    | NewEvent { name : String, budget : String, comment : Maybe String }


type EventDetails
    = Details { event : Event, details : WebData (SearchList Meal MealMsg) }


type Meal
    = Meal
        { event_id : Int
        , recipe_id : Int
        , recipe_name : String
        , commaent : Maybe String
        , place_id : Int
        , place_name : String
        , start_time : String
        , end_time : String
        , weight : Float
        , energy : Float
        , price : String
        , servings : Int
        }



-- MESSAGES


type EventTabMsg
    = EventListMsg (SearchList.SearchListMsg Event EventMsg)
    | EventDetails EventDetails ModificationMsg
    | GotWebData WebDataMsg
    | OpenModal EventDetails
    | SaveModal EventDetails
    | CloseModal
    | InitTab


type EventMsg
    = EditEvent Int
    | AddEvent
    | DeleteEvent Int



type ModificationMsg
    = Name String
    | Budget String
    | Comment String
    | MealModification


type MealMsg
    = MealMsg


type WebDataMsg
    = EventList (Result Http.Error (List Event))



-- Model read/write


emptyEventsData : EventsData
emptyEventsData =
    Data { events = NotAsked, modal = Nothing }


eventName : Event -> String
eventName event =
    case event of
        Exists { name } ->
            name

        NewEvent _ ->
            "<Add Event>"


newEventsList : WebData (List Event) -> WebData (SearchList Event EventMsg)
newEventsList webData =
    mapWebdata
        (\list ->
            SearchList.new
                (propertyFilter eventName)
                viewEvent
                list
        )
        webData


init : EventTabMsg
init =
    InitTab



-- VIEW


viewEvents : EventsData -> Html EventTabMsg
viewEvents data =
    case data of
        Data { events, modal } ->
            case events of
                NotAsked ->
                    div [] [ text "NotAsked" ]

                Loading ->
                    div [] [ text "Loading" ]

                Failure _ ->
                    div [] [ text "Error loading Events" ]

                Success searchList ->
                    case modal of
                        Just m ->
                            div []
                                [ Html.map EventListMsg <| SearchList.view searchList
                                , viewEventDetails (m)
                                ]
                        Nothing ->
                            div []
                                [ Html.map EventListMsg <| SearchList.view searchList
                                ]


viewEvent : Event -> List (Html EventMsg)
viewEvent event =
    case event of
        Exists { id, name, budget, comment } ->
            [ span [] [ text (String.fromInt id) ]
            , span [] [ text name ]
            , span [] [ text budget ]
            , span [] [ text (Maybe.withDefault "" comment) ]
            , a [ Html.Attributes.href "#", Html.Events.onClick (EditEvent id) ] [ FI.toHtml [] FI.edit ]
            , a [ Html.Attributes.href "#", Html.Events.onClick (DeleteEvent id) ] [ FI.toHtml [] FI.trash2 ]
            ]

        NewEvent _ ->
            [ span [] [ text "" ]
            , span [] [ text "" ]
            , span [] [ text "" ]
            , span [] [ text "" ]
            , a [ Html.Attributes.href "#", Html.Events.onClick AddEvent ] [ FI.toHtml [] FI.plus ]
            , span [] []
            ]


viewEventDetails : EventDetails -> Html EventTabMsg
viewEventDetails evDetails =
    case evDetails of
        Details { event, details } ->
            case details of
                NotAsked ->
                    div [] [ text "NotAsked" ]

                Loading ->
                    div [] [ text "Loading" ]

                Failure _ ->
                    div [] [ text "Error loading Meals" ]

                Success searchList ->
                    let
                        buttons =
                            [ a [ Html.Attributes.href "#", Html.Events.onClick CloseModal ] [ Html.text "Close" ]
                            , a [ Html.Attributes.href "#", Html.Events.onClick (SaveModal evDetails) ] [ Html.text "Save" ]
                            ]

                        fields =
                            [ Html.input
                                [ Html.Attributes.type_ "text"
                                , Html.Attributes.placeholder "Name"
                                , Html.Events.onInput (\name -> EventDetails evDetails ( Name name))
                                ]
                                []
                            ]

                        meals =
                            []
                    in
                    Modal.viewModal "Event Details" CloseModal buttons (fields ++ meals)



-- UPDATE


handleEventTabMsg : EventTabMsg -> EventsData -> ( EventsData, Cmd EventTabMsg )
handleEventTabMsg msg data =
    case Debug.log "data" data of
        Data { events, modal } ->
            case msg of
                EventListMsg searchListMsg ->
                    case events of
                        Success searchList ->
                            let
                                ( superCmd, newSearchList, cmd ) =
                                    SearchList.handleMsg handleEventMsg searchList searchListMsg
                            in
                            ( Data { events = Success newSearchList, modal = modal }, Cmd.batch [ Cmd.map EventListMsg cmd, superCmd ] )

                        _ ->
                            ( data, Cmd.none )


                OpenModal open ->
                    ( Debug.log "open" Data { events = events, modal = Just open }, Cmd.none)

                CloseModal ->
                    ( Data { events = events, modal = Nothing }, Cmd.none)

                GotWebData wdMsg ->
                    handleWebDataMsg wdMsg data

                InitTab ->
                    ( data, fetchEvents )

                SaveModal _ ->
                    Debug.todo "SaveModal"

                EventDetails ev evMsg ->
                    let
                        (details, cmd) =
                            handleEventDetailsMsg ev evMsg
                    in
                
                    ( Data { events = events, modal = Just details }, cmd )

handleWebDataMsg : WebDataMsg -> EventsData -> ( EventsData, Cmd EventTabMsg )
handleWebDataMsg msg data =
    case data of
        Data { modal } ->
            case msg of
                EventList (Ok list) ->
                    ( Data { events = newEventsList (Success <| Debug.log "" list ++ [ NewEvent { name = "", budget = "", comment = Nothing } ]), modal = modal }, Cmd.none )

                EventList (Err e) ->
                    ( Data { events = Failure e, modal = modal }, Cmd.none )



--handleEventsListMsg : SearchListMsg Event EventMsg -> WebData (SearchList Event EventMsg) -> ( WebData (SearchList Event EventMsg), Cmd (WebData (SearchList Event EventMsg)) )


handleEventMsg : EventMsg -> Event -> ( Cmd EventTabMsg, Event, Cmd EventMsg )
handleEventMsg msg event =
    case msg of
        EditEvent _ ->
            ( Cmd.map (always OpenModal <| Details { event = event, details = NotAsked }) Cmd.none, event, Cmd.none )

        DeleteEvent id ->
            ( deleteEvent id, event, Cmd.none )

        AddEvent ->
            ( Cmd.map (\_ -> OpenModal <| Details { event = Debug.log "event" event, details = NotAsked }) Cmd.none, event, Cmd.none )
        

handleEventDetailsMsg : EventDetails -> ModificationMsg -> ( EventDetails, Cmd EventTabMsg )
handleEventDetailsMsg ev msg =
    case ev of
        Details { event, details } ->
            case msg of
                Name name ->
                    ( Details { event = event, details = details }, Cmd.none )

                Budget budget ->
                    ( Details { event = event, details = details }, Cmd.none )

                Comment comment ->
                    ( Details { event = event, details = details }, Cmd.none )

                MealModification  ->
                    Debug.todo "MealModification"


-- SUBSCRIPTIONS


fetchEvents : Cmd EventTabMsg
fetchEvents =
    Http.get
        { url = backend "/events/list"
        , expect = Http.expectJson (GotWebData << EventList) eventListDecoder
        }


deleteEvent : Int -> Cmd EventTabMsg
deleteEvent id =
    Http.request
        { method = "DELETE"
        , url = backend ("/events/" ++ String.fromInt id)
        , body = Http.emptyBody
        , expect = Http.expectJson (GotWebData << EventList) eventListDecoder
        , timeout = Nothing
        , headers = []
        , tracker = Nothing
        }


eventListDecoder : Decode.Decoder (List Event)
eventListDecoder =
    Decode.list eventDecoder


eventDecoder : Decode.Decoder Event
eventDecoder =
    let
        new name budget id comment =
            Exists { name = name, budget = budget, id = id, comment = comment }
    in
    Decode.map4 new
        (Decode.field "name" Decode.string)
        (Decode.field "budget" Decode.string)
        (Decode.field "id" Decode.int)
        (Decode.field "comment" (Decode.maybe Decode.string))
