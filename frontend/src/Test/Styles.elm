module Test.Styles exposing (..)
import Element.Border
import Element
import Element.Font

red : Element.Color
red = Element.rgb 1 0 0

white : Element.Color
white = Element.rgb 1 1 1


invalidButton : List (Element.Attr decorative msg)
invalidButton =
    [ Element.Border.color red
    , Element.Font.color red
    ]