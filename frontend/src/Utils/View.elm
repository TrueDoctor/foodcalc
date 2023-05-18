module Utils.View exposing (filterListView, listView, searchableDropdown, showWebData, dropdown)

import FeatherIcons as FI
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Model exposing (Msg(..))
import Utils.Main exposing (nameFilter, role)
import Utils.Model exposing (DropdownData, DropdownEvents, RemoteData(..), WebData)


searchBar : (String -> Msg) -> Msg -> Html Msg
searchBar filterChange add =
    table []
        [ tr []
            [ td [] [ input [ class "search", type_ "text", placeholder "Search", onInput filterChange ] [] ]
            , td [] [ button [ onClick add ] [ FI.toHtml [] FI.plus ] ]
            ]
        ]


filterListView :
    { row : a -> List (Html Msg)
    , filter : a -> Bool
    , filterChange : String -> Msg
    , onAdd : Msg
    }
    -> List a
    -> Html Msg
filterListView options list =
    let
        filtered =
            List.filter options.filter list
    in
    div []
        [ searchBar options.filterChange options.onAdd
        , listView options.row filtered
        ]


listView : (a -> List (Html Msg)) -> List a -> Html Msg
listView row list =
    let
        rows =
            List.map (tr [] << List.map (\x -> td [] [ x ]) << row) list
    in
    table [ role "grid" ] [ tbody [] rows ]


search : (String -> msg) -> String -> Html msg
search action value =
    input [ class "search", type_ "text", placeholder "Filter...", onInput action, Html.Attributes.value value ] []


searchableDropdown : DropdownData a -> DropdownEvents a msg -> List a -> Html msg
searchableDropdown data ev list =
    let
        filteredList =
            List.filter (nameFilter data.filter << ev.property) list

        selectedProperty =
            data.selected 
            |> Maybe.map ev.property
            |> Maybe.withDefault ""
        options =
            List.map (\x -> li [] [ a [ onClick <| ev.onSelect x ] [ text <| ev.property x ] ]) filteredList
    in
    details [ role "list" ]
        [ summary [ attribute "aria-haspopup" "listbox" ] [ text selectedProperty ]
        , ul [ role "listbox" ] <| search ev.onFilter data.filter :: options
        ]


dropdown : String -> (String -> msg) -> List String -> Html msg
dropdown filter filterChange list =
    let
        options =
            List.map (\x -> option [ value x ] [ text x ]) list
    in
    select [ class "search", placeholder filter, onInput filterChange ] options


showWebData : (a -> Html msg) -> WebData a -> Html msg
showWebData f data =
    case data of
        NotAsked ->
            text ""

        Loading ->
            text "Loading..."

        Success x ->
            f x

        Failure x ->
            text <| "Error: " ++ Debug.toString x
