module Test.Styles exposing (..)

import Element exposing (rgb, rgb255)
import Element.Border
import Element.Font


red : Element.Color
red =
    Element.rgb 1 0 0


white : Element.Color
white =
    Element.rgb 1 1 1


grey : Element.Color
grey =
    rgb 0.4 0.4 0.4


grey10 : Element.Color
grey10 =
    rgb255 245 245 245


grey20 : Element.Color
grey20 =
    rgb255 235 235 235


invalidButton : List (Element.Attr decorative msg)
invalidButton =
    [ Element.Border.color red
    , Element.Font.color red
    ]
