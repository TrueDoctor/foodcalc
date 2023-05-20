module SearchList exposing (SearchList, SearchListMsg, addAll, empty,new, handleMsg, view)

import Html exposing (Html, div, input, li, ul)
import Html.Events exposing (onInput)


type SearchList a msg
    = SearchList
        { list : List a
        , search : String
        , filter : String -> a -> Bool
        , viewContent : a -> List (Html msg)
        }


type SearchListMsg a itemMsg
    = SetSearch String
    | ItemMsg a itemMsg


empty : (String -> a -> Bool) -> (a -> List (Html msg)) -> SearchList a msg
empty filter viewContent =
    SearchList
        { list = []
        , search = ""
        , filter = filter
        , viewContent = viewContent
        }

new : (String -> a -> Bool) -> (a -> List (Html msg)) -> List a -> SearchList a msg
new filter viewContent list =
    addAll list (empty filter viewContent)

addAll : List a -> SearchList a msg -> SearchList a msg
addAll list searchList =
    case searchList of
        SearchList s ->
            SearchList { s | list = list ++ s.list }


view : SearchList a msg -> Html (SearchListMsg a msg)
view searchList =
    div []
        [ input [ onInput SetSearch ] []
        , ul [] (List.map (viewItem searchList) (filterList searchList))
        ]


viewItem : SearchList a msg -> a -> Html (SearchListMsg a msg)
viewItem searchList item =
    case searchList of
        SearchList s ->
            li [] (List.map (Html.map <| ItemMsg item) (s.viewContent item))


filterList : SearchList a b -> List a
filterList searchList =
    case searchList of
        SearchList { list, search, filter } ->
            List.filter (filter search) list


handleMsg : (msg -> a -> ( Cmd superMsg,  a, Cmd msg )) -> SearchList a msg -> SearchListMsg a msg -> ( Cmd superMsg,  SearchList a msg, Cmd (SearchListMsg a msg) ) 
handleMsg handleItemMsg searchList msg =
    case searchList of
        SearchList s ->
            case msg of
                SetSearch newSearch ->
                    ( Cmd.none, SearchList { s | search = newSearch }, Cmd.none  )

                ItemMsg item itemMsg ->
                    let
                        ( super,  newItem, cmd  ) =
                            handleItemMsg itemMsg item

                        updateOne listItem =
                            if listItem == item then
                                newItem

                            else
                                listItem
                    in
                    ( super,  SearchList { s | list = List.map updateOne s.list }, Cmd.map (ItemMsg item) cmd ) 
