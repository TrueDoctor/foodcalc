module Meals exposing (MealList, MealListMsg, Place, Recipe, emptyList, fetchMeals, update, view)

import Element exposing (..)
import Element.Background
import Element.Border
import Element.Input
import Html
import Http
import Json.Decode
import Settings exposing (backend)
import Test.ExpandableList as ExpandableList exposing (ExpandableList, ExpandableListMsg, mapElementMsg)
import Test.SearchDropdown exposing (SearchDropdownData, searchDropdown)
import Test.StringUtils exposing (fuzzyContains)
import Test.Styles exposing (white)
import WebData exposing (RemoteData(..), WebData, errorString)


type MealList
    = MealList
        { meals : WebData (ExpandableList Meal MealListMsg MealMsg)
        , eventId : Maybe Int
        , recipes : WebData (List Recipe)
        , places : WebData (List Place)
        }


type Meal
    = Meal
        { data : MealData
        , edit : MealData
        }


type alias MealData =
    { eventId : Maybe Int
    , recipe : Maybe Recipe
    , place : Maybe Place
    , startTime : String
    , endTime : String
    , comment : Maybe String
    , weight : String
    , energy : String
    , price : String
    , servings : String
    , recipeList : WebData (SearchDropdownData Recipe)
    , placeList : WebData (SearchDropdownData Place)
    }


type alias Place =
    { id : Int, name : String }


type alias Recipe =
    { id : Int, name : String }


type MealListMsg
    = ListMsg (ExpandableListMsg Meal MealMsg)
    | GotWebData WebDataMsg


type WebDataMsg
    = GotMeals (Result Http.Error (List ( Bool, Meal )))


type MealMsg
    = Save
    | RecipeChanged Recipe
    | RecipeFilterChanged String
    | RecipeFocus
    | PlaceChanged Place
    | PlaceFilterChanged String
    | PlaceFocus
    | StartTimeChanged String
    | EndTimeChanged String
    | CommentChanged String
    | EnergyChanged String
    | ServingsChanged String
    | FetchData



-- setup


initList : Maybe Int -> String -> List ( Bool, Meal ) -> WebData (ExpandableList Meal MealListMsg MealMsg)
initList eventId search items =
    let
        filter : String -> Meal -> Bool
        filter string meal =
            case meal of
                Meal { data } ->
                    case data.recipe of
                        Just recipe ->
                            fuzzyContains string recipe.name

                        Nothing ->
                            string == ""
    in
    Success
        { search = search
        , filter = filter
        , items = items
        , viewElement = viewMeal
        , mapMsg = ListMsg
        , update = updateMeal
        , add = Just <| always <| newMeal eventId Nothing Nothing Nothing "" "" "" ""
        , expandItem = Just FetchData
        }


emptyList : Maybe Int -> MealList
emptyList id =
    MealList
        { meals = NotAsked
        , eventId = id
        , recipes = NotAsked
        , places = NotAsked
        }


newMeal : Maybe Int -> Maybe Recipe -> Maybe Place -> Maybe String -> String -> String -> String -> String -> Meal
newMeal eventId recipe place comment startTime endTime energy servings =
    let
        data : MealData
        data =
            { eventId = eventId
            , recipe = recipe
            , place = place
            , startTime = startTime
            , endTime = endTime
            , comment = comment
            , weight = ""
            , energy = energy
            , price = ""
            , servings = servings
            , recipeList = NotAsked
            , placeList = NotAsked
            }
    in
    Meal { data = data, edit = data }



-- view


view : MealList -> Element MealListMsg
view model =
    let
        wd =
            case model of
                MealList { meals } ->
                    meals
    in
    case wd of
        Success data ->
            ExpandableList.view data

        Failure e ->
            el [] <| text <| errorString e

        _ ->
            el [] <| text "Loading Meals..."


viewMeal : Attribute MealListMsg -> Bool -> Meal -> Element MealListMsg
viewMeal expand expanded meal =
    case meal of
        Meal { edit } ->
            column [ width fill ]
                [ el [ expand, width fill ] (viewRow edit)
                , if expanded then
                    viewExpanded meal

                  else
                    none
                ]


viewRow : MealData -> Element msg
viewRow meal =
    row [ width fill ]
        [ el [ width (fillPortion 3) ] <| text meal.startTime
        , el [ width (fillPortion 3) ] <| text <| Maybe.withDefault "" <| Maybe.map .name meal.recipe
        , el [ width (fillPortion 3) ] <| text <| Maybe.withDefault "" <| Maybe.map .name meal.place
        , el [ width (fillPortion 1) ] <| text <| meal.servings ++ "x"
        , el [ width (fillPortion 1) ] <| text <| meal.energy ++ "kJ"
        , el [ width (fillPortion 1) ] <| text <| meal.price ++ "eur"
        , el [ width (fillPortion 1) ] <| text <| meal.weight ++ "g"
        ]


viewExpanded : Meal -> Element MealListMsg
viewExpanded meal =
    let
        data =
            case meal of
                Meal m ->
                    m

        editRecipe =
            case data.edit.recipeList of
                Success r ->
                    searchDropdown
                        { select = RecipeChanged
                        , itemName = .name
                        , filterChange = RecipeFilterChanged
                        , onFocus = RecipeFocus
                        , title = "Recipe"
                        }
                        r

                _ ->
                    text <| Maybe.withDefault "" <| Maybe.map .name data.edit.recipe

        editPlace =
            case data.edit.placeList of
                Success p ->
                    searchDropdown
                        { select = PlaceChanged
                        , itemName = .name
                        , filterChange = PlaceFilterChanged
                        , onFocus = PlaceFocus
                        , title = "Place"
                        }
                        p

                _ ->
                    text <| Maybe.withDefault "" <| Maybe.map .name data.edit.place

        editStartTime =
            Element.Input.text []
                { onChange = StartTimeChanged
                , text = data.edit.startTime
                , label = Element.Input.labelAbove [] <| text "Start Time"
                , placeholder = Just <| Element.Input.placeholder [] <| text "yyyy-mm-dd hh:mm"
                }

        editEndTime =
            Element.Input.text []
                { onChange = EndTimeChanged
                , text = data.edit.endTime
                , label = Element.Input.labelAbove [] <| text "End Time"
                , placeholder = Just <| Element.Input.placeholder [] <| text "yyyy-mm-dd hh:mm"
                }

        commentChanged =
            Element.Input.text []
                { onChange = CommentChanged
                , text = data.edit.comment |> Maybe.withDefault ""
                , label = Element.Input.labelAbove [] <| text "Comment"
                , placeholder = Just <| Element.Input.placeholder [] <| text "Comment"
                }

        energyChanged =
            Element.Input.text []
                { onChange = EnergyChanged
                , text = data.edit.energy
                , label = Element.Input.labelAbove [] <| text "Energy"
                , placeholder = Just <| Element.Input.placeholder [] <| text "Energy"
                }

        servingsChanged =
            Element.Input.text []
                { onChange = ServingsChanged
                , text = data.edit.servings
                , label = Element.Input.labelAbove [] <| text "Servings"
                , placeholder = Just <| Element.Input.placeholder [] <| text "Servings"
                }

        rowSettings =
            [ width fill, spacing 10 ]
    in
    Element.map (ListMsg << mapElementMsg meal) <|
        column
            [ Element.Background.color Test.Styles.white
            , width fill
            , padding 10
            , spacing 10
            , Element.Border.rounded 5
            ]
            [ row rowSettings [ editRecipe, editPlace ]
            , row rowSettings [ editStartTime, editEndTime ]
            , row rowSettings [ energyChanged, servingsChanged ]
            , commentChanged
            ]


updateMeal : MealMsg -> Meal -> ( Meal, Cmd MealListMsg )
updateMeal msg meal =
    case ( msg, meal ) of
        ( Save, Meal { data, edit } ) ->
            ( meal, Cmd.none )

        _ ->
            ( meal, Cmd.none )


update : MealListMsg -> MealList -> ( MealList, Cmd MealListMsg )
update msg model =
    let
        list =
            case model of
                MealList mealList ->
                    Debug.log "" mealList
    in
    case ( msg, list.meals ) of
        ( ListMsg listMsg, Success meals ) ->
            let
                ( newMeals, cmd ) =
                    ExpandableList.update (Debug.log "mealListMsg" listMsg) meals
            in
            ( MealList { list | meals = Success newMeals }, cmd )

        ( GotWebData (GotMeals (Ok meals)), _ ) ->
            let
                newMeals =
                    initList list.eventId "" meals
            in
            ( MealList { list | meals = newMeals }, Cmd.none )

        ( GotWebData (GotMeals (Err e)), _ ) ->
            ( MealList { list | meals = Failure e }, Cmd.none )

        _ ->
            ( model, Cmd.none )



-- Decoding


decodeMeal : Json.Decode.Decoder Meal
decodeMeal =
    Json.Decode.map8 newMeal
        (Json.Decode.maybe <| Json.Decode.field "eventId" Json.Decode.int)
        (Json.Decode.maybe <| Json.Decode.field "recipe" decodeRecipe)
        (Json.Decode.maybe <| Json.Decode.field "place" decodePlace)
        (Json.Decode.field "comment" <| Json.Decode.nullable Json.Decode.string)
        (Json.Decode.field "startTime" Json.Decode.string)
        (Json.Decode.field "endTime" Json.Decode.string)
        (Json.Decode.field "energy" Json.Decode.string)
        (Json.Decode.field "servings" Json.Decode.string)


decodeRecipe : Json.Decode.Decoder Recipe
decodeRecipe =
    Json.Decode.map2 Recipe
        (Json.Decode.field "id" Json.Decode.int)
        (Json.Decode.field "name" Json.Decode.string)


decodePlace : Json.Decode.Decoder Place
decodePlace =
    Json.Decode.map2 Place
        (Json.Decode.field "id" Json.Decode.int)
        (Json.Decode.field "name" Json.Decode.string)


decodeMeals : Json.Decode.Decoder (List ( Bool, Meal ))
decodeMeals =
    Json.Decode.list <| Json.Decode.map2 Tuple.pair (Json.Decode.succeed False) decodeMeal



-- fetch


fetchMeals : Maybe Int -> Cmd MealListMsg
fetchMeals eventId =
    let
        url id =
            backend "/events/" ++ String.fromInt id ++ "/meals/list"
    in
    case eventId of
        Just id ->
            Http.get
                { url = url id
                , expect = Http.expectJson (GotWebData << GotMeals) decodeMeals
                }

        _ ->
            Cmd.none
