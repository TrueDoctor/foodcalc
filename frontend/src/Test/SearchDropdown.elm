module Test.SearchDropdown exposing (..)

import Element exposing (..)
import Element.Background
import Element.Border
import Element.Events exposing (onClick)
import Element.Input
import Html.Attributes exposing (hidden)
import Test.Styles exposing (grey, white)


type alias SearchDropdown a msg =
    { search : String
    , filter : String -> a -> Bool
    , items : List a
    , selection : Maybe a
    , viewItem : a -> Element msg
    , filterChange : String -> msg
    , select : a -> msg
    , onFocus : msg
    , hidden : Bool
    }


viewOverlay : SearchDropdown a msg -> Element msg
viewOverlay dropdown =
    column
        [ width fill
        , Element.Background.color white
        , padding 10
        , Element.Border.width 1
        ]
    <|
        List.map 
            (\i -> el [ onClick (dropdown.select i), width fill ] (dropdown.viewItem i)) 
            (List.filter (dropdown.filter dropdown.search) dropdown.items)


searchDropdown : SearchDropdown a msg -> Element msg
searchDropdown dropdown =
    if dropdown.hidden then
        el
            [ padding 10
            , width fill
            , onClick dropdown.onFocus
            ]
        <|
            Element.Input.text [ width fill ]
                { label = Element.Input.labelHidden "Filter"
                , onChange = dropdown.filterChange
                , placeholder = Just <| Element.Input.placeholder [] (Maybe.withDefault (text "") (Maybe.map dropdown.viewItem dropdown.selection))
                , text = ""
                }

    else
        el
            [ padding 10
            , width fill
            , below (viewOverlay dropdown)
            , onClick dropdown.onFocus
            ]
        <|
            Element.Input.text [ width fill ]
                { label = Element.Input.labelHidden "Filter"
                , onChange = dropdown.filterChange
                , placeholder = Just <| Element.Input.placeholder [] (text "Filter")
                , text = dropdown.search
                }
