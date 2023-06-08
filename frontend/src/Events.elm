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
import Html exposing (Html, a, div, s, span, text)
import Html.Attributes
import Html.Events
import Http
import Json.Decode as Decode
import Modal
import SearchList exposing (SearchList)
import Settings exposing (backend)
import Utils.Main exposing (mapWebdata, propertyFilter, role, toWebdata)
import Utils.Model exposing (RemoteData(..), WebData)



-- MODEL


type alias EventList =
    WebData (SearchList Event EventTabMsg)


type alias MealList =
    WebData (SearchList Meal EventTabMsg)


type EventsData
    = Data
        { events : EventList
        , eventModal : Maybe EventDetails
        }


type Event
    = Exists { name : String, budget : String, id : Int, comment : Maybe String }
    | NewEvent { name : String, budget : String, comment : Maybe String }


type EventDetails
    = Details { event : Event, details : MealList, mealModal : Maybe Meal }


type Meal
    = Meal
        { event_id : Int
        , recipe_id : Int
        , recipe_name : String
        , comment : Maybe String
        , place_id : Int
        , place_name : String
        , start_time : String
        , end_time : String
        , weight : Float
        , energy : Float
        , price : String
        , servings : Int
        }
    | NewMeal



-- MESSAGES


type EventTabMsg
    = EventListMsg (SearchList.SearchListMsg Event)
    | EventDetails Event ModificationMsg
    | GotWebData WebDataMsg
    | OpenModal EventDetails
    | SaveModal EventDetails
    | DeleteEvent Int
    | CloseModal
    | InitTab


type ModificationMsg
    = Name String
    | Budget String
    | Comment String
    | MealModification MealMsg
    | EditMeal Meal
    | DeleteMeal Meal


type MealMsg
    = MealSearchMsg (SearchList.SearchListMsg Meal)
    | MealName String
    | MealComment String
    | MealPlace String
    | MealStartTime String
    | MealEndTime String
    | MealWeight String
    | MealEnergy String
    | MealPrice String
    | MealServings String
    | AddNewMeal


type WebDataMsg
    = EventList (Result Http.Error (List Event))
    | MealList (Result Http.Error (List Meal))



-- Model read/write


emptyEventsData : EventsData
emptyEventsData =
    Data { events = NotAsked, eventModal = Nothing }


eventName : Event -> String
eventName event =
    case event of
        Exists { name } ->
            name

        NewEvent { name } ->
            name


getEvent : EventDetails -> Event
getEvent details =
    case details of
        Details { event } ->
            event


setEventName : Event -> String -> Event
setEventName event name =
    case event of
        Exists { budget, id, comment } ->
            Exists { name = name, budget = budget, id = id, comment = comment }

        NewEvent { budget, comment } ->
            NewEvent { name = name, budget = budget, comment = comment }


eventBudget : Event -> String
eventBudget event =
    case event of
        Exists { budget } ->
            budget

        NewEvent { budget } ->
            budget


setEventBudget : Event -> String -> Event
setEventBudget event budget =
    case event of
        Exists { name, id, comment } ->
            Exists { name = name, budget = budget, id = id, comment = comment }

        NewEvent { name, comment } ->
            NewEvent { name = name, budget = budget, comment = comment }


eventComment : Event -> String
eventComment event =
    case event of
        Exists { comment } ->
            Maybe.withDefault "" comment

        NewEvent { comment } ->
            Maybe.withDefault "" comment


setEventComment : Event -> String -> Event
setEventComment event comment =
    case event of
        Exists { name, budget, id } ->
            Exists { name = name, budget = budget, id = id, comment = Just comment }

        NewEvent { name, budget } ->
            NewEvent { name = name, budget = budget, comment = Just comment }


mealName : Meal -> String
mealName meal =
    case meal of
        Meal { recipe_name } ->
            recipe_name

        NewMeal ->
            ""


newEventsList : WebData (List Event) -> EventList
newEventsList webData =
    mapWebdata
        (\list ->
            SearchList.new
                EventListMsg
                (propertyFilter eventName)
                viewEvent
                list
        )
        webData


newMealsList : Event -> WebData (List Meal) -> WebData (SearchList.SearchList Meal EventTabMsg)
newMealsList event webData =
    mapWebdata
        (\list ->
            SearchList.new
                (\msg -> EventDetails event <| MealModification <| MealSearchMsg msg)
                (propertyFilter mealName)
                (viewMeal event)
                list
        )
        webData


init : EventTabMsg
init =
    InitTab


modalFromEvent : Event -> EventDetails
modalFromEvent event =
    Details { event = event, details = NotAsked, mealModal = Nothing }



-- VIEW


viewEvents : EventsData -> Html EventTabMsg
viewEvents data =
    case data of
        Data { events, eventModal } ->
            case events of
                NotAsked ->
                    div [] [ text "NotAsked" ]

                Loading ->
                    div [] [ text "Loading" ]

                Failure _ ->
                    div [] [ text "Error loading Events" ]

                Success searchList ->
                    case eventModal of
                        Just m ->
                            div [] [ SearchList.view searchList, viewEventDetails m ]

                        Nothing ->
                            div [] [ SearchList.view searchList ]


viewEvent : Event -> List (Html EventTabMsg)
viewEvent event =
    case event of
        Exists { id, name, budget, comment } ->
            [ span [] [ text (String.fromInt id) ]
            , span [] [ text name ]
            , span [] [ text budget ]
            , span [] [ text (Maybe.withDefault "" comment) ]
            , a [ Html.Attributes.href "#", Html.Events.onClick <| OpenModal <| modalFromEvent event ] [ FI.toHtml [] FI.edit ]
            , a [ Html.Attributes.href "#", Html.Events.onClick (DeleteEvent id) ] [ FI.toHtml [] FI.trash2 ]
            ]

        NewEvent _ ->
            [ span [] [ text "" ]
            , span [] [ text "" ]
            , span [] [ text "" ]
            , span [] [ text "" ]
            , a [ Html.Attributes.href "#", Html.Events.onClick <| OpenModal <| modalFromEvent event ] [ FI.toHtml [] FI.plus ]
            , span [] []
            ]


viewEventDetails : EventDetails -> Html EventTabMsg
viewEventDetails evDetails =
    case evDetails of
        Details { event, details } ->
            let
                buttons =
                    [ a
                        [ Html.Attributes.href "#"
                        , Html.Events.onClick CloseModal
                        , role "button"
                        , Html.Attributes.class "secondary"
                        ]
                        [ Html.text "Close" ]
                    , a
                        [ Html.Attributes.href "#"
                        , Html.Events.onClick (SaveModal evDetails)
                        , role "button"
                        ]
                        [ Html.text "Save" ]
                    ]

                ( nameField, budgetField, commentField ) =
                    ( Html.input
                        [ Html.Attributes.type_ "text"
                        , Html.Attributes.placeholder "Name"
                        , Html.Events.onInput (\name -> EventDetails event (Name name))
                        , Html.Attributes.value (eventName event)
                        ]
                        []
                    , Html.input
                        [ Html.Attributes.type_ "text"
                        , Html.Attributes.placeholder "Budget"
                        , Html.Events.onInput (\budget -> EventDetails event (Budget budget))
                        , Html.Attributes.value (eventBudget event)
                        ]
                        []
                    , Html.input
                        [ Html.Attributes.type_ "text"
                        , Html.Attributes.placeholder "Comment"
                        , Html.Events.onInput (\comment -> EventDetails event (Comment comment))
                        , Html.Attributes.value (eventComment event)
                        ]
                        []
                    )

                fields =
                    [ div [ Html.Attributes.class "grid" ] [ nameField, budgetField ], commentField ]

                meals =
                    case details of
                        NotAsked ->
                            div [] [ text "NotAsked" ]

                        Loading ->
                            div [] [ text "Loading" ]

                        Failure _ ->
                            div [] [ text "Error loading Meals" ]

                        Success searchList ->
                            SearchList.view <| SearchList.addAll [ NewMeal ] searchList
            in
            Modal.viewModal "Event Details" CloseModal buttons (fields ++ [ meals ])


viewMeal : Event -> Meal -> List (Html EventTabMsg)
viewMeal event meal =
    case meal of
        Meal { recipe_name, place_name, start_time, price, weight, servings } ->
            [ span [] [ text recipe_name ]
            , span [] [ text place_name ]
            , span [] [ text start_time ]
            , span [] [ text (price ++ "€") ]
            , span [] [ text (String.fromFloat weight ++ "kg") ]
            , span [] [ text (String.fromInt servings) ]
            , a
                [ Html.Attributes.href "#"
                , Html.Events.onClick (EventDetails event <| EditMeal meal)
                ]
                [ FI.toHtml [] FI.edit ]
            , a
                [ Html.Attributes.href "#"
                , Html.Events.onClick (EventDetails event <| DeleteMeal meal)
                ]
                [ FI.toHtml [] FI.trash2 ]
            ]

        -- Adding a new meal
        NewMeal ->
            [ span [] [ text "" ]
            , span [] [ text "" ]
            , span [] [ text "" ]
            , span [] [ text "" ]
            , span [] [ text "" ]
            , span [] [ text "" ]
            , a
                [ Html.Attributes.href "#"
                , Html.Events.onClick (EventDetails event <| MealModification <| AddNewMeal)
                ]
                [ FI.toHtml [] FI.edit ]
            ]



-- UPDATE


handleEventTabMsg : EventTabMsg -> EventsData -> ( EventsData, Cmd EventTabMsg )
handleEventTabMsg msg data =
    case data of
        Data { events, eventModal } ->
            case msg of
                EventListMsg searchListMsg ->
                    case events of
                        Success searchList ->
                            let
                                ( superCmd, newSearchList, cmd ) =
                                    SearchList.handleMsg searchList searchListMsg
                            in
                            ( Data { events = Success newSearchList, eventModal = eventModal }, Cmd.batch [ Cmd.map EventListMsg cmd, superCmd ] )

                        _ ->
                            ( data, Cmd.none )

                OpenModal open ->
                    let
                        ( openModal, cmd ) =
                            case open of
                                Details { event, mealModal } ->
                                    case event of
                                        Exists { id } ->
                                            ( Just open, fetchMeals id )

                                        NewEvent _ ->
                                            ( Just <| Details { event = event, details = newMealsList event <| Success [], mealModal = mealModal }
                                            , Cmd.none
                                            )
                    in
                    ( Data { events = events, eventModal = openModal }, cmd )

                CloseModal ->
                    ( Data { events = events, eventModal = Nothing }, Cmd.none )

                GotWebData wdMsg ->
                    handleWebDataMsg wdMsg data

                InitTab ->
                    ( data, fetchEvents )

                SaveModal _ ->
                    Debug.todo "SaveModal"

                EventDetails ev evMsg ->
                    case eventModal of
                        Nothing ->
                            ( data, Cmd.none )

                        Just evDetails ->
                            let
                                ( details, cmd ) =
                                    handleEventDetailsMsg evDetails evMsg
                            in
                            if ev == getEvent evDetails then
                                ( data, Cmd.none )

                            else
                                ( Data { events = events, eventModal = Just details }, cmd )

                DeleteEvent id ->
                    ( data, deleteEvent id )


handleWebDataMsg : WebDataMsg -> EventsData -> ( EventsData, Cmd EventTabMsg )
handleWebDataMsg msg data =
    case data of
        Data { eventModal, events } ->
            case msg of
                EventList (Ok list) ->
                    ( Data { events = newEventsList (Success <| list ++ [ NewEvent { name = "", budget = "", comment = Nothing } ]), eventModal = eventModal }, Cmd.none )

                EventList (Err e) ->
                    ( Data { events = Failure e, eventModal = eventModal }, Cmd.none )

                MealList list ->
                    let
                        setdetails =
                            case eventModal of
                                Just (Details { event }) ->
                                    Just (Details { event = event, details = newMealsList event (toWebdata list), mealModal = Nothing })

                                _ ->
                                    eventModal
                    in
                    ( Data
                        { events = events
                        , eventModal = setdetails
                        }
                    , Cmd.none
                    )



--handleEventsListMsg : SearchListMsg Event EventMsg -> WebData (SearchList Event EventMsg) -> ( WebData (SearchList Event EventMsg), Cmd (WebData (SearchList Event EventMsg)) )


handleEventDetailsMsg : EventDetails -> ModificationMsg -> ( EventDetails, Cmd EventTabMsg )
handleEventDetailsMsg ev msg =
    case ev of
        Details { event, details, mealModal } ->
            case msg of
                Name name ->
                    ( Details { event = setEventName event name, details = details, mealModal = mealModal }, Cmd.none )

                Budget budget ->
                    ( Details { event = setEventBudget event budget, details = details, mealModal = mealModal }, Cmd.none )

                Comment comment ->
                    ( Details { event = setEventComment event comment, details = details, mealModal = mealModal }, Cmd.none )

                EditMeal meal ->
                    ( Details { event = event, details = details, mealModal = Just meal }, Cmd.none )

                DeleteMeal meal ->
                    Debug.todo "MealModification"

                MealModification _ ->
                    Debug.todo "MealModification"



-- SUBSCRIPTIONS


fetchEvents : Cmd EventTabMsg
fetchEvents =
    Http.get
        { url = backend "/events/list"
        , expect = Http.expectJson (GotWebData << EventList) eventListDecoder
        }


fetchMeals : Int -> Cmd EventTabMsg
fetchMeals id =
    Http.get
        { url = backend ("/events/" ++ String.fromInt id ++ "/meals/list")
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
