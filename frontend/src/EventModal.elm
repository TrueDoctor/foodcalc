module EventModal exposing (Event)

import MealModal
import Events exposing (EventTabMsg)
import Forms
import Html 
import Modal
import SearchList exposing (SearchList)
import WebData exposing (WebData)



-- IMPORTS
-- TYPES


type alias MealList =
    WebData (SearchList MealModal.Meal EventTabMsg)


type Event msg
    = Event
        { name : String
        , budget : String
        , id : Maybe Int
        , comment : Maybe String
        , meals : MealList
        , mealModal : Maybe MealModal.Meal
        , closeMsg : msg
        , deleteMsg : msg
        , mapMsg : EventMsg -> msg
        , open : Bool
        }





-- MESSAGES


type EventMsg
    = Name String
    | Budget String
    | Comment String
    | MealModification MealModal.MealMsg
    | OpenModal
    | Save
    | EditMeal MealModal.Meal
    | DeleteMeal MealModal.Meal




-- BUILDERS
-- GETTERS
-- for name, comment, budget, etc


id : Event msg -> Maybe Int
id event =
    case event of
        Event e ->
            e.id


name : Event msg -> String
name event =
    case event of
        Event e ->
            e.name


budget : Event msg -> String
budget event =
    case event of
        Event e ->
            e.budget


comment : Event msg -> Maybe String
comment event =
    case event of
        Event e ->
            e.comment


meals : Event msg -> MealList
meals event =
    case event of
        Event e ->
            e.meals


mealModal : Event msg -> Maybe MealModal.Meal
mealModal event =
    case event of
        Event e ->
            e.mealModal


closeMsg : Event msg -> msg
closeMsg event =
    case event of
        Event e ->
            e.closeMsg


deleteMsg : Event msg -> msg
deleteMsg event =
    case event of
        Event e ->
            e.deleteMsg


mapMsg : Event msg -> EventMsg -> msg
mapMsg event msg =
    case event of
        Event e ->
            e.mapMsg msg



-- SETTERS


setName : String -> Event msg -> Event msg
setName newName event =
    case event of
        Event e ->
            Event { e | name = newName }


setBudget : String -> Event msg -> Event msg
setBudget newBudget event =
    case event of
        Event e ->
            Event { e | budget = newBudget }


setComment : Maybe String -> Event msg -> Event msg
setComment newComment event =
    case event of
        Event e ->
            Event { e | comment = newComment }


setMeals : MealList -> Event msg -> Event msg
setMeals newMeals event =
    case event of
        Event e ->
            Event { e | meals = newMeals }


setMealModal : Maybe MealModal.Meal -> Event msg -> Event msg
setMealModal newMealModal event =
    case event of
        Event e ->
            Event { e | mealModal = newMealModal }



-- UPDATE


update : EventMsg -> Event msg -> Event msg
update msg event =
    case msg of
        Name newName ->
            setName newName event

        Budget newBudget ->
            setBudget newBudget event

        Comment newComment ->
            setComment (Just newComment) event

        MealModification mealMsg ->
            Debug.todo "MealModification"

        OpenModal ->
            Debug.todo "OpenModal"

        EditMeal meal ->
            setMealModal (Just meal) event

        Save ->
            Debug.todo "Save"

        DeleteMeal meal ->
            Debug.todo "DeleteMeal"



-- VIEW

view : Event msg -> Html.Html msg
view event =
    case event of 
        Event {open} ->
            if open then
                viewModal event
            else
                listItem event
    

listItem : Event msg -> Html.Html msg
listItem event =
    Forms.displayRow
        (mapMsg event OpenModal)
        (deleteMsg event)
        [ String.fromInt (Maybe.withDefault 0 (id event))
        , name event
        , budget event
        , Maybe.withDefault "" (comment event)
        ]


viewModal : Event msg -> Html.Html msg
viewModal event =
    let
        content =
            [ Forms.inputText "Name" (mapMsg event << Name) (name event)
            , Forms.inputText "Budget" (mapMsg event << Budget) (budget event)
            , Forms.inputText "Comment" (mapMsg event << Comment) (Maybe.withDefault "" (comment event))

            -- TODO: view meals
            ]

        footer =
            [ Forms.button "Save" (mapMsg event Save)
            , Forms.button "Cancel" (closeMsg event)
            ]
    in
    Modal.viewModal
        "Event"
        (closeMsg event)
        footer
        content
