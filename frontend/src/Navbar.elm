module Navbar exposing (..)

import Cursor exposing (Cursor)
import Html exposing (a, li, ul)
import Html.Attributes exposing (class)
import Html.Events exposing (onClick)
import State exposing (Msg(..), Tab)


generateNavbar : (Tab -> String) -> Cursor Tab -> Html.Html Msg
generateNavbar view tabs =
    let
        l =
            Cursor.left tabs |> List.map (generateNavbarItem False view)

        a =
            generateNavbarItem True view (Cursor.active tabs)

        r =
            Cursor.right tabs |> List.map (generateNavbarItem False view)
    in
    ul ([ "navbar" ] |> List.map class)
        (l ++ a :: r)


generateNavbarItem : Bool -> (Tab -> String) -> Tab -> Html.Html Msg
generateNavbarItem active view tab =
    if active then
        li
            [ class "navbar-item active"
            , onClick (ChangeTab <| tab)
            ]
            [ a [ class "navbar-link" ] [ Html.text <| view <| tab ]
            ]

    else
        li
            [ class "navbar-item"
            , onClick (ChangeTab <| tab)
            ]
            [ a [ class "navbar-link" ] [ Html.text <| view <| tab ]
            ]
