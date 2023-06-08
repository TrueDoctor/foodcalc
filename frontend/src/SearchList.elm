module SearchList exposing (SearchList, SearchListMsg, addAll, empty, handleMsg, new, view, filter)

import Html exposing (Html, div, input, li, ul)
import Html.Events exposing (onInput)


type SearchList a msg
    = SearchList
        { list : List a
        , search : String
        , viewFilter : String -> a -> Bool
        , viewContent : a -> List (Html msg)
        , mapMsg : SearchListMsg a -> msg
        }


type SearchListMsg a
    = SetSearch String


empty : (SearchListMsg a -> msg) -> (String -> a -> Bool) -> (a -> List (Html msg)) -> SearchList a msg
empty mapMsg viewFilter viewContent =
    SearchList
        { list = []
        , search = ""
        , viewFilter = viewFilter
        , viewContent = viewContent
        , mapMsg = mapMsg
        }


new : (SearchListMsg a -> msg) -> (String -> a -> Bool) -> (a -> List (Html msg)) -> List a -> SearchList a msg
new mapMsg viewFilter viewContent list =
    addAll list (empty mapMsg viewFilter viewContent)


filter : (a -> Bool) -> SearchList a msg -> SearchList a msg
filter listFilter searchList =
    case searchList of
        SearchList s ->
            SearchList { s | list = List.filter listFilter s.list }


addAll : List a -> SearchList a msg -> SearchList a msg
addAll list searchList =
    case searchList of
        SearchList s ->
            SearchList { s | list = list ++ s.list }


view : SearchList a msg -> Html msg
view searchList =
    case searchList of
        SearchList { mapMsg } ->
            div []
                [ input [ onInput <| \i -> mapMsg <| SetSearch i ] []
                , ul [] (List.map (viewItem searchList) (filterList searchList))
                ]


viewItem : SearchList a msg -> a -> Html msg
viewItem searchList item =
    case searchList of
        SearchList { viewContent } ->
            li [] (viewContent item)


filterList : SearchList a b -> List a
filterList searchList =
    case searchList of
        SearchList { list, search, viewFilter } ->
            List.filter (viewFilter search) list


handleMsg : SearchList a msg -> SearchListMsg a -> ( Cmd superMsg, SearchList a msg, Cmd (SearchListMsg a) )
handleMsg searchList msg =
    case searchList of
        SearchList s ->
            case msg of
                SetSearch newSearch ->
                    ( Cmd.none, SearchList { s | search = newSearch }, Cmd.none )
