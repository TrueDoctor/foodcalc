module Test.Styles exposing (..)
import Browser.Dom exposing (Element)
import Element.Border
import Element
import Element.Font

red : Element.Color
red = Element.rgb 1 0 0

invalidButton : List (Element.Attr decorative msg)
invalidButton =
    [ Element.Border.color red
    , Element.Font.color red
    ]