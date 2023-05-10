module Util exposing (..)

import Http 
import State exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Html exposing (..)

mapWebdata : Result Http.Error a -> WebData a
mapWebdata r =
    case r of
        Ok a ->
            Success a

        Err e ->
            Failure (Debug.log "" e)

roleAttr : String -> Attribute msg
roleAttr = attribute "role"
