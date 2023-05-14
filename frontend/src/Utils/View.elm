module Utils.View exposing (filterListView, listView)

import FeatherIcons as FI
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Model exposing (Msg(..))
import Utils.Main exposing (role)


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
