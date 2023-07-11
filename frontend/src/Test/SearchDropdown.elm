module Test.SearchDropdown exposing (..)

import Element exposing (..)
import Element.Background
import Element.Border
import Element.Events exposing (onClick)
import Element.Input
import Html.Attributes exposing (hidden)
import Test.Styles exposing (grey, white)
import Test.StringUtils exposing (fuzzyContains)
import Test.StringUtils exposing (fuzzyContainedBy)


type alias SearchDropdown a msg =
    { search : String
    , items : List a
    , selection : Maybe a
    , itemName : a -> String
    , filterChange : String -> msg
    , select : a -> msg
    , onFocus : msg
    , hidden : Bool
    , title : String
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
            (\i -> el [ onClick (dropdown.select i), width fill ] (text <| dropdown.itemName i)) 
            (List.filter (fuzzyContainedBy dropdown.search << dropdown.itemName) dropdown.items)


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
                { label = Element.Input.labelAbove [] <| text dropdown.title
                , onChange = dropdown.filterChange
                , placeholder = Just <| Element.Input.placeholder [] <| text (Maybe.withDefault "" (Maybe.map dropdown.itemName dropdown.selection))
                , text = Maybe.withDefault "" <| Maybe.map dropdown.itemName dropdown.selection
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
                { label = Element.Input.labelAbove [] <| text dropdown.title
                , onChange = dropdown.filterChange
                , placeholder = Just <| Element.Input.placeholder [] (text "Filter")
                , text = dropdown.search
                }
