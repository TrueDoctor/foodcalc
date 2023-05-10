module Navbar exposing (..)

import Cursor exposing (Cursor)
import Html exposing (a, li, nav, strong, ul)
import Html.Attributes exposing (attribute, class)
import Html.Events exposing (onClick)
import State exposing (Msg(..), Tab)
import Util exposing (roleAttr)


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
    nav []
        [ ul [] [ li [] [ strong [] [ Html.text "foodcalc" ] ] ]
        , ul [] (l ++ a :: r)
        ]


generateNavbarItem : Bool -> (Tab -> String) -> Tab -> Html.Html Msg
generateNavbarItem active view tab =
    if active then
        li
            [ onClick (ChangeTab <| tab) ]
            [ a [ roleAttr "button" ] [ Html.text <| view <| tab ] ]

    else
        li
            [ onClick (ChangeTab <| tab) ]
            [ a [ ] [ Html.text <| view <| tab ] ]
